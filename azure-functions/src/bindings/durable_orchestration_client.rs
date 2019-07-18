use crate::{
    durable::{CreationUrls, ManagementUrls},
    rpc::{typed_data::Data, TypedData},
};
use serde_derive::Deserialize;
use serde_json::from_str;

/// Represents the Durable Functions orchestration client input binding.
///
/// The following binding attributes are supported:
///
/// | Name         | Description                                                                                                                                 |
/// |--------------|---------------------------------------------------------------------------------------------------------------------------------------------|
/// | `name`       | The name of the parameter being bound.                                                                                                      |
/// | `task_hub`   | The name of the task hub to use.  Defaults to the value from host.json                                                                      |
/// | `connection` | The name of an app setting that contains a storage account connection string. Defaults to the storage account for the function application. |
///
/// # Examples
///
/// TODO: IMPLEMENT
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DurableOrchestrationClient {
    #[serde(rename = "taskHubName")]
    task_hub: String,
    creation_urls: CreationUrls,
    management_urls: ManagementUrls,
}

#[doc(hidden)]
impl From<TypedData> for DurableOrchestrationClient {
    fn from(data: TypedData) -> Self {
        match &data.data {
            Some(Data::String(s)) => {
                from_str(s).expect("failed to parse durable orchestration client data")
            }
            _ => panic!("expected string data for durable orchestration client"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_converts_from_typed_data() {
        let data = TypedData {
            data: Some(Data::String(r#"{"taskHubName":"test","creationUrls":{"createNewInstancePostUri":"http://localhost:8080/runtime/webhooks/durabletask/orchestrators/{functionName}[/{instanceId}]?code=foo","createAndWaitOnNewInstancePostUri":"http://localhost:8080/runtime/webhooks/durabletask/orchestrators/{functionName}[/{instanceId}]?timeout={timeoutInSeconds}&pollingInterval={intervalInSeconds}&code=foo"},"managementUrls":{"id":"INSTANCEID","statusQueryGetUri":"http://localhost:8080/runtime/webhooks/durabletask/instances/INSTANCEID?taskHub=DurableFunctionsHub&connection=Storage&code=foo","sendEventPostUri":"http://localhost:8080/runtime/webhooks/durabletask/instances/INSTANCEID/raiseEvent/{eventName}?taskHub=DurableFunctionsHub&connection=Storage&code=foo","terminatePostUri":"http://localhost:8080/runtime/webhooks/durabletask/instances/INSTANCEID/terminate?reason={text}&taskHub=DurableFunctionsHub&connection=Storage&code=foo","rewindPostUri":"http://localhost:8080/runtime/webhooks/durabletask/instances/INSTANCEID/rewind?reason={text}&taskHub=DurableFunctionsHub&connection=Storage&code=foo","purgeHistoryDeleteUri":"http://localhost:8080/runtime/webhooks/durabletask/instances/INSTANCEID?taskHub=DurableFunctionsHub&connection=Storage&code=foo"}}"#.to_owned())),
        };

        let client: DurableOrchestrationClient = data.into();
        assert_eq!(client.task_hub, "test");
        assert_eq!(
            client.creation_urls.create_new_instance_url,
            "http://localhost:8080/runtime/webhooks/durabletask/orchestrators/{functionName}[/{instanceId}]?code=foo"
        );
        assert_eq!(
            client.creation_urls.create_new_instance_and_wait_url,
            "http://localhost:8080/runtime/webhooks/durabletask/orchestrators/{functionName}[/{instanceId}]?timeout={timeoutInSeconds}&pollingInterval={intervalInSeconds}&code=foo"
        );
        assert_eq!(client.management_urls.id, "INSTANCEID");
        assert_eq!(
            client.management_urls.status_query_url,
            "http://localhost:8080/runtime/webhooks/durabletask/instances/INSTANCEID?taskHub=DurableFunctionsHub&connection=Storage&code=foo"
        );
        assert_eq!(
            client.management_urls.raise_event_url,
            "http://localhost:8080/runtime/webhooks/durabletask/instances/INSTANCEID/raiseEvent/{eventName}?taskHub=DurableFunctionsHub&connection=Storage&code=foo"
        );
        assert_eq!(
            client.management_urls.terminate_url,
            "http://localhost:8080/runtime/webhooks/durabletask/instances/INSTANCEID/terminate?reason={text}&taskHub=DurableFunctionsHub&connection=Storage&code=foo"
        );
        assert_eq!(
            client.management_urls.rewind_url,
            "http://localhost:8080/runtime/webhooks/durabletask/instances/INSTANCEID/rewind?reason={text}&taskHub=DurableFunctionsHub&connection=Storage&code=foo"
        );
        assert_eq!(
            client.management_urls.purge_history_url,
            "http://localhost:8080/runtime/webhooks/durabletask/instances/INSTANCEID?taskHub=DurableFunctionsHub&connection=Storage&code=foo"
        );
    }
}
