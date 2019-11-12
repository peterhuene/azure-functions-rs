use crate::http::Body;
use crate::rpc::{typed_data::Data, TypedData};
use azure_functions_durable::{
    Client, OrchestrationData, OrchestrationRuntimeStatus, OrchestrationStatus, Result,
};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use serde_json::{from_str, to_value, Value};

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
/// Starting a new orchestration:
///
/// ```rust
/// use azure_functions::{
///     bindings::{DurableOrchestrationClient, HttpRequest, HttpResponse},
///     func,
/// };
/// use serde_json::Value;
///
/// #[func]
/// pub async fn start(_req: HttpRequest, client: DurableOrchestrationClient) -> HttpResponse {
///     match client
///         .start_new(
///             "orchestration",
///             None,
///             Value::Null,
///         )
///         .await
///     {
///         Ok(data) => data.into(),
///         Err(e) => format!("Failed to start orchestration: {}", e).into(),
///     }
/// }
/// ```
pub struct DurableOrchestrationClient {
    client: Client,
}

impl DurableOrchestrationClient {
    /// Gets the status of an orchestration instance.
    pub async fn instance_status(
        &self,
        instance_id: &str,
        show_history: bool,
        show_history_output: bool,
        show_input: bool,
    ) -> Result<OrchestrationStatus> {
        self.client
            .instance_status(instance_id, show_history, show_history_output, show_input)
            .await
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
        self.client
            .query_instances(
                created_time_from,
                created_time_to,
                runtime_statuses,
                top,
                show_history,
                show_history_output,
                show_input,
            )
            .await
    }

    /// Purges the history of the given orchestration instance.
    pub async fn purge_history(&self, instance_id: &str) -> Result<()> {
        self.client.purge_history(instance_id).await
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
        self.client
            .purge_history_by_query(created_time_from, created_time_to, runtime_statuses)
            .await
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
        self.client
            .raise_event(instance_id, event_name, event_data)
            .await
    }

    /// Restores a failed orchestration instance into a running state by replaying the most recent failed operations.
    pub async fn rewind(&self, instance_id: &str, reason: &str) -> Result<()> {
        self.client.rewind(instance_id, reason).await
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
        self.client
            .start_new(function_name, instance_id, input)
            .await
    }

    /// Terminates a running orchestration instance.
    pub async fn terminate(&self, instance_id: &str, reason: &str) -> Result<()> {
        self.client.terminate(instance_id, reason).await
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

impl<'a> Into<Body<'a>> for OrchestrationData {
    fn into(self) -> Body<'a> {
        to_value(&self).unwrap().into()
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
