use chrono::{DateTime, FixedOffset};
use serde::Serialize;
use serde_json::Value;

#[doc(hidden)]
#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(tag = "actionType", rename_all = "camelCase")]
pub enum Action {
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
    ContinueAsNew { input: Value },

    #[serde(rename_all = "camelCase")]
    CreateTimer {
        fire_at: DateTime<FixedOffset>,
        is_cancelled: bool,
    },

    #[serde(rename_all = "camelCase")]
    WaitForExternalEvent { external_event_name: String },
}

#[doc(hidden)]
#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct RetryOptions {
    first_retry_interval_in_milliseconds: i32,
    max_number_of_attempts: i32,
}

#[cfg(test)]
mod tests {
    use crate::durable::{Action, RetryOptions};
    use chrono::DateTime;

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
                    first_retry_interval_in_milliseconds: 1000,
                    max_number_of_attempts: 3,
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
                    first_retry_interval_in_milliseconds: 1000,
                    max_number_of_attempts: 3,
                },
                instance_id: Some("1231232144".to_string()),
                input: "World".into()
            },
            r#"{"actionType":"callSubOrchestratorWithRetry","functionName":"hello","retryOptions":{"firstRetryIntervalInMilliseconds":1000,"maxNumberOfAttempts":3},"instanceId":"1231232144","input":"World"}"#
        ),
        continue_as_new_converts_to_json:
        (
            Action::ContinueAsNew { input: "World".into() },
            r#"{"actionType":"continueAsNew","input":"World"}"#
        ),
        create_timer_converts_to_json:
        (
           Action::CreateTimer {
                fire_at: DateTime::parse_from_rfc3339("2019-07-18T06:22:27.016757+00:00").unwrap(),
                is_cancelled: true,
           },
           r#"{"actionType":"createTimer","fireAt":"2019-07-18T06:22:27.016757+00:00","isCancelled":true}"#
        ),
        wait_for_external_event_converts_to_json:
        (
            Action::WaitForExternalEvent { external_event_name: "SmsChallengeResponse".to_string() },
            r#"{"actionType":"waitForExternalEvent","externalEventName":"SmsChallengeResponse"}"#
        ),
    }
}
