use crate::durable::HttpRequest;
use chrono::{DateTime, Utc};
use serde::Serialize;
use serde_json::Value;

/// Defines retry policies that can be passed as parameters to various Durable Functions operations.
#[derive(Debug, Clone, Default, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct RetryOptions {
    /// The first retry interval in milliseconds.
    #[serde(rename = "firstRetryIntervalInMilliseconds")]
    pub first_retry_interval_ms: i32,

    /// The maximum number of retry attempts.
    #[serde(rename = "maxNumberOfAttempts")]
    pub max_attempts: i32,

    /// The backoff coefficient used to determine rate of increase of backoff. Defaults to 1.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub backoff_coefficient: Option<f64>,

    /// The max retry interval in milliseconds.
    #[serde(
        rename = "maxRetryIntervalInMilliseconds",
        skip_serializing_if = "Option::is_none"
    )]
    pub max_retry_interval_ms: Option<i32>,

    /// The timeout for retries in milliseconds.
    #[serde(
        rename = "retryTimeoutInMilliseconds",
        skip_serializing_if = "Option::is_none"
    )]
    pub retry_timeout_ms: Option<i32>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(tag = "actionType", rename_all = "camelCase")]
pub(crate) enum Action {
    #[serde(rename_all = "camelCase")]
    CallActivity { function_name: String, input: Value },

    #[serde(rename_all = "camelCase")]
    CallActivityWithRetry {
        function_name: String,
        retry_options: RetryOptions,
        input: Value,
    },

    #[serde(rename_all = "camelCase")]
    CallSubOrchestrator {
        function_name: String,
        instance_id: Option<String>,
        input: Value,
    },

    #[serde(rename_all = "camelCase")]
    CallSubOrchestratorWithRetry {
        function_name: String,
        retry_options: RetryOptions,
        instance_id: Option<String>,
        input: Value,
    },

    #[serde(rename_all = "camelCase")]
    ContinueAsNew {
        input: Value,
        preserve_unprocessed_events: bool,
    },

    #[serde(rename_all = "camelCase")]
    CreateTimer {
        fire_at: DateTime<Utc>,

        #[serde(rename = "isCanceled")]
        canceled: bool,
    },

    #[serde(rename_all = "camelCase")]
    WaitForExternalEvent { external_event_name: String },

    #[serde(rename_all = "camelCase")]
    CallEntity {
        instance_id: String,
        operation: String,
        input: Value,
    },

    #[serde(rename_all = "camelCase")]
    CallHttp { http_request: HttpRequest },
}

#[cfg(test)]
mod tests {
    use crate::durable::{Action, RetryOptions};
    use chrono::{DateTime, Utc};

    macro_rules! it_converts_to_json {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (action, expected) = $value;
                    let json = serde_json::to_string(&action).unwrap();
                    assert_eq!(expected, json);
                }
            )*
            }
        }
    it_converts_to_json! {
        call_activity_converts_to_json:
        (
            Action::CallActivity {
                function_name: "hello".to_owned(),
                input: "World".into(),
            },
            r#"{"actionType":"callActivity","functionName":"hello","input":"World"}"#
        ),
        call_activity_with_retry_converts_to_json:
        (
            Action::CallActivityWithRetry {
                function_name: "hello".to_owned(),
                retry_options: RetryOptions {
                    first_retry_interval_ms: 1000,
                    max_attempts: 3,
                    ..Default::default()
                },
                input: "World".into(),
            },
            r#"{"actionType":"callActivityWithRetry","functionName":"hello","retryOptions":{"firstRetryIntervalInMilliseconds":1000,"maxNumberOfAttempts":3},"input":"World"}"#
        ),
        call_sub_orchestrator_converts_to_json:
        (
            Action::CallSubOrchestrator {
                function_name: "hello".to_string(),
                instance_id: Some("1231232144".to_string()),
                input: "World".into()
            },
            r#"{"actionType":"callSubOrchestrator","functionName":"hello","instanceId":"1231232144","input":"World"}"#
        ),
        call_sub_orchestrator_with_retry_converts_to_json:
        (
            Action::CallSubOrchestratorWithRetry {
                function_name: "hello".to_string(),
                retry_options: RetryOptions {
                    first_retry_interval_ms: 1000,
                    max_attempts: 3,
                    ..Default::default()
                },
                instance_id: Some("1231232144".to_string()),
                input: "World".into()
            },
            r#"{"actionType":"callSubOrchestratorWithRetry","functionName":"hello","retryOptions":{"firstRetryIntervalInMilliseconds":1000,"maxNumberOfAttempts":3},"instanceId":"1231232144","input":"World"}"#
        ),
        continue_as_new_converts_to_json:
        (
            Action::ContinueAsNew { input: "World".into(), preserve_unprocessed_events: true, },
            r#"{"actionType":"continueAsNew","input":"World","preserveUnprocessedEvents":true}"#
        ),
        create_timer_converts_to_json:
        (
           Action::CreateTimer {
                fire_at: DateTime::<Utc>::from(DateTime::parse_from_rfc3339("2019-07-18T06:22:27.016757Z").unwrap()),
                canceled: true,
           },
           r#"{"actionType":"createTimer","fireAt":"2019-07-18T06:22:27.016757Z","isCanceled":true}"#
        ),
        wait_for_external_event_converts_to_json:
        (
            Action::WaitForExternalEvent { external_event_name: "SmsChallengeResponse".to_string() },
            r#"{"actionType":"waitForExternalEvent","externalEventName":"SmsChallengeResponse"}"#
        ),
    }
}
