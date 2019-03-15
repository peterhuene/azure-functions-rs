use crate::http::Body;
use crate::rpc::protocol;
use serde_derive::{Deserialize, Serialize};
use serde_json::{from_str, to_string};
use std::borrow::Cow;

/// Represents the SignalR connection information input binding.
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
impl From<protocol::TypedData> for SignalRConnectionInfo {
    fn from(data: protocol::TypedData) -> Self {
        from_str(data.get_json()).expect("failed to parse SignalR connection info")
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
        let mut data = protocol::TypedData::new();
        data.set_json(r#"{ "url": "foo", "accessToken": "bar"}"#.to_owned());

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
