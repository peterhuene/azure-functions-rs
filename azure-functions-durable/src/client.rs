use crate::endpoint::Endpoint;
use crate::error::ClientError;
use crate::Result;
use bytes::Buf;
use chrono::{DateTime, Utc};
use hyper::{self, Body, Request, StatusCode};
use serde::{Deserialize, Serialize};
use serde_json::{from_slice, to_string, Value};
use std::fmt::{Display, Formatter};
use url::Url;

/// Represents the runtime status of an orchestration.
#[derive(Debug, Clone, Deserialize, PartialEq)]
pub enum OrchestrationRuntimeStatus {
    /// The orchestration is running.
    Running,
    /// The orchestration is pending.
    Pending,
    /// The orchestration has failed.
    Failed,
    /// The orchestration was canceled.
    Canceled,
    /// The orchestration was terminated.
    Terminated,
    /// The orchestration completed successfully.
    Completed,
}

impl Display for OrchestrationRuntimeStatus {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

/// Represents an orchestration history event.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct OrchestrationHistoryEvent {
    /// The event type.
    pub event_type: String,
    /// The orchestration status for the event.
    pub orchestration_status: Option<OrchestrationRuntimeStatus>,
    /// The function name for the event.
    pub function_name: Option<String>,
    /// The result (output) for the event.
    pub result: Option<Value>,
    /// The scheduled time for the event.
    pub scheduled_time: Option<DateTime<Utc>>,
    /// The timestamp for the event.
    pub timestamp: DateTime<Utc>,
}

/// Represents an orchestration's status.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrchestrationStatus {
    /// The runtime status of the orchestration.
    pub runtime_status: OrchestrationRuntimeStatus,
    /// The input of the orchestration.
    pub input: Option<Value>,
    /// The custom status of the orchestration.
    pub custom_status: Option<Value>,
    /// The output of the orchestration.
    pub output: Option<Value>,
    /// The created time of the orchestration.
    pub created_time: DateTime<Utc>,
    /// The event history of the orchestration.
    pub history_events: Option<Vec<OrchestrationHistoryEvent>>,
}

/// Represents new orchestration data.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrchestrationData {
    /// The orchestration instance id.
    #[serde(rename = "id")]
    pub instance_id: String,
    /// The instance status query URI (GET).
    pub status_query_get_uri: String,
    /// The send event URI (POST).
    pub send_event_post_uri: String,
    /// The terminate instance URI (POST).
    pub terminate_post_uri: String,
    /// The purge history URI (DELETE).
    pub purge_history_delete_uri: String,
    /// The rewind URI (POST).
    pub rewind_post_uri: Option<String>,
}

/// Represents the Durable Functions HTTP client.
pub struct Client {
    endpoint: Endpoint,
    client: hyper::Client<hyper::client::HttpConnector>,
}

async fn body_from_response(res: hyper::Response<hyper::Body>) -> Result<impl bytes::Buf> {
    hyper::body::aggregate(res)
        .await
        .map_err(|e| ClientError::Message(format!("failed to read response: {}", e)))
}

impl Client {
    /// Creates a new client from the given status query URL.
    pub fn new(status_query_url: &str) -> Self {
        Self {
            endpoint: Endpoint::new(
                Url::parse(status_query_url).expect("expected a valid query URL"),
            ),
            client: hyper::Client::builder().build_http(),
        }
    }

    /// Gets the task hub associated with the client.
    pub fn task_hub(&self) -> &str {
        self.endpoint.task_hub()
    }

    /// Gets the status of an orchestration instance.
    pub async fn instance_status(
        &self,
        instance_id: &str,
        show_history: bool,
        show_history_output: bool,
        show_input: bool,
    ) -> Result<OrchestrationStatus> {
        let mut url = self.endpoint.status_query_url(Some(instance_id));

        url.query_pairs_mut()
            .append_pair("showHistory", if show_history { "true" } else { "false " })
            .append_pair(
                "showHistoryOutput",
                if show_history_output {
                    "true"
                } else {
                    "false "
                },
            )
            .append_pair("showInput", if show_input { "true" } else { "false " });

        let req = Request::builder()
            .method("GET")
            .uri(url.into_string())
            .header("Content-Type", "application/json")
            .body(Body::empty())
            .unwrap();

        match self.client.request(req).await {
            Ok(res) => match res.status() {
                StatusCode::OK | StatusCode::ACCEPTED => {
                    let body = body_from_response(res).await?;
                    from_slice(body.bytes()).map_err(|e| {
                        ClientError::Message(format!(
                            "failed to deserialize orchestration status: {}",
                            e
                        ))
                    })
                }
                StatusCode::BAD_REQUEST => Err(ClientError::InstanceFailedOrTerminated),
                StatusCode::NOT_FOUND => Err(ClientError::InstanceNotFound),
                StatusCode::INTERNAL_SERVER_ERROR => Err(ClientError::InternalServerError),
                _ => unreachable!("unexpected response from server"),
            },
            Err(e) => Err(ClientError::Message(format!(
                "failed to send request: {}",
                e
            ))),
        }
    }

    /// Queries the status for instances in a given date range or with runtime status.
    #[allow(clippy::too_many_arguments)]
    pub async fn query_instances<I>(
        &self,
        created_time_from: Option<DateTime<Utc>>,
        created_time_to: Option<DateTime<Utc>>,
        runtime_statuses: Option<I>,
        top: Option<u32>,
        show_history: bool,
        show_history_output: bool,
        show_input: bool,
    ) -> Result<Vec<OrchestrationStatus>>
    where
        I: Iterator<Item = OrchestrationRuntimeStatus>,
    {
        let mut url = self.endpoint.status_query_url(None);

        {
            let mut query = url.query_pairs_mut();

            created_time_from.map(|t| query.append_pair("createdTimeFrom", &t.to_rfc3339()));
            created_time_to.map(|t| query.append_pair("createdTimeTo", &t.to_rfc3339()));
            runtime_statuses.map(|s| {
                query.append_pair(
                    "runtimeStatus",
                    &s.map(|s| s.to_string()).collect::<Vec<_>>().join(","),
                )
            });

            top.map(|t| query.append_pair("top", &t.to_string()));

            query
                .append_pair("showHistory", if show_history { "true" } else { "false " })
                .append_pair(
                    "showHistoryOutput",
                    if show_history_output {
                        "true"
                    } else {
                        "false "
                    },
                )
                .append_pair("showInput", if show_input { "true" } else { "false " });
        }

        let req = Request::builder()
            .method("GET")
            .uri(url.into_string())
            .header("Content-Type", "application/json")
            .body(Body::empty())
            .unwrap();

        match self.client.request(req).await {
            Ok(res) => match res.status() {
                StatusCode::OK | StatusCode::ACCEPTED => {
                    let body = body_from_response(res).await?;
                    from_slice(body.bytes()).map_err(|e| {
                        ClientError::Message(format!(
                            "failed to deserialize orchestration status: {}",
                            e
                        ))
                    })
                }
                StatusCode::BAD_REQUEST => Err(ClientError::InstanceFailedOrTerminated),
                StatusCode::NOT_FOUND => Err(ClientError::InstanceNotFound),
                StatusCode::INTERNAL_SERVER_ERROR => Err(ClientError::InternalServerError),
                _ => unreachable!("unexpected response from server"),
            },
            Err(e) => Err(ClientError::Message(format!(
                "failed to send request: {}",
                e
            ))),
        }
    }

    /// Purges the history of the given orchestration instance.
    pub async fn purge_history(&self, instance_id: &str) -> Result<()> {
        let req = Request::builder()
            .method("DELETE")
            .uri(
                self.endpoint
                    .purge_history_url(Some(instance_id))
                    .into_string(),
            )
            .header("Content-Type", "application/json")
            .body(Body::empty())
            .unwrap();

        match self.client.request(req).await {
            Ok(res) => match res.status() {
                StatusCode::OK => Ok(()),
                StatusCode::NOT_FOUND => Err(ClientError::InstanceNotFound),
                _ => unreachable!("unexpected response from server"),
            },
            Err(e) => Err(ClientError::Message(format!(
                "failed to send request: {}",
                e
            ))),
        }
    }

    /// Purges the history of orchestrations matching the given date range or runtime statuses.
    pub async fn purge_history_by_query<I>(
        &self,
        created_time_from: Option<DateTime<Utc>>,
        created_time_to: Option<DateTime<Utc>>,
        runtime_statuses: Option<I>,
    ) -> Result<u32>
    where
        I: Iterator<Item = OrchestrationRuntimeStatus>,
    {
        let mut url = self.endpoint.purge_history_url(None);

        {
            let mut query = url.query_pairs_mut();

            created_time_from.map(|t| query.append_pair("createdTimeFrom", &t.to_rfc3339()));
            created_time_to.map(|t| query.append_pair("createdTimeTo", &t.to_rfc3339()));
            runtime_statuses.map(|s| {
                query.append_pair(
                    "runtimeStatus",
                    &s.map(|s| s.to_string()).collect::<Vec<_>>().join(","),
                )
            });
        }

        let req = Request::builder()
            .method("DELETE")
            .uri(url.into_string())
            .header("Content-Type", "application/json")
            .body(Body::empty())
            .unwrap();

        #[derive(Debug, Clone, Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct PurgeHistoryResult {
            instances_deleted: u32,
        }

        match self.client.request(req).await {
            Ok(res) => match res.status() {
                StatusCode::OK => {
                    let body = body_from_response(res).await?;
                    let result: PurgeHistoryResult = from_slice(body.bytes()).map_err(|e| {
                        ClientError::Message(format!(
                            "failed to deserialize orchestration status: {}",
                            e
                        ))
                    })?;

                    Ok(result.instances_deleted)
                }
                StatusCode::NOT_FOUND => Err(ClientError::InstanceNotFound),
                _ => unreachable!("unexpected response from server"),
            },
            Err(e) => Err(ClientError::Message(format!(
                "failed to send request: {}",
                e
            ))),
        }
    }

    /// Raises an event for the given orchestration instance.
    pub async fn raise_event<D>(
        &self,
        instance_id: &str,
        event_name: &str,
        event_data: D,
    ) -> Result<()>
    where
        D: Into<Value>,
    {
        let req = Request::builder()
            .method("POST")
            .uri(
                self.endpoint
                    .raise_event_url(instance_id, event_name)
                    .into_string(),
            )
            .header("Content-Type", "application/json")
            .body(Body::from(to_string(&event_data.into()).unwrap()))
            .unwrap();

        match self.client.request(req).await {
            Ok(res) => match res.status() {
                StatusCode::ACCEPTED => Ok(()),
                StatusCode::NOT_FOUND => Err(ClientError::InstanceNotFound),
                StatusCode::BAD_REQUEST => Err(ClientError::BadRequest),
                StatusCode::GONE => Err(ClientError::InstanceCompletedOrFailed),
                _ => unreachable!("unexpected response from server"),
            },
            Err(e) => Err(ClientError::Message(format!(
                "failed to send request: {}",
                e
            ))),
        }
    }

    /// Restores a failed orchestration instance into a running state by replaying the most recent failed operations.
    pub async fn rewind(&self, instance_id: &str, reason: &str) -> Result<()> {
        let req = Request::builder()
            .method("POST")
            .uri(self.endpoint.rewind_url(instance_id, reason).into_string())
            .header("Content-Type", "application/json")
            .body(Body::empty())
            .unwrap();

        match self.client.request(req).await {
            Ok(res) => match res.status() {
                StatusCode::ACCEPTED => Ok(()),
                StatusCode::NOT_FOUND => Err(ClientError::InstanceNotFound),
                StatusCode::BAD_REQUEST => Err(ClientError::BadRequest),
                StatusCode::GONE => Err(ClientError::InstanceCompletedOrFailed),
                _ => unreachable!("unexpected response from server"),
            },
            Err(e) => Err(ClientError::Message(format!(
                "failed to send request: {}",
                e
            ))),
        }
    }

    /// Starts a new orchestration by calling the given orchestration function.
    pub async fn start_new<D>(
        &self,
        function_name: &str,
        instance_id: Option<&str>,
        input: D,
    ) -> Result<OrchestrationData>
    where
        D: Into<Value>,
    {
        let req = Request::builder()
            .method("POST")
            .uri(
                self.endpoint
                    .create_new_instance_url(function_name, instance_id)
                    .into_string(),
            )
            .header("Content-Type", "application/json")
            .body(Body::from(input.into().to_string()))
            .unwrap();

        match self.client.request(req).await {
            Ok(res) => match res.status() {
                StatusCode::ACCEPTED => {
                    let body = body_from_response(res).await?;
                    from_slice(body.bytes()).map_err(|e| {
                        ClientError::Message(format!(
                            "failed to deserialize orchestration data: {}",
                            e
                        ))
                    })
                }
                StatusCode::BAD_REQUEST => Err(ClientError::BadCreateRequest),
                _ => unreachable!("unexpected response from server"),
            },
            Err(e) => Err(ClientError::Message(format!(
                "failed to send request: {}",
                e
            ))),
        }
    }

    /// Terminates a running orchestration instance.
    pub async fn terminate(&self, instance_id: &str, reason: &str) -> Result<()> {
        let req = Request::builder()
            .method("POST")
            .uri(
                self.endpoint
                    .terminate_url(instance_id, reason)
                    .into_string(),
            )
            .header("Content-Type", "application/json")
            .body(Body::empty())
            .unwrap();

        match self.client.request(req).await {
            Ok(res) => match res.status() {
                StatusCode::ACCEPTED => Ok(()),
                StatusCode::NOT_FOUND => Err(ClientError::InstanceNotFound),
                StatusCode::GONE => Err(ClientError::InstanceCompletedOrFailed),
                _ => unreachable!("unexpected response from server"),
            },
            Err(e) => Err(ClientError::Message(format!(
                "failed to send request: {}",
                e
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::offset::TimeZone;
    use serde_json::from_str;

    #[test]
    fn test_instance_history() {
        let h1: String = r#"{"EventType": "ExecutionStarted",
          "FunctionName": "E1_HelloSequence",
          "Timestamp": "2018-02-28T05:18:49Z"
        }"#
        .to_owned();

        let compare_dt = Utc.ymd(2018, 2, 28).and_hms(5, 18, 49);

        let h1_obj: OrchestrationHistoryEvent = from_str(&h1).unwrap();
        assert_eq!(h1_obj.event_type, "ExecutionStarted");
        assert_eq!(h1_obj.timestamp, compare_dt);

        let h2: String = r#"{
          "EventType": "ExecutionCompleted",
          "OrchestrationStatus": "Completed",
          "Result": [
              "Hello Tokyo!",
              "Hello Seattle!",
              "Hello London!"
          ],
          "Timestamp": "2018-02-28T05:18:54.3660895Z"
        }"#
        .to_owned();

        let h2_obj: OrchestrationHistoryEvent = from_str(&h2).unwrap();
        assert_eq!(h2_obj.orchestration_status.is_some(), true);
        assert_eq!(
            h2_obj.orchestration_status.unwrap(),
            OrchestrationRuntimeStatus::Completed
        );
    }

    #[test]
    fn test_instance_status() {
        let example: String = r#"{
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
        }"#
        .to_owned();

        let instance_status: OrchestrationStatus = from_str(&example).unwrap();
        assert_eq!(instance_status.history_events.is_some(), true);
        assert_eq!(instance_status.history_events.unwrap().len(), 5);

        assert_eq!(instance_status.custom_status.is_some(), true);
        assert_eq!(instance_status.custom_status.unwrap().is_object(), true);
    }
}
