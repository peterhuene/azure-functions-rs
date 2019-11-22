use crate::http::Status;
use serde::Deserialize;
use std::collections::HashMap;

/// Represents a HTTP response for a Durable Orchestration function.
#[derive(Debug, Clone, Default, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct HttpResponse {
    /// The status code of the response.
    pub status_code: Status,
    /// The content of the response.
    pub content: String,
    /// The headers of the response.
    pub headers: HashMap<String, String>,
}
