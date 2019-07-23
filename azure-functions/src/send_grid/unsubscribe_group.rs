use serde::{Deserialize, Serialize};

/// Represents an unsubscribe group associated with an email message that specifies how to handle unsubscribes.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct UnsubscribeGroup {
    /// The unsubscribe group id associated with the email message.
    pub group_id: i32,
    /// The list containing the unsubscribe groups that you would like to be displayed on the unsubscribe preferences page.
    /// See https://sendgrid.com/docs/User_Guide/Suppressions/recipient_subscription_preferences.html
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub groups_to_display: Vec<i32>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::to_string;

    #[test]
    fn it_serializes_to_json() {
        let json = to_string(&UnsubscribeGroup {
            group_id: 12345,
            groups_to_display: Vec::new(),
        })
        .unwrap();

        assert_eq!(json, r#"{"group_id":12345}"#);

        let json = to_string(&UnsubscribeGroup {
            group_id: 12345,
            groups_to_display: vec![1, 2, 3, 4, 5],
        })
        .unwrap();

        assert_eq!(
            json,
            r#"{"group_id":12345,"groups_to_display":[1,2,3,4,5]}"#
        );
    }
}
