use crate::{
    durable::{
        Action, ActionFuture, EventType, HistoryEvent, JoinAll, OrchestrationFuture,
        OrchestrationState, RetryOptions, SelectAll,
    },
    rpc::{typed_data::Data, TypedData},
};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use serde_json::{from_str, Value};
use std::{cell::RefCell, collections::HashMap, rc::Rc};

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
                    state: Rc::new(RefCell::new(OrchestrationState::new(data.history))),
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
    pub fn state(&self) -> Rc<RefCell<OrchestrationState>> {
        self.state.clone()
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
    ) -> ActionFuture<Result<Value, String>>
    where
        D: Into<Value>,
    {
        self.call_activity_impl(
            Action::CallActivity {
                function_name: activity_name.to_string(),
                input: data.into(),
            },
            activity_name,
        )
    }

    /// Schedules an activity function for execution with retry options.
    #[must_use = "futures do nothing unless you `.await` or poll them"]
    pub fn call_activity_with_retry<D>(
        &self,
        activity_name: &str,
        data: D,
        retry_options: RetryOptions,
    ) -> ActionFuture<Result<Value, String>>
    where
        D: Into<Value>,
    {
        self.call_activity_impl(
            Action::CallActivityWithRetry {
                function_name: activity_name.to_string(),
                retry_options,
                input: data.into(),
            },
            activity_name,
        )
    }

    fn call_activity_impl(
        &self,
        action: Action,
        activity_name: &str,
    ) -> ActionFuture<Result<Value, String>> {
        let mut state = self.state.borrow_mut();

        state.push_action(action);

        let mut result: Option<Result<Value, String>> = None;
        let mut event_index = None;

        // Attempt to resolve the activity
        if let Some((idx, scheduled)) = state.find_scheduled_task(activity_name) {
            scheduled.is_processed = true;

            if let Some((idx, finished)) = state.find_finished_task(idx) {
                finished.is_processed = true;
                event_index = Some(idx);

                result = Some(match finished.event_type {
                    EventType::TaskCompleted => Ok(finished
                        .result
                        .as_ref()
                        .map(|s| from_str(&s).unwrap_or_default())
                        .unwrap_or(Value::Null)),
                    EventType::TaskFailed => Err(finished.reason.clone().unwrap_or_default()),
                    _ => panic!("task must either complete or fail"),
                });
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
