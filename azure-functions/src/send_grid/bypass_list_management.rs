use serde_derive::{Deserialize, Serialize};

/// Represents the ability to bypass list management for an email message.
///
/// This setting allows you to bypass all unsubscribe groups and suppressions to ensure
/// that the email is delivered to every single recipient. This should only be used in
/// emergencies when it is absolutely necessary that every recipient receives your email.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct BypassListManagement {
    /// The value indicating whether this setting is enabled.
    pub enable: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::to_string;

    #[test]
    fn it_serializes_to_json() {
        let json = to_string(&BypassListManagement { enable: true }).unwrap();

        assert_eq!(json, r#"{"enable":true}"#);
    }
}
