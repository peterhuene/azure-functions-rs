use serde_derive::{Deserialize, Serialize};

/// Represents an email address containing the email address and name of the sender or recipient.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct EmailAddress {
    /// The email address of the sender or recipient.
    pub email: String,
    /// The name of the sender or recipient.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

impl EmailAddress {
    /// Creates a new email address.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use azure_functions::send_grid::EmailAddress;
    /// let address = EmailAddress::new("foo@example.com");
    /// assert_eq!(address.email, "foo@example.com");
    /// assert_eq!(address.name, None);
    /// ```
    pub fn new<T>(email: T) -> EmailAddress
    where
        T: Into<String>,
    {
        EmailAddress {
            email: email.into(),
            name: None,
        }
    }

    /// Creates a new email address with the given name.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use azure_functions::send_grid::EmailAddress;
    /// let address = EmailAddress::new_with_name("foo@example.com", "Peter");
    /// assert_eq!(address.email, "foo@example.com");
    /// assert_eq!(address.name, Some("Peter".to_string()));
    /// ```
    pub fn new_with_name<T, U>(email: T, name: U) -> EmailAddress
    where
        T: Into<String>,
        U: Into<String>,
    {
        EmailAddress {
            email: email.into(),
            name: Some(name.into()),
        }
    }
}

impl<T> From<T> for EmailAddress
where
    T: Into<String>,
{
    fn from(email: T) -> Self {
        EmailAddress::new(email)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::to_string;

    #[test]
    fn it_serializes_to_json() {
        let json = to_string(&EmailAddress {
            email: "foo@example.com".to_owned(),
            name: None,
        })
        .unwrap();

        assert_eq!(json, r#"{"email":"foo@example.com"}"#);

        let json = to_string(&EmailAddress {
            email: "foo@example.com".to_owned(),
            name: Some("Foo Example".to_owned()),
        })
        .unwrap();

        assert_eq!(json, r#"{"email":"foo@example.com","name":"Foo Example"}"#);
    }

    #[test]
    fn it_converts_from_string() {
        let address: EmailAddress = "foo@example.com".into();

        assert_eq!(address.email, "foo@example.com");
        assert_eq!(address.name, None);
    }
}
