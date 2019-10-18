//! # Durable Functions for Rust
#![feature(async_await)]

#[macro_use]
extern crate derive_builder;

use std::result::Result::*;
use chrono::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Represents the Durable Functions client creation URLs.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreationUrls {
    /// The URL for creating a new orchestration instance.
    #[serde(rename = "createNewInstancePostUri")]
    pub create_new_instance_url: String,
    /// The URL for creating and waiting on a new orchestration instance.
    #[serde(rename = "createAndWaitOnNewInstancePostUri")]
    pub create_new_instance_and_wait_url: String,
}

/// Represents the Durable Functions client management URLs.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ManagementUrls {
    /// The ID of the orchestration instance.
    pub id: String,
    /// The status URL of the orchestration instance.
    #[serde(rename = "statusQueryGetUri")]
    pub status_query_url: String,
    /// The "raise event" URL of the orchestration instance.
    #[serde(rename = "sendEventPostUri")]
    pub raise_event_url: String,
    /// The "terminate" URL of the orchestration instance.
    #[serde(rename = "terminatePostUri")]
    pub terminate_url: String,
    /// The "rewind" URL of the orchestration instance.
    #[serde(rename = "rewindPostUri")]
    pub rewind_url: String,
    /// The "purge history" URL of the orchestration instance.
    #[serde(rename = "purgeHistoryDeleteUri")]
    pub purge_history_url: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DurableOrchestrationClientBinding {
    #[serde(rename = "taskHubName")]
    task_hub: String,
    creation_urls: CreationUrls,
    management_urls: ManagementUrls,
}

impl DurableOrchestrationClientBinding {
    // Mostly for debugging purposes
    pub fn new(hub:String, create_new:String, create_and_wait:String, id:String, status_query:String, raise:String, terminate:String, rewind:String, purge:String) -> DurableOrchestrationClientBinding {
        DurableOrchestrationClientBinding {
            task_hub:hub,
            creation_urls: CreationUrls { 
                create_new_instance_url: create_new, 
                create_new_instance_and_wait_url: create_and_wait 
            },
            management_urls: ManagementUrls {
                id,
                status_query_url:status_query,
                raise_event_url:raise,
                terminate_url:terminate,
                rewind_url:rewind,
                purge_history_url:purge
            }
        }
    }
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub enum OrchestrationRuntimeStatus {
    Running,
    Pending,
    Failed,
    Canceled,
    Terminated,
    Completed
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub enum OrchestrationClientError
{
    //400
    InstanceFailedOrTerminated,
    //410
    InstanceCompletedOrFailed,
    //404
    InstanceNotFound,
    //400
    BadRaiseEventContent,
    //500
    UnspecifiedError
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct OrchestrationHistoryEvent {
    event_type: String,
    orchestration_status:Option<OrchestrationRuntimeStatus>,
    function_name: Option<String>,
    result: Option<Value>,
    scheduled_time: Option<DateTime<Utc>>,
    timestamp: DateTime<Utc>
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrchestrationStatus {
    runtime_status:OrchestrationRuntimeStatus,
    input: Option<Value>,
    custom_status: Option<Value>,
    output: Option<Value>,
    created_time: DateTime<Utc>,
    history_events: Option<Vec<OrchestrationHistoryEvent>>
}

#[derive(Default, Builder, Debug)]
#[builder(setter(strip_option))]
pub struct InstanceQuery<'a> {
    instance_id:Option<&'a str>,
    created_time_from:Option<DateTime<Utc>>, 
    created_time_to:Option<DateTime<Utc>>, 
    runtime_status:Option<Vec<OrchestrationRuntimeStatus>>,
    task_hub:Option<&'a str>,
    connection_name:Option<&'a str>,
    top:u32
}

impl<'a> InstanceQuery<'a> {
    pub fn with_id(id:&str) -> InstanceQuery {
        InstanceQueryBuilder::default().instance_id(id).build().unwrap()
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PurgeHistoryResult {
    instances_deleted:u32
}

type OrchestrationResult<T> = Result<T, OrchestrationClientError>;

pub struct OrchestrationClient {
    binding: DurableOrchestrationClientBinding
}

impl OrchestrationClient {
    pub fn task_hub(&self) -> &str {
        &self.binding.task_hub
    }

    pub async fn get_status<'a>(&self, query:InstanceQuery<'a>) -> OrchestrationResult<OrchestrationStatus> {
        Err(OrchestrationClientError::UnspecifiedError)
    }

    pub async fn get_statuses_by<'a>(&self, query:InstanceQuery<'a>)
        -> OrchestrationResult<Vec<OrchestrationStatus>>{
        let items = Vec::new();
        let error = true;

        if error {
            return Err(OrchestrationClientError::UnspecifiedError);
        }

        Ok(items)
    }

    pub async fn get_status_all<'a, D>(&self, query:Option<InstanceQuery<'a>>) -> OrchestrationResult<D>
        where D: IntoIterator<Item = OrchestrationStatus> {
        Err(OrchestrationClientError::UnspecifiedError)
    }

    pub async fn purge_instance_history<'a>(&self, query:InstanceQuery<'a>) -> OrchestrationResult<PurgeHistoryResult> {
        Err(OrchestrationClientError::UnspecifiedError)
    }

    pub async fn purge_history_by<'a>(&self, query:InstanceQuery<'a>)
        -> OrchestrationResult<PurgeHistoryResult> {
        Err(OrchestrationClientError::UnspecifiedError)
    }

    pub async fn raise_event<'a, D>(&self, event_name:&str, event_data:D, query:InstanceQuery<'a>) 
        -> OrchestrationResult<()>
        where D: Into<serde_json::Value> {
        Err(OrchestrationClientError::UnspecifiedError)
    }

    pub async fn rewind<'a>(&self, reason:&str, query:InstanceQuery<'a>) -> OrchestrationResult<()> {
        Err(OrchestrationClientError::UnspecifiedError)
    }

    pub async fn start_new<D>(&self, orchestrator_function_name:&str, instance_id:Option<&str>, input:Option<D>)
        -> OrchestrationResult<String>
        where D: Into<serde_json::Value> {
        Err(OrchestrationClientError::UnspecifiedError)
    }

    pub async fn terminate<'a>(&self, reason:&str, query:InstanceQuery<'a>) -> OrchestrationResult<()> {
        Err(OrchestrationClientError::UnspecifiedError)
    }
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

        let h1_obj:OrchestrationHistoryEvent = serde_json::from_str(&h1).unwrap();
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

        let h2_obj:OrchestrationHistoryEvent = serde_json::from_str(&h2).unwrap();
        assert_eq!(h2_obj.orchestration_status.is_some(), true);
        assert_eq!(h2_obj.orchestration_status.unwrap(), OrchestrationRuntimeStatus::Completed);
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
    
        let instance_status:OrchestrationStatus = serde_json::from_str(&example).unwrap();
        assert_eq!(instance_status.history_events.is_some(), true);
        assert_eq!(instance_status.history_events.unwrap().len(), 5);

        assert_eq!(instance_status.custom_status.is_some(), true);
        assert_eq!(instance_status.custom_status.unwrap().is_object(), true);
    }

    #[test]
    fn test_query_builder() {
        let query = InstanceQueryBuilder::default().created_time_from(Utc.ymd(2018, 2, 28).and_hms(5, 18, 49)).build().unwrap();
        assert_eq!(query.created_time_from.is_some(), true);
    }
}