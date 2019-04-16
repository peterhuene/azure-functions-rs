use crate::{
    http::{Body, ResponseBuilder, Status},
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
///         .into()
/// }
/// ```
#[derive(Default, Debug)]
pub struct HttpResponse {
    pub(crate) data: RpcHttp,
    pub(crate) status: Status,
}

impl HttpResponse {
    pub(crate) fn new() -> Self {
        HttpResponse {
            data: RpcHttp::default(),
            status: Status::Ok,
        }
    }

    /// Creates a new [ResponseBuilder](../http/struct.ResponseBuilder.html) for building a response.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use azure_functions::bindings::HttpResponse;
    /// use azure_functions::http::Status;
    ///
    /// let response: HttpResponse = HttpResponse::build().status(Status::NotFound).into();
    /// assert_eq!(response.status(), Status::NotFound);
    /// ```
    pub fn build() -> ResponseBuilder {
        ResponseBuilder::new()
    }

    /// Gets the status code for the response.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use azure_functions::bindings::HttpResponse;
    /// use azure_functions::http::Status;
    ///
    /// let response: HttpResponse = HttpResponse::build().status(Status::BadRequest).into();
    /// assert_eq!(response.status(), Status::BadRequest);
    /// ```
    pub fn status(&self) -> Status {
        self.status
    }

    /// Gets the body of the response.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use azure_functions::bindings::HttpResponse;
    ///
    /// let response: HttpResponse = HttpResponse::build().body("example").into();
    /// assert_eq!(response.body().as_str().unwrap(), "example");
    /// ```
    pub fn body(&self) -> Body {
        self.data
            .body
            .as_ref()
            .map(|b| Body::from(&**b))
            .unwrap_or(Body::Empty)
    }

    /// Gets the headers of the response.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use azure_functions::bindings::HttpResponse;
    ///
    /// let response: HttpResponse = HttpResponse::build().header("Content-Type", "text/plain").into();
    /// assert_eq!(response.headers().get("Content-Type").unwrap(), "text/plain");
    /// ```
    pub fn headers(&self) -> &HashMap<String, String> {
        &self.data.headers
    }
}

impl<'a, T> From<T> for HttpResponse
where
    T: Into<Body<'a>>,
{
    fn from(data: T) -> Self {
        HttpResponse::build().body(data).into()
    }
}

impl From<ResponseBuilder> for HttpResponse {
    fn from(builder: ResponseBuilder) -> Self {
        builder.0
    }
}

#[doc(hidden)]
impl Into<TypedData> for HttpResponse {
    fn into(mut self) -> TypedData {
        self.data.status_code = self.status.to_string();

        TypedData {
            data: Some(Data::Http(Box::new(self.data))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use matches::matches;
    use serde::{Deserialize, Serialize};

    #[test]
    fn it_is_empty_by_default() {
        let response = HttpResponse::new();

        assert_eq!(response.status(), Status::Ok);
        assert!(matches!(response.body(), Body::Empty));
    }

    #[test]
    fn it_is_empty_from_a_builder() {
        let response: HttpResponse = HttpResponse::build().into();

        assert_eq!(response.status(), Status::Ok);
        assert!(matches!(response.body(), Body::Empty));
    }

    #[test]
    fn it_builds_with_a_status() {
        let response: HttpResponse = HttpResponse::build().status(Status::Continue).into();

        assert_eq!(response.status(), Status::Continue);
    }

    #[test]
    fn it_builds_with_a_string_body() {
        const BODY: &'static str = "test body";

        let response: HttpResponse = HttpResponse::build().body(BODY).into();

        assert_eq!(
            response.headers().get("Content-Type").unwrap(),
            "text/plain"
        );
        assert_eq!(response.body().as_str().unwrap(), BODY);
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

        let response: HttpResponse = HttpResponse::build()
            .body(::serde_json::to_value(data).unwrap())
            .into();

        assert_eq!(
            response.headers().get("Content-Type").unwrap(),
            "application/json"
        );
        assert_eq!(response.body().as_json::<Data>().unwrap().message, MESSAGE);
    }

    #[test]
    fn it_builds_with_a_bytes_body() {
        const BODY: &'static [u8] = &[1, 2, 3];

        let response: HttpResponse = HttpResponse::build().body(BODY).into();

        assert_eq!(
            response.headers().get("Content-Type").unwrap(),
            "application/octet-stream"
        );
        assert_eq!(response.body().as_bytes(), BODY);
    }

    #[test]
    fn it_builds_with_headers() {
        let response: HttpResponse = HttpResponse::build()
            .header("header1", "value1")
            .header("header2", "value2")
            .header("header3", "value3")
            .into();

        assert_eq!(response.headers().get("header1").unwrap(), "value1");
        assert_eq!(response.headers().get("header2").unwrap(), "value2");
        assert_eq!(response.headers().get("header3").unwrap(), "value3");
    }

    #[test]
    fn it_converts_to_typed_data() {
        let response: HttpResponse = HttpResponse::build()
            .status(Status::BadRequest)
            .header("header", "value")
            .body("body")
            .into();

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
