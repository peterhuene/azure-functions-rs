use crate::{
    http::{Body, Cookie, ResponseBuilder, Status},
    rpc::{typed_data::Data, RpcHttp, TypedData},
};
use std::collections::HashMap;

/// Represents a HTTP output binding.
///
/// The following binding attributes are supported:
///
/// | Name         | Description                                                                                                                                                                                       |
/// |--------------|---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
/// | `name`       | The name of the parameter being bound.                                                                                                                                                            |
///
/// Responses can be created for any type that implements `Into<Body>`.
///
/// # Examples
///
/// Creating a response from a string:
///
/// ```rust
/// use azure_functions::bindings::{HttpRequest, HttpResponse};
/// use azure_functions::func;
///
/// #[func]
/// pub fn example(_req: HttpRequest) -> HttpResponse {
///     "Hello world!".into()
/// }
/// ```
///
/// Creating a response from a JSON value (see the [json! macro](https://docs.serde.rs/serde_json/macro.json.html) from the `serde_json` crate):
///
/// ```rust
/// use azure_functions::bindings::{HttpRequest, HttpResponse};
/// use azure_functions::func;
/// use serde_json::json;
///
/// #[func]
/// pub fn example(_req: HttpRequest) -> HttpResponse {
///     json!({ "hello": "world" }).into()
/// }
/// ```
///
/// Creating a response from a sequence of bytes:
///
/// ```rust
/// use azure_functions::bindings::{HttpRequest, HttpResponse};
/// use azure_functions::func;
///
/// #[func]
/// pub fn example(_req: HttpRequest) -> HttpResponse {
///     [1, 2, 3][..].into()
/// }
/// ```
///
/// Building a custom response:
///
/// ```rust
/// use azure_functions::bindings::{HttpRequest, HttpResponse};
/// use azure_functions::func;
/// use azure_functions::http::Status;
///
/// #[func]
/// pub fn example(_req: HttpRequest) -> HttpResponse {
///     HttpResponse::build()
///         .status(Status::MovedPermanently)
///         .header("Location", "http://example.com")
///         .body("The requested resource has moved to: http://example.com")
///         .finish()
/// }
/// ```
#[derive(Default, Debug)]
pub struct HttpResponse {
    /// The status code of the response.
    pub status: Status,
    /// The headers of the response.
    pub headers: HashMap<String, String>,
    /// The body of the response.
    pub body: Body,
    /// Whether or not content negotiation is enabled in the response.
    pub enable_content_negotiation: bool,
    /// The cookies of the response.
    pub cookies: Vec<Cookie>,
}

impl HttpResponse {
    /// Creates a new HttpResponse.
    pub fn new() -> Self {
        Self {
            status: Status::Ok,
            ..Default::default()
        }
    }

    /// Creates a new [ResponseBuilder](../http/struct.ResponseBuilder.html) for building a response.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use azure_functions::bindings::HttpResponse;
    /// use azure_functions::http::Status;
    ///
    /// let response = HttpResponse::build().status(Status::NotFound).finish();
    /// assert_eq!(response.status, Status::NotFound);
    /// ```
    pub fn build() -> ResponseBuilder {
        ResponseBuilder::new()
    }
}

impl<T> From<T> for HttpResponse
where
    T: Into<Body>,
{
    fn from(body: T) -> Self {
        Self::build().body(body).finish()
    }
}

#[doc(hidden)]
impl Into<TypedData> for HttpResponse {
    fn into(self) -> TypedData {
        TypedData {
            data: Some(Data::Http(Box::new(RpcHttp {
                headers: self.headers,
                body: Some(Box::new(self.body.into())),
                status_code: self.status.to_string(),
                enable_content_negotiation: self.enable_content_negotiation,
                cookies: self.cookies.into_iter().map(|c| c.into()).collect(),
                ..Default::default()
            }))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};
    use serde_json::{from_slice, to_value};

    #[test]
    fn it_is_empty_by_default() {
        let response = HttpResponse::new();

        assert_eq!(response.status, Status::Ok);
        assert_eq!(response.body.to_str().unwrap(), "");
    }

    #[test]
    fn it_is_empty_from_a_builder() {
        let response: HttpResponse = HttpResponse::build().finish();

        assert_eq!(response.status, Status::Ok);
        assert_eq!(response.body.to_str().unwrap(), "");
    }

    #[test]
    fn it_builds_with_a_status() {
        let response: HttpResponse = HttpResponse::build().status(Status::Continue).finish();

        assert_eq!(response.status, Status::Continue);
    }

    #[test]
    fn it_builds_with_a_string_body() {
        const BODY: &'static str = "test body";

        let response: HttpResponse = HttpResponse::build().body(BODY).finish();

        assert_eq!(response.headers.get("Content-Type").unwrap(), "text/plain");
        assert_eq!(response.body.to_str().unwrap(), BODY);
    }

    #[test]
    fn it_builds_with_a_json_body() {
        #[derive(Serialize, Deserialize)]
        struct Data {
            message: String,
        };

        const MESSAGE: &'static str = "test";

        let data = Data {
            message: MESSAGE.to_string(),
        };

        let response = HttpResponse::build().body(to_value(data).unwrap()).finish();

        assert_eq!(
            response.headers.get("Content-Type").unwrap(),
            "application/json"
        );

        let data: Data = from_slice(response.body.as_bytes()).unwrap();
        assert_eq!(data.message, MESSAGE);
    }

    #[test]
    fn it_builds_with_a_bytes_body() {
        const BODY: &'static [u8] = &[1, 2, 3];

        let response: HttpResponse = HttpResponse::build().body(BODY).finish();

        assert_eq!(
            response.headers.get("Content-Type").unwrap(),
            "application/octet-stream"
        );
        assert_eq!(response.body.as_bytes(), BODY);
    }

    #[test]
    fn it_builds_with_headers() {
        let response: HttpResponse = HttpResponse::build()
            .header("header1", "value1")
            .header("header2", "value2")
            .header("header3", "value3")
            .finish();

        assert_eq!(response.headers.get("header1").unwrap(), "value1");
        assert_eq!(response.headers.get("header2").unwrap(), "value2");
        assert_eq!(response.headers.get("header3").unwrap(), "value3");
    }

    #[test]
    fn it_converts_to_typed_data() {
        let response: HttpResponse = HttpResponse::build()
            .status(Status::BadRequest)
            .header("header", "value")
            .body("body")
            .finish();

        let data: TypedData = response.into();
        match data.data {
            Some(Data::Http(http)) => {
                assert_eq!(http.status_code, "400");
                assert_eq!(http.headers.get("header").unwrap(), "value");
                assert_eq!(
                    http.body,
                    Some(Box::new(TypedData {
                        data: Some(Data::String("body".to_string()))
                    }))
                );
            }
            _ => assert!(false),
        }
    }
}
