use crate::{
    http::Body,
    rpc::{typed_data::Data, TypedData},
};
use serde::{Deserialize, Serialize};
use serde_json::{from_str, to_string};
use std::borrow::Cow;

/// Represents the SignalR connection information input binding.
///
/// The following binding attributes are supported:
///
/// | Name         | Description                                                                                                                  |
/// |--------------|------------------------------------------------------------------------------------------------------------------------------|
/// | `name`       | The name of the parameter being bound.                                                                                       |
/// | `hub_name`   | The name of the SignalR hub for which the connection information is generated.                                               |
/// | `user_id`    | The value of the user identifier claim to be set in the access key token (optional).                                         |
/// | `connection` | The name of the app setting that contains the SignalR Service connection string. Defaults to `AzureSignalRConnectionString`. |
///
/// # Examples
///
/// This example implements an HTTP-triggered Azure Function that returns SignalR connection information:
///
/// ```rust
/// use azure_functions::{
///     bindings::{HttpRequest, HttpResponse, SignalRConnectionInfo},
///     func,
/// };
///
/// #[func]
/// #[binding(name = "_req", auth_level = "anonymous")]
/// #[binding(
///     name = "info",
///     hub_name = "chat",
///     user_id = "{headers.x-ms-signalr-userid}",
///     connection = "myconnection"
/// )]
/// pub fn negotiate(_req: HttpRequest, info: SignalRConnectionInfo) -> HttpResponse {
///     info.into()
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SignalRConnectionInfo {
    /// The endpoint URL for the SignalR service.
    pub url: String,
    /// The access token for the SignalR service.
    pub access_token: String,
}

#[doc(hidden)]
impl From<TypedData> for SignalRConnectionInfo {
    fn from(data: TypedData) -> Self {
        match &data.data {
            Some(Data::Json(s)) => from_str(s).expect("failed to parse SignalR connection info"),
            _ => panic!("expected JSON data for SignalR connection info"),
        }
    }
}

impl<'a> Into<Body<'a>> for SignalRConnectionInfo {
    fn into(self) -> Body<'a> {
        Body::Json(Cow::from(
            to_string(&self).expect("failed to serialize SignalR connection info"),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_serializes_to_json() {
        let json = to_string(&SignalRConnectionInfo {
            url: "foo".to_owned(),
            access_token: "bar".to_owned(),
        })
        .unwrap();

        assert_eq!(json, r#"{"url":"foo","accessToken":"bar"}"#);
    }

    #[test]
    fn it_converts_from_typed_data() {
        let data = TypedData {
            data: Some(Data::Json(
                r#"{ "url": "foo", "accessToken": "bar"}"#.to_owned(),
            )),
        };

        let info: SignalRConnectionInfo = data.into();
        assert_eq!(info.url, "foo");
        assert_eq!(info.access_token, "bar");
    }

    #[test]
    fn it_converts_to_body() {
        let info = SignalRConnectionInfo {
            url: "foo".to_owned(),
            access_token: "bar".to_owned(),
        };

        let body: Body = info.into();
        assert_eq!(
            body.as_str().unwrap(),
            r#"{"url":"foo","accessToken":"bar"}"#
        );
    }
}
