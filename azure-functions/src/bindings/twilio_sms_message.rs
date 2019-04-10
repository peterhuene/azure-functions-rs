use crate::{
    rpc::{typed_data::Data, TypedData},
    FromVec,
};
use serde_derive::{Deserialize, Serialize};
use serde_json::{to_string, to_value, Value};

/// Represents the Twilio SMS message output binding.
///
/// The following binding attributes are supported:
///
/// | Name          | Description                                                                                                                                        |
/// |---------------|----------------------------------------------------------------------------------------------------------------------------------------------------|
/// | `name`        | The name of the parameter being bound.                                                                                                             |
/// | `account_sid` | The name of an app setting that holds your Twilio Account SID. If not set, the default app setting name is "AzureWebJobsTwilioAccountSid".         |
/// | `auth_token`  | The name of an app setting that holds your Twilio authentication token. If not set, the default app setting name is "AzureWebJobsTwilioAuthToken". |
/// | `from`        | The default phone number that the SMS text is sent from.                                                                                           |
/// | `body`        | The default SMS message body to use.                                                                                                               |
///
/// # Examples
///
/// An example HTTP-triggered function that outputs a Twilio SMS message:
///
/// ```rust
/// use azure_functions::{
///     bindings::{HttpRequest, TwilioSmsMessage},
///     func,
/// };
/// use std::borrow::ToOwned;
///
/// #[func]
/// #[binding(name = "$return", from = "+15555555555")]
/// pub fn send_sms(req: HttpRequest) -> TwilioSmsMessage {
///     let params = req.query_params();
///
///     TwilioSmsMessage {
///         to: params.get("to").unwrap().to_owned(),
///         body: params.get("body").map(ToOwned::to_owned),
///         ..Default::default()
///     }
/// }
/// ```
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TwilioSmsMessage {
    /// The phone number to send the SMS message to.
    pub to: String,
    /// The optional phone number to send the SMS message from. If None, the `from` binding attribute is used.
    pub from: Option<String>,
    /// The optional SMS message body. If None, the `body` binding attribute is used.
    pub body: Option<String>,
}

#[doc(hidden)]
impl Into<TypedData> for TwilioSmsMessage {
    fn into(self) -> TypedData {
        TypedData {
            data: Some(Data::Json(
                to_string(&self).expect("failed to convert Twilio SMS message to JSON string"),
            )),
        }
    }
}

#[doc(hidden)]
impl FromVec<TwilioSmsMessage> for TypedData {
    fn from_vec(vec: Vec<TwilioSmsMessage>) -> Self {
        TypedData {
            data: Some(Data::Json(
                Value::Array(vec.into_iter().map(|m| to_value(m).unwrap()).collect()).to_string(),
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_serializes_to_json() {
        let json = to_string(&TwilioSmsMessage {
            to: "foo".to_owned(),
            from: Some("bar".to_owned()),
            body: Some("baz".to_owned()),
        })
        .unwrap();

        assert_eq!(json, r#"{"to":"foo","from":"bar","body":"baz"}"#);
    }

    #[test]
    fn it_converts_to_typed_data() {
        let message = TwilioSmsMessage {
            to: "foo".to_owned(),
            from: Some("bar".to_owned()),
            body: Some("baz".to_owned()),
        };

        let data: TypedData = message.into();
        assert_eq!(
            data.data,
            Some(Data::Json(
                r#"{"to":"foo","from":"bar","body":"baz"}"#.to_string()
            ))
        );
    }
}
