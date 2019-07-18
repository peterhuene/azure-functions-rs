use serde_derive::Deserialize;

/// Represents the Durable Funtions client management URLs.
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
