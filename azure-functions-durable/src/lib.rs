//! # Durable Functions for Rust
#![feature(async_await)]
#![feature(result_map_or_else)]

#[macro_use]
extern crate derive_builder;
extern crate hyper;
//extern crate hyper_tls;
extern crate futures;

use crate::futures::FutureExt;
use crate::futures::TryFutureExt;
use crate::futures::TryStreamExt;
use chrono::prelude::*;
use futures::{future, Future};
use log::{debug, info, warn};
use serde::{Deserialize, Serialize};
use serde_json::{from_slice, from_str, Map, Value};
use std::fmt::{Debug, Display, Formatter};
use std::result::Result::*;
use url::Url;

//use hyper_tls::HttpsConnector;
use hyper::{client::HttpConnector, Body, Client, Request, StatusCode, Uri};

static EVENT_NAME_PLACEHOLDER: &'static str = "{eventName}";
static FUNC_NAME_PLACEHOLDER: &'static str = "{functionName}";
static INSTANCE_ID_PLACEHOLDER: &'static str = "[/{instanceId}]";
static REASON_PLACEHOLDER: &'static str = "{text}";
static CREATED_TIME_FROM_KEY: &'static str = "createdTimeFrom";
static CREATED_TIME_TO_KEY: &'static str = "createdTimeTo";
static RUNTIME_STATUS_KEY: &'static str = "runtimeStatus";
static SHOW_HISTORY_KEY: &'static str = "showHistory";
static SHOW_HISTORY_OUTPUT_KEY: &'static str = "showHistoryOutput";
static SHOW_INPUT_KEY: &'static str = "showInput";

#[derive(Debug, Clone, Builder)]
pub(crate) struct OrchestrationEndpoint {
    base_uri: Url,
    task_hub: String,
    connection: String,
    code: String,
}

impl OrchestrationEndpoint {
    pub(crate) fn new(status_query_url: &str) -> OrchestrationEndpoint {
        let status_url = Url::parse(status_query_url).unwrap();
        OrchestrationEndpoint::new_from_url(status_url)
    }

    pub(crate) fn new_from_url(status_query_url: Url) -> OrchestrationEndpoint {
        let mut builder = OrchestrationEndpointBuilder::default();
        builder.base_uri(status_query_url.clone());
        for (k, v) in status_query_url.query_pairs() {
            match k.to_ascii_lowercase().as_ref() {
                "taskhub" => builder.task_hub(v.into_owned()),
                "connection" => builder.connection(v.into_owned()),
                "code" => builder.code(v.into_owned()),
                _ => &mut builder,
            };
        }

        builder.build().unwrap()
    }

    pub(crate) fn create_new_instance_url(
        &self,
        function_name: &str,
        instance_id: Option<&str>,
    ) -> Url {
        let mut new_url = self.base_uri.clone();
        let instance_id = instance_id.map_or("".to_owned(), |i| format!("/{}", i));
        let path = format!(
            "/runtime/webhooks/durabletask/orchestrators/{}{}",
            function_name, instance_id
        );
        new_url
            .query_pairs_mut()
            .clear()
            .append_pair("code", &self.code);
        new_url.set_path(&path);
        new_url
    }

    pub(crate) fn build_query_url(
        &self,
        instance_id: Option<&str>,
        action: Option<&str>,
        task_hub: Option<&str>,
        connection: Option<&str>,
        code: Option<&str>,
    ) -> Url {
        let mut new_url = self.base_uri.clone();
        let instance_id = instance_id.map_or("".to_owned(), |i| format!("/{}", i));
        let action = action.map_or("".to_owned(), |a| format!("/{}", a));
        let path = format!(
            "/runtime/webhooks/durabletask/instances{}{}",
            instance_id, action
        );
        new_url.set_path(&path);
        new_url
            .query_pairs_mut()
            .clear()
            .append_pair("taskHub", task_hub.unwrap_or(&self.task_hub))
            .append_pair("connection", connection.unwrap_or(&self.connection))
            .append_pair("code", code.unwrap_or(&self.code));
        new_url
    }

    pub(crate) fn status_query_url(
        &self,
        instance_id: Option<&str>,
        task_hub: Option<&str>,
        connection: Option<&str>,
        code: Option<&str>,
    ) -> Url {
        self.build_query_url(instance_id, None, task_hub, connection, code)
    }

    pub(crate) fn purge_history_url(
        &self,
        instance_id: &str,
        task_hub: Option<&str>,
        connection: Option<&str>,
        code: Option<&str>,
    ) -> Url {
        self.status_query_url(Some(instance_id), task_hub, connection, code)
    }

    pub(crate) fn rewind_url(
        &self,
        instance_id: &str,
        reason: &str,
        task_hub: Option<&str>,
        connection: Option<&str>,
        code: Option<&str>,
    ) -> Url {
        let mut re_url = self.build_query_url(
            Some(instance_id),
            Some("rewind"),
            task_hub,
            connection,
            code,
        );
        re_url.query_pairs_mut().append_pair("reason", reason);
        re_url
    }

    pub(crate) fn raise_event_url(
        &self,
        instance_id: &str,
        event_name: &str,
        task_hub: Option<&str>,
        connection: Option<&str>,
        code: Option<&str>,
    ) -> Url {
        let event_name = format!("raiseEvent/{}", event_name);
        self.build_query_url(
            Some(instance_id),
            Some(&event_name),
            task_hub,
            connection,
            code,
        )
    }

    pub(crate) fn terminate_url(
        &self,
        instance_id: &str,
        reason: &str,
        task_hub: Option<&str>,
        connection: Option<&str>,
        code: Option<&str>,
    ) -> Url {
        let mut re_url = self.build_query_url(
            Some(instance_id),
            Some("terminate"),
            task_hub,
            connection,
            code,
        );
        re_url.query_pairs_mut().append_pair("reason", reason);
        re_url
    }
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub enum OrchestrationRuntimeStatus {
    Running,
    Pending,
    Failed,
    Canceled,
    Terminated,
    Completed,
}

impl Display for OrchestrationRuntimeStatus {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
        // or, alternatively:
        // fmt::Debug::fmt(self, f)
    }
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub enum OrchestrationClientError {
    //400
    InstanceFailedOrTerminated,
    //410
    InstanceCompletedOrFailed,
    //404
    InstanceNotFound,
    //400
    BadRaiseEventContent,
    //500
    UnspecifiedError,
    CommunicationError(String),
    InvalidResponse,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct OrchestrationHistoryEvent {
    event_type: String,
    orchestration_status: Option<OrchestrationRuntimeStatus>,
    function_name: Option<String>,
    result: Option<Value>,
    scheduled_time: Option<DateTime<Utc>>,
    timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrchestrationStatus {
    runtime_status: OrchestrationRuntimeStatus,
    input: Option<Value>,
    custom_status: Option<Value>,
    output: Option<Value>,
    created_time: DateTime<Utc>,
    history_events: Option<Vec<OrchestrationHistoryEvent>>,
}

#[derive(Default, Builder, Debug)]
#[builder(setter(strip_option), default)]
pub struct InstanceQuery<'a> {
    instance_id: Option<&'a str>,
    created_time_from: Option<DateTime<Utc>>,
    created_time_to: Option<DateTime<Utc>>,
    runtime_status: Option<Vec<OrchestrationRuntimeStatus>>,
    task_hub: Option<&'a str>,
    connection_name: Option<&'a str>,
    code: Option<&'a str>,
    show_history: Option<bool>,
    show_history_output: Option<bool>,
    show_input: Option<bool>,
    show_output: Option<bool>,
    top: Option<u32>,
}

impl<'a> InstanceQuery<'a> {
    pub fn with_id(id: &str) -> InstanceQuery {
        InstanceQueryBuilder::default()
            .instance_id(id)
            .build()
            .unwrap()
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PurgeHistoryResult {
    instances_deleted: u32,
}

type OrchestrationResult<T> = Result<T, OrchestrationClientError>;

pub struct OrchestrationClient {
    endpoint: OrchestrationEndpoint,
    client: Client<HttpConnector>,
}

impl OrchestrationClient {
    pub fn new(status_query_url: &str) -> OrchestrationClient {
        OrchestrationClient::new_with_client_builder(status_query_url, Client::builder())
    }

    pub fn new_with_client_builder(
        status_query_url: &str,
        client_builder: hyper::client::Builder,
    ) -> OrchestrationClient {
        //let https = HttpsConnector::new(4).unwrap();
        OrchestrationClient {
            endpoint: OrchestrationEndpoint::new(status_query_url),
            client: client_builder.build_http(),
        }
    }

    pub fn new_from_params(
        origin: &str,
        task_hub: &str,
        connection: &str,
        code: &str,
        client_builder: Option<hyper::client::Builder>,
    ) -> OrchestrationClient {
        let end_url = Url::parse_with_params(
            origin,
            &[
                ("taskHub", task_hub),
                ("connection", connection),
                ("code", code),
            ],
        )
        .unwrap();
        let client_builder = client_builder.unwrap_or(Client::builder());
        OrchestrationClient {
            endpoint: OrchestrationEndpoint::new_from_url(end_url),
            client: client_builder.build_http(),
        }
    }

    pub fn task_hub(&self) -> &str {
        &self.endpoint.task_hub
    }

    pub async fn get_status<'a>(
        &self,
        query: InstanceQuery<'a>,
    ) -> OrchestrationResult<OrchestrationStatus> {
        let mut status_url = self.endpoint.status_query_url(
            query.instance_id,
            query.task_hub,
            query.connection_name,
            query.code,
        );

        status_url
            .query_pairs_mut()
            .append_pair(
                "showHistory",
                &query.show_history.unwrap_or(false).to_string(),
            )
            .append_pair(
                "showHistoryOutput",
                &query.show_history_output.unwrap_or(false).to_string(),
            )
            .append_pair("showInput", &query.show_input.unwrap_or(false).to_string());

        debug!(target:"azure_functions_durable", "Querying URL {:?}", status_url);

        let req = Request::builder()
            .method("GET")
            .uri(status_url.into_string())
            .header("Content-Type", "application/json")
            .body(Body::empty())
            .unwrap();

        let res = self.client.request(req).await;
        match res {
            Ok(response) => match response.status() {
                StatusCode::OK
                | StatusCode::ACCEPTED
                | StatusCode::BAD_REQUEST
                | StatusCode::NOT_FOUND
                | StatusCode::INTERNAL_SERVER_ERROR => {
                    let body = response.into_body().try_concat().await;
                    body.map_or_else(
                        |e| {
                            Err(OrchestrationClientError::CommunicationError(format!(
                                "{:?}",
                                e
                            )))
                        },
                        |b| {
                            serde_json::from_slice::<OrchestrationStatus>(&b).map_err(|e2| {
                                OrchestrationClientError::CommunicationError(format!("{:?}", e2))
                            })
                        },
                    )
                }
                s => Err(OrchestrationClientError::CommunicationError(format!(
                    "Web hook returned unknown status code {}",
                    s
                ))),
            },
            Err(e) => Err(OrchestrationClientError::CommunicationError(format!(
                "{:?}",
                e
            ))),
        }
    }

    pub async fn get_statuses_by<'a>(
        &self,
        query: InstanceQuery<'a>,
    ) -> OrchestrationResult<Vec<OrchestrationStatus>> {
        let mut status_url = self.endpoint.status_query_url(
            query.instance_id,
            query.task_hub,
            query.connection_name,
            query.code,
        );

        let mut url_query = status_url.query_pairs_mut();
        query
            .created_time_from
            .map(|ctf| url_query.append_pair("createdTimeFrom", &ctf.to_rfc3339()));
        query
            .created_time_to
            .map(|ctt| url_query.append_pair("createdTimeTo", &ctt.to_rfc3339()));
        query.runtime_status.map(|rsv| {
            let statuses: Vec<String> = rsv.iter().map(|s| s.to_string()).collect();
            url_query.append_pair("runtimeStatus", &statuses.join(","))
        });
        query
            .top
            .map(|t| url_query.append_pair("top", &t.to_string()));

        url_query
            .append_pair(
                "showHistory",
                &query.show_history.unwrap_or(false).to_string(),
            )
            .append_pair(
                "showHistoryOutput",
                &query.show_history_output.unwrap_or(false).to_string(),
            )
            .append_pair("showInput", &query.show_input.unwrap_or(false).to_string());

        debug!(target:"azure_functions_durable", "Querying URL {:?}", status_url);

        let req = Request::builder()
            .method("GET")
            .uri(status_url.into_string())
            .header("Content-Type", "application/json")
            .body(Body::empty())
            .unwrap();

        let res = self.client.request(req).await;
        match res {
            Ok(response) => {
                if response.status() > StatusCode::ACCEPTED {
                    Err(OrchestrationClientError::CommunicationError(format!(
                        "Web hook returned unknown status code {}",
                        response.status()
                    )))
                } else {
                    let body = response.into_body().try_concat().await;
                    body.map_or_else(
                        |e| {
                            Err(OrchestrationClientError::CommunicationError(format!(
                                "{:?}",
                                e
                            )))
                        },
                        |b| {
                            serde_json::from_slice(&b).map_err(|e2| {
                                OrchestrationClientError::CommunicationError(format!("{:?}", e2))
                            })
                        },
                    )
                }
            }
        }
    }

    pub async fn purge_instance_history<'a>(
        &self,
        query: InstanceQuery<'a>,
    ) -> OrchestrationResult<PurgeHistoryResult> {
        Err(OrchestrationClientError::UnspecifiedError)
    }

    pub async fn purge_history_by<'a>(
        &self,
        query: InstanceQuery<'a>,
    ) -> OrchestrationResult<PurgeHistoryResult> {
        Err(OrchestrationClientError::UnspecifiedError)
    }

    pub async fn raise_event<'a, D>(
        &self,
        event_name: &str,
        event_data: D,
        query: InstanceQuery<'a>,
    ) -> OrchestrationResult<()>
    where
        D: Into<serde_json::Value>,
    {
        Err(OrchestrationClientError::UnspecifiedError)
    }

    pub async fn rewind<'a>(
        &self,
        reason: &str,
        query: InstanceQuery<'a>,
    ) -> OrchestrationResult<()> {
        Err(OrchestrationClientError::UnspecifiedError)
    }

    pub async fn start_new<D>(
        &self,
        orchestrator_function_name: &str,
        instance_id: Option<&str>,
        input: Option<D>,
    ) -> OrchestrationResult<String>
    where
        D: Into<serde_json::Value>,
    {
        let creationUri = self
            .endpoint
            .create_new_instance_url(orchestrator_function_name, instance_id);
        let body_value = input.map_or("".to_owned(), |i| i.into().to_string());

        let req = Request::builder()
            .method("POST")
            .uri(creationUri.into_string())
            .header("Content-Type", "application/json")
            .body(Body::from(body_value))
            .unwrap();

        let res = self.client.request(req).await;
        match res {
            Ok(response) => {
                if response.status() > StatusCode::ACCEPTED {
                    Err(OrchestrationClientError::UnspecifiedError)
                } else {
                    let body = response.into_body().try_concat().await;
                    body.map_or_else(
                        |e| {
                            Err(OrchestrationClientError::CommunicationError(format!(
                                "{:?}",
                                e
                            )))
                        },
                        |b| {
                            from_slice::<Map<String, Value>>(&b)
                                .map_err(|e2| {
                                    OrchestrationClientError::CommunicationError(format!(
                                        "{:?}",
                                        e2
                                    ))
                                })
                                .map(|create_response| {
                                    create_response.get("id").map_or(
                                        Err(OrchestrationClientError::CommunicationError(
                                            "No `id` in response".to_owned(),
                                        )),
                                        |id| {
                                            id.as_str().map_or(
                                                Err(OrchestrationClientError::CommunicationError(
                                                    "Id in response is not a string".to_owned(),
                                                )),
                                                |id_str| Ok(id_str.to_owned()),
                                            )
                                        },
                                    )
                                })
                                .unwrap()
                        },
                    )
                }
            }
            Err(e) => Err(OrchestrationClientError::CommunicationError(format!(
                "{:?}",
                e
            ))),
        }
    }

    pub async fn terminate<'a>(
        &self,
        reason: &str,
        query: InstanceQuery<'a>,
    ) -> OrchestrationResult<()> {
        Err(OrchestrationClientError::UnspecifiedError)
    }
}

#[cfg(test)]
use mockito;
use std::ops::Deref;

#[cfg(test)]
extern crate tokio;

#[cfg(test)]
mod tests {
    use futures::future::lazy;
    use mockito::mock;
    use tokio;
    use tokio_core::reactor::Core;

    use super::*;
    use hyper::rt;
    use hyper::{Body, Request, Response, Server};

    static EP_GOOD: &'static str = "http://localhost:7071/runtime/webhooks/durabletask/instances/INSTANCEID?taskHub=myHub&connection=Storage&code=myCode";
    static EP_BAD: &'static str =
        "http://localhost:7071/runtime/webhooks/durabletask/instances/INSTANC";

    #[test]
    fn test_endpoint_parsing() {
        let endpoint = OrchestrationEndpoint::new(EP_GOOD);
        assert_eq!(endpoint.code, "myCode");

        let rewind_result = "http://localhost:7071/runtime/webhooks/durabletask/instances/1234/rewind?taskHub=myHub&connection=Storage&code=myCode&reason=myReason";
        let rewind_url = endpoint.rewind_url("1234", "myReason", None, None, None);
        println!("{:?}", rewind_url);
        assert_eq!(rewind_url.to_string(), rewind_result);
    }

    #[test]
    #[should_panic]
    fn test_bad_endpoint() {
        OrchestrationEndpoint::new(EP_BAD);
    }

    #[test]
    fn test_instance_history() {
        let h1: String = r#"{"EventType": "ExecutionStarted",
          "FunctionName": "E1_HelloSequence",
          "Timestamp": "2018-02-28T05:18:49Z"
        }"#
        .to_owned();

        let compare_dt = Utc.ymd(2018, 2, 28).and_hms(5, 18, 49);

        let h1_obj: OrchestrationHistoryEvent = serde_json::from_str(&h1).unwrap();
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

        let h2_obj: OrchestrationHistoryEvent = serde_json::from_str(&h2).unwrap();
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

        let instance_status: OrchestrationStatus = serde_json::from_str(&example).unwrap();
        assert_eq!(instance_status.history_events.is_some(), true);
        assert_eq!(instance_status.history_events.unwrap().len(), 5);

        assert_eq!(instance_status.custom_status.is_some(), true);
        assert_eq!(instance_status.custom_status.unwrap().is_object(), true);
    }

    #[test]
    fn test_query_builder() {
        let query = InstanceQueryBuilder::default()
            .created_time_from(Utc.ymd(2018, 2, 28).and_hms(5, 18, 49))
            .build()
            .unwrap();
        assert_eq!(query.created_time_from.is_some(), true);
    }
}
