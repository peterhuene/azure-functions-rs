//! # Durable Functions for Rust
#![feature(async_await)]

use chrono::prelude::*;
use serde::{Deserialize};
use serde_json::Value;

#[derive(Debug, Clone, Deserialize, PartialEq)]
enum InstanceRuntimeState {
    Running,
    Pending,
    Failed,
    Canceled,
    Terminated,
    Completed
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct InstanceHistoryEvent {
    event_type: String,
    orchestration_status:Option<InstanceRuntimeState>,
    function_name: Option<String>,
    result: Option<Value>,
    scheduled_time: Option<DateTime<FixedOffset>>,
    timestamp: DateTime<FixedOffset>
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstanceStatus {
    runtime_status:InstanceRuntimeState,
    input: Option<Value>,
    custom_status: Option<Value>,
    output: Option<Value>,
    created_time: DateTime<FixedOffset>,
    history_events: Option<Vec<InstanceHistoryEvent>>

}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_instance_history() {
        let h1:String = r#"{"EventType": "ExecutionStarted",
          "FunctionName": "E1_HelloSequence",
          "Timestamp": "2018-02-28T05:18:49Z"
        }"#.to_owned();

        let compare_dt = Utc.ymd(2018, 2, 28).and_hms(5, 18, 49);

        let h1_obj:InstanceHistoryEvent = serde_json::from_str(&h1).unwrap();
        assert_eq!(h1_obj.event_type, "ExecutionStarted");
        assert_eq!(h1_obj.timestamp, compare_dt);

        let h2:String = r#"{
          "EventType": "ExecutionCompleted",
          "OrchestrationStatus": "Completed",
          "Result": [
              "Hello Tokyo!",
              "Hello Seattle!",
              "Hello London!"
          ],
          "Timestamp": "2018-02-28T05:18:54.3660895Z"
        }"#.to_owned();

        let h2_obj:InstanceHistoryEvent = serde_json::from_str(&h2).unwrap();
        assert_eq!(h2_obj.orchestration_status.is_some(), true);
        assert_eq!(h2_obj.orchestration_status.unwrap(), InstanceRuntimeState::Completed);
    }

    #[test]
    fn test_instance_status() {
        let example:String = r#"{
            "createdTime": "2018-02-28T05:18:49Z",
            "historyEvents": [
            {
                "EventType": "ExecutionStarted",
                "FunctionName": "E1_HelloSequence",
                "Timestamp": "2018-02-28T05:18:49.3452372Z"
            },
            {
                "EventType": "TaskCompleted",
                "FunctionName": "E1_SayHello",
                "Result": "Hello Tokyo!",
                "ScheduledTime": "2018-02-28T05:18:51.3939873Z",
                "Timestamp": "2018-02-28T05:18:52.2895622Z"
            },
            {
                "EventType": "TaskCompleted",
                "FunctionName": "E1_SayHello",
                "Result": "Hello Seattle!",
                "ScheduledTime": "2018-02-28T05:18:52.8755705Z",
                "Timestamp": "2018-02-28T05:18:53.1765771Z"
            },
            {
                "EventType": "TaskCompleted",
                "FunctionName": "E1_SayHello",
                "Result": "Hello London!",
                "ScheduledTime": "2018-02-28T05:18:53.5170791Z",
                "Timestamp": "2018-02-28T05:18:53.891081Z"
            },
            {
                "EventType": "ExecutionCompleted",
                "OrchestrationStatus": "Completed",
                "Result": [
                    "Hello Tokyo!",
                    "Hello Seattle!",
                    "Hello London!"
                ],
                "Timestamp": "2018-02-28T05:18:54.3660895Z"
            }
        ],
        "input": null,
        "customStatus": { "nextActions": ["A", "B", "C"], "foo": 2 },
        "lastUpdatedTime": "2018-02-28T05:18:54Z",
        "output": [
            "Hello Tokyo!",
            "Hello Seattle!",
            "Hello London!"
        ],
        "runtimeStatus": "Completed"
        }"#.to_owned();
    
        let instance_status:InstanceStatus = serde_json::from_str(&example).unwrap();
        assert_eq!(instance_status.history_events.is_some(), true);
        assert_eq!(instance_status.history_events.unwrap().len(), 5);

        assert_eq!(instance_status.custom_status.is_some(), true);
        assert_eq!(instance_status.custom_status.unwrap().is_object(), true);
    }

}