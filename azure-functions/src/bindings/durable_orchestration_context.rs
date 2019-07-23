use serde::Deserialize;
use serde_json::{from_str, Value};
use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::durable::{Action, EventType, HistoryEvent};
use crate::{
    durable::ExecutionResult,
    rpc::{typed_data::Data, TypedData},
};
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

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
#[derive(Debug)]
pub struct DurableOrchestrationContext {
    data: DurableOrchestrationContextData,
    result: Rc<RefCell<ExecutionResult>>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DurableOrchestrationContextData {
    instance_id: String,
    is_replaying: bool,
    parent_instance_id: Option<String>,
    input: Value,
    history: Vec<HistoryEvent>,
}

impl DurableOrchestrationContext {
    #[doc(hidden)]
    pub fn new(data: TypedData, _metadata: HashMap<String, TypedData>) -> Self {
        match &data.data {
            Some(Data::String(s)) => DurableOrchestrationContext {
                data: from_str(s).expect("failed to parse orchestration context data"),
                result: Rc::new(RefCell::new(ExecutionResult::default())),
            },
            _ => panic!("expected JSON data for orchestration context data"),
        }
    }

    /// Gets the instance ID of the currently executing orchestration.
    pub fn instance_id(&self) -> &str {
        &self.data.instance_id
    }

    /// Gets the parent instance ID of the currently executing sub-orchestration.
    pub fn parent_instance_id(&self) -> Option<&str> {
        self.data.parent_instance_id.as_ref().map(|id| &**id)
    }

    /// Gets a value indicating whether the orchestrator function is currently replaying itself.
    pub fn is_replaying(&self) -> bool {
        self.data.is_replaying
    }

    /// The JSON-serializeable input to pass to the orchestrator function.
    pub fn input(&self) -> &Value {
        &self.data.input
    }

    #[doc(hidden)]
    pub fn execution_result(&self) -> Rc<RefCell<ExecutionResult>> {
        self.result.clone()
    }

    #[doc(hidden)]
    pub fn call_activity<D>(
        &mut self,
        activity_name: &str,
        data: D,
    ) -> impl Future<Output = Result<Value, String>>
    where
        D: Into<Value>,
    {
        // TODO Assume each of these methods mark found entries as processed
        // TODO They will also be responsible for moving the "current" timestamp forward as they process the execution history
        let scheduled = self.find_scheduled_activity(activity_name);
        let completed = scheduled.and_then(|scheduled| self.find_completed_activity(scheduled));
        let failed = scheduled.and_then(|scheduled| self.find_failed_activity(scheduled));

        self.result.borrow_mut().add_action(Action::CallActivity {
            function_name: activity_name.to_string(),
            input: data.into(),
        });

        if let Some(completed) = completed {
            return ActionFuture(Some(Ok(self.parse_history_event(completed))));
        }

        if let Some(failed) = failed {
            return ActionFuture(Some(Err(failed.reason.clone().unwrap_or_default())));
        }

        return ActionFuture(None);
    }

    fn find_scheduled_activity(&self, activity_name: &str) -> Option<&HistoryEvent> {
        self.data.history.iter().find(|event| {
            event.name == Some(activity_name.to_owned())
                && event.event_type == EventType::TaskScheduled
                && !event.is_processed
        })
    }

    fn find_completed_activity(&self, scheduled: &HistoryEvent) -> Option<&HistoryEvent> {
        self.data.history.iter().find(|event| {
            event.event_type == EventType::TaskCompleted
                && event.task_scheduled_id == Some(scheduled.event_id)
        })
    }

    fn find_failed_activity(&self, scheduled: &HistoryEvent) -> Option<&HistoryEvent> {
        self.data.history.iter().find(|event| {
            event.event_type == EventType::TaskFailed
                && event.task_scheduled_id == Some(scheduled.event_id)
        })
    }

    fn parse_history_event(&self, event: &HistoryEvent) -> Value {
        match event.event_type {
            EventType::EventRaised => event.input.clone(),
            EventType::SubOrchestrationInstanceCompleted | EventType::TaskCompleted => {
                event.result.clone()
            }
            _ => None,
        }
        .unwrap_or_default()
    }
}

struct ActionFuture<T>(Option<T>);

impl<T> Future for ActionFuture<T>
where
    T: Unpin,
{
    type Output = T;

    fn poll(self: Pin<&mut Self>, _context: &mut Context) -> Poll<T> {
        if let Some(v) = self.get_mut().0.take() {
            return Poll::Ready(v);
        }

        Poll::Pending
    }
}

/*
{
   "history":[
      {
         "EventType":12,
         "EventId":-1,
         "IsPlayed":false,
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
   "input":{

   },
   "instanceId":"49497890673e4a75ab380e7a956c607b",
   "isReplaying":false,
   "parentInstanceId":null
}
*/

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
    fn new_constructs_an_orchestration_context_without_history() {
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

        let context = DurableOrchestrationContext::new(data, HashMap::new());
        assert_eq!(context.instance_id(), "49497890673e4a75ab380e7a956c607b");
        assert_eq!(
            context.parent_instance_id(),
            Some("1234123412341234123412341234")
        );
        assert!(!context.is_replaying());
        assert_eq!(context.data.history, vec![]);
        assert_eq!(context.input(), &serde_json::Value::Array(vec![]));
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
                       "IsPlayed":false,
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
        assert_eq!(context.instance_id(), "49497890673e4a75ab380e7a956c607b");
        assert_eq!(context.parent_instance_id(), None);
        assert!(!context.is_replaying());
        assert_eq!(context.input(), &serde_json::Value::Array(vec![]));
        assert_eq!(
            context.data.history,
            vec![
                HistoryEvent {
                    event_type: EventType::OrchestratorStarted,
                    event_id: -1,
                    is_played: false,
                    timestamp: DateTime::parse_from_rfc3339("2019-07-18T06:22:27.016757Z").unwrap(),
                    is_processed: false,
                    name: None,
                    input: None,
                    result: None,
                    task_scheduled_id: None,
                    instance_id: None,
                    reason: None,
                    details: None,
                    fire_at: None,
                    timer_id: None
                },
                HistoryEvent {
                    event_type: EventType::ExecutionStarted,
                    event_id: -1,
                    is_played: false,
                    timestamp: DateTime::parse_from_rfc3339("2019-07-18T06:22:26.626966Z").unwrap(),
                    is_processed: false,
                    name: Some("HelloWorld".to_owned()),
                    input: Some("{}".into()),
                    result: None,
                    task_scheduled_id: None,
                    instance_id: None,
                    reason: None,
                    details: None,
                    fire_at: None,
                    timer_id: None
                }
            ]
        );
    }
}
