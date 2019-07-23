//! Module for SignalR types.
use serde::{Deserialize, Serialize};

/// Represents an action to take on a SignalR group.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum GroupAction {
    /// Adds a user to a group.
    Add,
    /// Removes a user from a group.
    Remove,
}
