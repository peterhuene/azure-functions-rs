use crate::durable::{Action, EventType, HistoryEvent};
use crate::{
    durable::ExecutionResult,
    rpc::{typed_data::Data, TypedData},
};
use chrono::{DateTime, Utc};
use futures::future::{join_all, FutureExt};
use serde::Deserialize;
use serde_json::{from_str, Value};
use std::{
    cell::RefCell,
    collections::HashMap,
    future::Future,
    pin::Pin,
    rc::Rc,
    task::{Context, Poll},
};

/// Represents a Future returned by the orchestration context.
pub trait OrchestrationFuture: Future {
    #[doc(hidden)]
    fn notify_inner(&mut self);

    #[doc(hidden)]
    fn event_index(&self) -> Option<usize>;
}

struct ActionFuture<T> {
    result: Option<T>,
    state: Rc<RefCell<OrchestrationState>>,
    event_index: Option<usize>,
    is_inner: bool,
}

impl<T> ActionFuture<T> {
    fn new(
        result: Option<T>,
        state: Rc<RefCell<OrchestrationState>>,
        event_index: Option<usize>,
    ) -> Self {
        assert!(
            (result.is_none() && event_index.is_none())
                || (result.is_some() && event_index.is_some())
        );

        ActionFuture {
            result,
            state,
            event_index,
            is_inner: false,
        }
    }
}

impl<T> Future for ActionFuture<T>
where
    T: Unpin,
{
    type Output = T;

    fn poll(mut self: Pin<&mut Self>, _context: &mut Context) -> Poll<T> {
        if let Some(v) = self.result.take() {
            if !self.is_inner {
                self.state.borrow_mut().update(self.event_index.unwrap());
            }
            return Poll::Ready(v);
        }

        Poll::Pending
    }
}

impl<T> OrchestrationFuture for ActionFuture<T>
where
    T: Unpin,
{
    fn notify_inner(&mut self) {
        self.is_inner = true;
    }

    fn event_index(&self) -> Option<usize> {
        self.event_index
    }
}

/// Future for the `DurableOrchestrationContext::join_all` function.
#[must_use = "futures do nothing unless you `.await` or poll them"]
pub struct JoinAll<F>
where
    F: OrchestrationFuture,
{
    inner: futures::future::JoinAll<F>,
    state: Rc<RefCell<OrchestrationState>>,
    event_index: Option<usize>,
    is_inner: bool,
}

impl<F> JoinAll<F>
where
    F: OrchestrationFuture,
{
    fn new<T>(state: Rc<RefCell<OrchestrationState>>, iter: T) -> Self
    where
        T: IntoIterator<Item = F>,
        F: OrchestrationFuture,
    {
        let inner: Vec<_> = iter
            .into_iter()
            .map(|mut f| {
                f.notify_inner();
                f
            })
            .collect();

        // The event index of a join is the maximum of the sequence, provided all inner futures have event indexes
        let event_index = inner
            .iter()
            .try_fold(None, |i, f| {
                let next = f.event_index();
                if next.is_none() {
                    Err(())
                } else if i < next {
                    Ok(next)
                } else {
                    Ok(i)
                }
            })
            .unwrap_or(None);

        JoinAll {
            inner: join_all(inner),
            state,
            event_index,
            is_inner: false,
        }
    }
}

impl<F> Future for JoinAll<F>
where
    F: OrchestrationFuture,
{
    type Output = Vec<F::Output>;

    fn poll(mut self: Pin<&mut Self>, context: &mut Context) -> Poll<Self::Output> {
        let result = self.inner.poll_unpin(context);

        if !self.is_inner {
            if let Poll::Ready(_) = &result {
                self.state.borrow_mut().update(self.event_index.unwrap());
            }
        }

        result
    }
}

impl<F> OrchestrationFuture for JoinAll<F>
where
    F: OrchestrationFuture,
{
    fn notify_inner(&mut self) {
        self.is_inner = true;
    }

    fn event_index(&self) -> Option<usize> {
        self.event_index
    }
}

/// Future for the `DurableOrchestrationContext::select_all` function.
#[must_use = "futures do nothing unless you `.await` or poll them"]
pub struct SelectAll<F> {
    inner: Vec<F>,
    state: Rc<RefCell<OrchestrationState>>,
    event_index: Option<usize>,
    is_inner: bool,
}

impl<F> SelectAll<F>
where
    F: OrchestrationFuture,
{
    fn new<T>(state: Rc<RefCell<OrchestrationState>>, iter: T) -> Self
    where
        T: IntoIterator<Item = F>,
        F: OrchestrationFuture,
    {
        let inner: Vec<_> = iter
            .into_iter()
            .map(|mut f| {
                f.notify_inner();
                f
            })
            .collect();

        assert!(!inner.is_empty());

        // The event index of a select is the minimum present index of the sequence
        let event_index = inner.iter().filter_map(|f| f.event_index()).min();

        SelectAll {
            inner,
            state,
            event_index,
            is_inner: false,
        }
    }
}

impl<F> Unpin for SelectAll<F> where F: Unpin {}

impl<F> Future for SelectAll<F>
where
    F: OrchestrationFuture + Unpin,
{
    type Output = (F::Output, usize, Vec<F>);

    fn poll(mut self: Pin<&mut Self>, context: &mut Context) -> Poll<Self::Output> {
        let event_index = self.event_index;
        if event_index.is_none() {
            return Poll::Pending;
        }

        let item = self.inner.iter_mut().enumerate().find_map(|(i, f)| {
            if f.event_index() != event_index {
                return None;
            }
            match f.poll_unpin(context) {
                Poll::Pending => None,
                Poll::Ready(e) => Some((i, e)),
            }
        });

        match item {
            Some((idx, res)) => {
                self.inner.remove(idx);
                let rest = std::mem::replace(&mut self.inner, Vec::new());

                if !self.is_inner {
                    self.state.borrow_mut().update(event_index.unwrap());
                }

                Poll::Ready((res, idx, rest))
            }
            None => Poll::Pending,
        }
    }
}

impl<F> OrchestrationFuture for SelectAll<F>
where
    F: OrchestrationFuture + Unpin,
{
    fn notify_inner(&mut self) {
        self.is_inner = true;
    }

    fn event_index(&self) -> Option<usize> {
        self.event_index
    }
}

struct OrchestrationState {
    history: Vec<HistoryEvent>,
    result: Rc<RefCell<ExecutionResult>>,
    started_index: usize,
    completed_index: Option<usize>,
}

impl OrchestrationState {
    fn new(history: Vec<HistoryEvent>, result: Rc<RefCell<ExecutionResult>>) -> Self {
        let started_index = history
            .iter()
            .position(|event| event.event_type == EventType::OrchestratorStarted)
            .expect("failed to find orchestration started event");

        let completed_index = history[started_index..]
            .iter()
            .position(|event| event.event_type == EventType::OrchestratorCompleted)
            .map(|pos| pos + started_index);

        OrchestrationState {
            history,
            result,
            started_index,
            completed_index,
        }
    }

    fn is_replaying(&self) -> bool {
        self.completed_index.is_some()
    }

    fn current_time(&self) -> DateTime<Utc> {
        self.history[self.started_index].timestamp
    }

    fn execution_result(&self) -> Rc<RefCell<ExecutionResult>> {
        self.result.clone()
    }

    fn find_scheduled_activity(
        &mut self,
        activity_name: &str,
    ) -> Option<(usize, &mut HistoryEvent)> {
        let index = self.history.iter().position(|event| {
            event.name == Some(activity_name.to_owned())
                && event.event_type == EventType::TaskScheduled
                && !event.is_processed
        })?;

        Some((index, &mut self.history[index]))
    }

    fn find_completed_activity(
        &mut self,
        scheduled_index: usize,
    ) -> Option<(usize, &mut HistoryEvent)> {
        if scheduled_index + 1 >= self.history.len() {
            return None;
        }

        let id = self.history[scheduled_index].event_id;

        let index = self.history[scheduled_index + 1..]
            .iter()
            .position(|event| {
                event.event_type == EventType::TaskCompleted && event.task_scheduled_id == Some(id)
            })
            .map(|p| p + scheduled_index + 1)?;

        Some((index, &mut self.history[index]))
    }

    fn find_failed_activity(
        &mut self,
        scheduled_index: usize,
    ) -> Option<(usize, &mut HistoryEvent)> {
        if scheduled_index + 1 >= self.history.len() {
            return None;
        }

        let id = self.history[scheduled_index].event_id;

        let index = self.history[scheduled_index + 1..]
            .iter()
            .position(|event| {
                event.event_type == EventType::TaskFailed && event.task_scheduled_id == Some(id)
            })
            .map(|p| p + scheduled_index + 1)?;

        Some((index, &mut self.history[index]))
    }

    fn update(&mut self, event_index: usize) {
        // Check for end of history
        if self.started_index + 1 >= self.history.len() || self.completed_index.is_none() {
            return;
        }

        while self.completed_index.unwrap() < event_index {
            let started_index = self.history[self.started_index + 1..]
                .iter()
                .position(|event| event.event_type == EventType::OrchestratorStarted)
                .map(|pos| pos + self.started_index + 1);

            if started_index.is_none() {
                return;
            }

            self.started_index = started_index.unwrap();
            self.completed_index = self.history[self.started_index..]
                .iter()
                .position(|event| event.event_type == EventType::OrchestratorCompleted)
                .map(|pos| pos + self.started_index);

            self.result.borrow_mut().notify_new_execution();

            if self.completed_index.is_none() {
                return;
            }
        }
    }
}

/// Represents the Durable Functions orchestration context binding.
///
/// The following binding attributes are supported:
///
/// | Name            | Description                                                           |
/// |-----------------|-----------------------------------------------------------------------|
/// | `name`          | The name of the parameter being bound.                                |
/// | `orchestration` | The name of the orchestration.  Defaults to the name of the function. |
///
/// # Examples
///
/// TODO: IMPLEMENT
pub struct DurableOrchestrationContext {
    /// The orchestration instance identifier.
    pub instance_id: String,
    /// The parent orchestration instance identifier.
    pub parent_instance_id: Option<String>,
    /// The input value to the orchestration.
    pub input: Value,
    state: Rc<RefCell<OrchestrationState>>,
}

impl DurableOrchestrationContext {
    #[doc(hidden)]
    pub fn new(data: TypedData, _metadata: HashMap<String, TypedData>) -> Self {
        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct BindingData {
            instance_id: String,
            parent_instance_id: Option<String>,
            input: Value,
            history: Vec<HistoryEvent>,
        }

        match &data.data {
            Some(Data::String(s)) => {
                let data: BindingData =
                    from_str(s).expect("failed to parse orchestration context data");

                DurableOrchestrationContext {
                    instance_id: data.instance_id,
                    parent_instance_id: data.parent_instance_id,
                    input: data.input,
                    state: Rc::new(RefCell::new(OrchestrationState::new(
                        data.history,
                        Rc::new(RefCell::new(ExecutionResult::default())),
                    ))),
                }
            }
            _ => panic!("expected JSON data for orchestration context data"),
        }
    }

    /// Gets a value indicating whether the orchestrator function is currently replaying itself.
    pub fn is_replaying(&self) -> bool {
        self.state.borrow().is_replaying()
    }

    /// Gets the current date/time in a way that is safe for use by orchestrator functions.
    pub fn current_time(&self) -> DateTime<Utc> {
        self.state.borrow().current_time()
    }

    #[doc(hidden)]
    pub fn execution_result(&self) -> Rc<RefCell<ExecutionResult>> {
        self.state.borrow().execution_result()
    }

    /// Creates a future which represents a collection of the outputs of the futures given.
    ///
    /// The returned future will drive execution for all of its underlying futures,
    /// collecting the results into a destination `Vec<T>` in the same order as they
    /// were provided.
    #[must_use = "futures do nothing unless you `.await` or poll them"]
    pub fn join_all<I>(&self, iter: I) -> JoinAll<I::Item>
    where
        I: IntoIterator,
        I::Item: OrchestrationFuture,
    {
        JoinAll::new(self.state.clone(), iter)
    }

    /// Creates a new future which will select over a list of futures.
    ///
    /// The returned future will wait for any future within `iter` to be ready. Upon
    /// completion the item resolved will be returned, along with the index of the
    /// future that was ready and the list of all the remaining futures.
    ///
    /// # Panics
    ///
    /// This function will panic if the iterator specified contains no items.
    pub fn select_all<I>(&self, iter: I) -> SelectAll<I::Item>
    where
        I: IntoIterator,
        I::Item: OrchestrationFuture,
    {
        SelectAll::new(self.state.clone(), iter)
    }

    /// Schedules an activity function for execution.
    #[must_use = "futures do nothing unless you `.await` or poll them"]
    pub fn call_activity<D>(
        &self,
        activity_name: &str,
        data: D,
    ) -> impl Future<Output = Result<Value, String>> + OrchestrationFuture
    where
        D: Into<Value>,
    {
        let mut state = self.state.borrow_mut();

        // Push the action on the execution result
        state.result.borrow_mut().push_action(Action::CallActivity {
            function_name: activity_name.to_string(),
            input: data.into(),
        });

        let mut result: Option<Result<Value, String>> = None;
        let mut event_index = None;

        // Attempt to resolve the activity
        if let Some((idx, scheduled)) = state.find_scheduled_activity(activity_name) {
            scheduled.is_processed = true;

            if let Some((idx, completed)) = state.find_completed_activity(idx) {
                completed.is_processed = true;
                event_index = Some(idx);
                result = Some(Ok(completed
                    .result
                    .as_ref()
                    .map(|s| from_str(&s).unwrap_or_default())
                    .unwrap_or(Value::Null)));
            } else if let Some((idx, failed)) = state.find_failed_activity(idx) {
                failed.is_processed = true;
                event_index = Some(idx);
                result = Some(Err(failed.reason.clone().unwrap_or_default()));
            }
        }

        ActionFuture::new(result, self.state.clone(), event_index)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::durable::{EventType, HistoryEvent};
    use crate::rpc::typed_data::Data;
    use chrono::DateTime;

    #[test]
    #[should_panic(expected = "expected JSON data for orchestration context data")]
    fn new_panics_if_no_data_provided() {
        let data = TypedData { data: None };

        let _ = DurableOrchestrationContext::new(data, HashMap::new());
    }

    #[test]
    #[should_panic(expected = "failed to parse orchestration context data")]
    fn new_panics_if_no_json_provided() {
        let data = TypedData {
            data: Some(Data::String(r#"{ }"#.to_owned())),
        };

        let _ = DurableOrchestrationContext::new(data, HashMap::new());
    }

    #[test]
    #[should_panic(expected = "failed to find orchestration started event")]
    fn it_panics_if_missing_history() {
        let data = TypedData {
            data: Some(Data::String(
                r#"{
                "instanceId":"49497890673e4a75ab380e7a956c607b",
                "isReplaying":false,
                "parentInstanceId":"1234123412341234123412341234",
                "input": [],
                "history": []
            }"#
                .to_owned(),
            )),
        };

        DurableOrchestrationContext::new(data, HashMap::new());
    }

    #[test]
    fn new_constructs_an_orchestration_context_with_history() {
        let data = TypedData {
            data: Some(Data::String(
                r#"{
                "history":[
                    {
                       "EventType":12,
                       "EventId":-1,
                       "IsPlayed":true,
                       "Timestamp":"2019-07-18T06:22:27.016757Z"
                    },
                    {
                        "OrchestrationInstance":{
                           "InstanceId":"49497890673e4a75ab380e7a956c607b",
                           "ExecutionId":"5d2025984bef476bbaacefaa499a4f5f"
                        },
                        "EventType":0,
                        "ParentInstance":null,
                        "Name":"HelloWorld",
                        "Version":"",
                        "Input":"{}",
                        "Tags":null,
                        "EventId":-1,
                        "IsPlayed":false,
                       "Timestamp":"2019-07-18T06:22:26.626966Z"
                    }
                ],
                "instanceId":"49497890673e4a75ab380e7a956c607b",
                "isReplaying":false,
                "parentInstanceId":null,
                "input": []
            }"#
                .to_owned(),
            )),
        };

        let context = DurableOrchestrationContext::new(data, HashMap::new());
        assert_eq!(context.instance_id, "49497890673e4a75ab380e7a956c607b");
        assert_eq!(context.parent_instance_id, None);
        assert!(!context.is_replaying());
        assert_eq!(context.input, serde_json::Value::Array(vec![]));
        assert_eq!(
            context.state.borrow().history,
            vec![
                HistoryEvent {
                    event_type: EventType::OrchestratorStarted,
                    event_id: -1,
                    is_played: true,
                    timestamp: DateTime::<Utc>::from(
                        DateTime::parse_from_rfc3339("2019-07-18T06:22:27.016757Z").unwrap()
                    ),
                    is_processed: false,
                    name: None,
                    input: None,
                    result: None,
                    task_scheduled_id: None,
                    instance_id: None,
                    reason: None,
                    details: None,
                    fire_at: None,
                    timer_id: None,
                },
                HistoryEvent {
                    event_type: EventType::ExecutionStarted,
                    event_id: -1,
                    is_played: false,
                    timestamp: DateTime::<Utc>::from(
                        DateTime::parse_from_rfc3339("2019-07-18T06:22:26.626966Z").unwrap()
                    ),
                    is_processed: false,
                    name: Some("HelloWorld".to_owned()),
                    input: Some("{}".into()),
                    result: None,
                    task_scheduled_id: None,
                    instance_id: None,
                    reason: None,
                    details: None,
                    fire_at: None,
                    timer_id: None,
                }
            ]
        );
    }
}
