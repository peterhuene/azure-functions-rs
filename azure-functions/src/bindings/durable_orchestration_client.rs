use crate::rpc::{typed_data::Data, TypedData};
use azure_functions_durable::{Client, OrchestrationData, Result};
use serde::Deserialize;
use serde_json::{from_str, Value};

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
pub struct DurableOrchestrationClient {
    client: Client,
}

impl DurableOrchestrationClient {
    /// Starts a new orchestration.
    ///
    /// TODO: provide example
    pub async fn start_new<D>(
        &self,
        function_name: &str,
        instance_id: Option<&str>,
        input: D,
    ) -> Result<OrchestrationData>
    where
        D: Into<Value>,
    {
        self.client
            .start_new(function_name, instance_id, input)
            .await
    }
}

#[doc(hidden)]
impl From<TypedData> for DurableOrchestrationClient {
    fn from(data: TypedData) -> Self {
        #[derive(Debug, Clone, Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct ManagementUrls {
            #[serde(rename = "statusQueryGetUri")]
            status_query_url: String,
        }

        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct BindingData {
            management_urls: ManagementUrls,
        }

        let data: BindingData = match &data.data {
            Some(Data::String(s)) => {
                from_str(s).expect("failed to parse durable orchestration client data")
            }
            _ => panic!("expected string data for durable orchestration client"),
        };

        DurableOrchestrationClient {
            client: Client::new(&data.management_urls.status_query_url),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_converts_from_typed_data() {
        let data = TypedData {
            data: Some(Data::String(r#"{"taskHubName":"DurableFunctionsHub","creationUrls":{"createNewInstancePostUri":"http://localhost:8080/runtime/webhooks/durabletask/orchestrators/{functionName}[/{instanceId}]?code=foo","createAndWaitOnNewInstancePostUri":"http://localhost:8080/runtime/webhooks/durabletask/orchestrators/{functionName}[/{instanceId}]?timeout={timeoutInSeconds}&pollingInterval={intervalInSeconds}&code=foo"},"managementUrls":{"id":"INSTANCEID","statusQueryGetUri":"http://localhost:8080/runtime/webhooks/durabletask/instances/INSTANCEID?taskHub=DurableFunctionsHub&connection=Storage&code=foo","sendEventPostUri":"http://localhost:8080/runtime/webhooks/durabletask/instances/INSTANCEID/raiseEvent/{eventName}?taskHub=DurableFunctionsHub&connection=Storage&code=foo","terminatePostUri":"http://localhost:8080/runtime/webhooks/durabletask/instances/INSTANCEID/terminate?reason={text}&taskHub=DurableFunctionsHub&connection=Storage&code=foo","rewindPostUri":"http://localhost:8080/runtime/webhooks/durabletask/instances/INSTANCEID/rewind?reason={text}&taskHub=DurableFunctionsHub&connection=Storage&code=foo","purgeHistoryDeleteUri":"http://localhost:8080/runtime/webhooks/durabletask/instances/INSTANCEID?taskHub=DurableFunctionsHub&connection=Storage&code=foo"}}"#.to_owned())),
        };

        let client: DurableOrchestrationClient = data.into();
        assert_eq!(client.client.task_hub(), "DurableFunctionsHub");
    }
}
