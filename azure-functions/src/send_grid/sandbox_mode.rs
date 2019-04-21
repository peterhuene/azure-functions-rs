use serde_derive::{Deserialize, Serialize};

/// Represents the ability to send a test email in a sandbox.
///
/// This setting allows you to send a test email to ensure that your request body is valid
/// and formatted correctly.
///
/// For more information, please see the classroom documentation:
/// https://sendgrid.com/docs/Classroom/Send/v3_Mail_Send/sandbox_mode.html
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct SandboxMode {
    /// The value indicating whether this setting is enabled.
    pub enable: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::to_string;

    #[test]
    fn it_serializes_to_json() {
        let json = to_string(&SandboxMode { enable: true }).unwrap();

        assert_eq!(json, r#"{"enable":true}"#);
    }
}
