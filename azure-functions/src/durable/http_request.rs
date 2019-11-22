use serde::Serialize;
use std::collections::HashMap;

/// Represents an Azure Active Directory token source.
#[derive(Debug, Clone, Default, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TokenSource {
    /// The Azure Active Directory resource identifier of the web API being invoked.
    pub resource: String,
}

/// Represents a HTTP request from a Durable Orchestration function.
#[derive(Debug, Clone, Default, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct HttpRequest {
    /// The HTTP method to use for the request.
    pub method: String,
    /// The URI to use for the request.
    pub uri: String,
    /// The content of the request.
    pub content: String,
    /// The headers of the request.
    pub headers: HashMap<String, String>,
    /// The Azure Active Directory token source to use for the request.
    pub token_source: TokenSource,
}
