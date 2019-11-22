use crate::util::nested_json;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// A unique identifier for an entity.
#[derive(Serialize, Deserialize, Default, Debug)]
#[serde(rename_all = "camelCase")]
pub(crate) struct EntityId {
    pub name: String,
    pub key: String,
}

#[doc(hidden)]
#[derive(Serialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub(crate) struct OperationResult {
    pub is_error: bool,
    pub duration: f64,
    #[serde(with = "nested_json")]
    pub result: Value,
}

#[doc(hidden)]
#[derive(Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Signal {
    pub target: EntityId,
    pub name: String,
    #[serde(with = "nested_json")]
    pub input: Value,
}

#[doc(hidden)]
#[derive(Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct EntityState {
    #[serde(rename = "entityExists")]
    pub(crate) exists: bool,
    #[serde(rename = "entityState", with = "nested_json")]
    pub(crate) value: Option<Value>,
    pub(crate) results: Vec<OperationResult>,
    pub(crate) signals: Vec<Signal>,
}
