use serde_derive::Deserialize;

/// Represents the Durable Funtions client creation URLs.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreationUrls {
    /// The URL for creating a new orchestration instance.
    #[serde(rename = "createNewInstancePostUri")]
    pub create_new_instance_url: String,
    /// The URL for creating and waiting on a new orchestration instance.
    #[serde(rename = "createAndWaitOnNewInstancePostUri")]
    pub create_new_instance_and_wait_url: String,
}
