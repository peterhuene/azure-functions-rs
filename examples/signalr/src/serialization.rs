use serde_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct ChatMessage {
    pub sender: Option<String>,
    pub text: Option<String>,
    #[serde(rename = "groupname")]
    pub group_name: Option<String>,
    pub recipient: Option<String>,
    #[serde(rename = "isPrivate")]
    pub is_private: Option<bool>,
}
