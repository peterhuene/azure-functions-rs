use crate::durable::{Action, EventType, HistoryEvent};
use crate::{
    durable::ExecutionResult,
    rpc::{typed_data::Data, TypedData},
};
use chrono::{DateTime, Utc};
use futures::future::join_all;
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
}

struct ActionFuture<T> {
    result: Option<T>,
    state: Rc<RefCell<OrchestrationState>>,
    is_inner: bool,
}

impl<T> ActionFuture<T> {
    fn new(result: Option<T>, state: Rc<RefCell<OrchestrationState>>) -> Self {
        ActionFuture {
            result,
            state,
            is_inner: false,
        }
    }
}

impl<T> Future for ActionFuture<T>
where
    T: Unpin,
{
    type Output = T;

    fn poll(self: Pin<&mut Self>, _context: &mut Context) -> Poll<T> {
        let this = self.get_mut();
        if let Some(v) = this.result.take() {
            if !this.is_inner {
                this.state.borrow_mut().update();
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
}

/// Future for the `DurableOrchestrationContext::join_all` function.
#[must_use = "futures do nothing unless you `.await` or poll them"]
pub struct JoinAll<F>
where
    F: OrchestrationFuture,
{
    inner: futures::future::JoinAll<F>,
    state: Rc<RefCell<OrchestrationState>>,
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
        let futs: Vec<_> = iter
            .into_iter()
            .map(|mut f| {
                f.notify_inner();
                f
            })
            .collect();

        JoinAll {
            inner: join_all(futs),
            state,
            is_inner: false,
        }
    }
}

impl<F> Future for JoinAll<F>
where
    F: OrchestrationFuture,
{
    type Output = <futures::future::JoinAll<F> as Future>::Output;

    fn poll(mut self: Pin<&mut Self>, context: &mut Context) -> Poll<Self::Output> {
        let result = Future::poll(Pin::new(&mut self.inner), context);

        if !self.is_inner {
            if let Poll::Ready(_) = &result {
                self.get_mut().state.borrow_mut().update();
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
}

struct OrchestrationState {
    is_replaying: bool,
    history: Vec<HistoryEvent>,
    result: Rc<RefCell<ExecutionResult>>,
    orchestration_index: usize,
}

impl OrchestrationState {
    fn find_scheduled_activity(&mut self, activity_name: &str) -> Option<&mut HistoryEvent> {
        self.history.iter_mut().find(|event| {
            event.name == Some(activity_name.to_owned())
                && event.event_type == EventType::TaskScheduled
                && !event.is_processed
        })
    }

    fn find_completed_activity(&mut self, event_id: i32) -> Option<&mut HistoryEvent> {
        self.history.iter_mut().find(|event| {
            event.event_type == EventType::TaskCompleted
                && event.task_scheduled_id == Some(event_id)
        })
    }

    fn find_failed_activity(&mut self, event_id: i32) -> Option<&mut HistoryEvent> {
        self.history.iter_mut().find(|event| {
            event.event_type == EventType::TaskFailed && event.task_scheduled_id == Some(event_id)
        })
    }

    fn update(&mut self) {
        fn is_newer_event(event: &HistoryEvent, timestamp: DateTime<Utc>) -> bool {
            event.event_type == EventType::OrchestratorStarted && event.timestamp > timestamp
        }

        let start = self.orchestration_index + 1;
        if start >= self.history.len() {
            return;
        }

        let current_timestamp = self.history[self.orchestration_index].timestamp;

        match self.history[start..]
            .iter()
            .position(|event| is_newer_event(event, current_timestamp))
            .map(|pos| pos + start)
        {
            Some(next) => {
                // A new orchestration execution has occurred
                self.orchestration_index = next;
                self.result.borrow_mut().notify_new_execution();

                let current_timestamp = self.history[self.orchestration_index].timestamp;
                let start = self.orchestration_index + 1;

                // Check to see if this is the last execution; if so, no more replaying
                if start >= self.history.len()
                    || !self.history[start..]
                        .iter()
                        .any(|event| is_newer_event(event, current_timestamp))
                {
                    self.is_replaying = false;
                }
            }
            None => {
                self.is_replaying = false;
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
            is_replaying: bool,
            parent_instance_id: Option<String>,
            input: Value,
            history: Vec<HistoryEvent>,
        }

        match &data.data {
            Some(Data::String(s)) => {
                let data: BindingData =
                    from_str(s).expect("failed to parse orchestration context data");

                let orchestration_index = data
                    .history
                    .iter()
                    .position(|event| event.event_type == EventType::OrchestratorStarted)
                    .expect("failed to find orchestration started event");

                DurableOrchestrationContext {
                    instance_id: data.instance_id,
                    parent_instance_id: data.parent_instance_id,
                    input: data.input,
                    state: Rc::new(RefCell::new(OrchestrationState {
                        orchestration_index,
                        is_replaying: data.is_replaying,
                        history: data.history,
                        result: Rc::new(RefCell::new(ExecutionResult::default())),
                    })),
                }
            }
            _ => panic!("expected JSON data for orchestration context data"),
        }
    }

    /// Gets a value indicating whether the orchestrator function is currently replaying itself.
    pub fn is_replaying(&self) -> bool {
        self.state.borrow().is_replaying
    }

    /// Gets the current date/time in a way that is safe for use by orchestrator functions.
    pub fn current_time(&self) -> DateTime<Utc> {
        let state = self.state.borrow();
        state.history[state.orchestration_index].timestamp
    }

    #[doc(hidden)]
    pub fn execution_result(&self) -> Rc<RefCell<ExecutionResult>> {
        self.state.borrow().result.clone()
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

    /// Schedules an activity function for execution.
    #[must_use = "futures do nothing unless you `.await` or poll them"]
    pub fn call_activity<D>(
        &mut self,
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

        // Attempt to resolve the activity
        if let Some(scheduled) = state.find_scheduled_activity(activity_name) {
            scheduled.is_processed = true;

            let id = scheduled.event_id;
            if let Some(completed) = state.find_completed_activity(id) {
                completed.is_processed = true;
                result = Some(Ok(completed
                    .result
                    .as_ref()
                    .map(|s| from_str(&s).unwrap_or_default())
                    .unwrap_or(Value::Null)));
            } else if let Some(failed) = state.find_failed_activity(id) {
                failed.is_processed = true;
                result = Some(Err(failed.reason.clone().unwrap_or_default()));
            }
        }

        ActionFuture::new(result, self.state.clone())
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
