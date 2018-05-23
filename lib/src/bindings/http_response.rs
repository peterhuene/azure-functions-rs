use http::{Body, Status};
use rpc::protocol;
use std::collections::HashMap;
use std::mem::replace;

/// Represents a HTTP output binding.
///
/// # Usage
///
/// Responses can be easily created for any type that implements `Into<Body>`.
///
/// # Examples
///
/// Creating a response from a string:
///
/// ```rust
/// use azure_functions::bindings::HttpResponse;
/// use azure_functions::http::{Body, Status};
///
/// let response: HttpResponse = "hello world".into();
///
/// assert_eq!(response.status(), Status::Ok);
/// assert_eq!(
///     response
///         .headers()
///         .get("Content-Type")
///         .unwrap(),
///     "text/plain");
/// assert_eq!(
///     match response.body() {
///         Body::String(s) => s,
///         _ => panic!("unexpected body.")
///     },
///     "hello world"
/// );
/// ```
///
/// Creating a response from a JSON value (see the [json! macro](https://docs.serde.rs/serde_json/macro.json.html) from the `serde_json` crate):
///
/// ```rust
/// # #[macro_use] extern crate serde_json;
/// # extern crate azure_functions;
/// use azure_functions::bindings::HttpResponse;
/// use azure_functions::http::{Body, Status};
///
/// let response: HttpResponse = json!({ "hello": "world!" }).into();
///
/// assert_eq!(response.status(), Status::Ok);
/// assert_eq!(
///     response
///         .headers()
///         .get("Content-Type")
///         .unwrap(),
///     "application/json"
/// );
/// assert_eq!(
///     match response.body() {
///         Body::Json(s) => s,
///         _ => panic!("unexpected body.")
///     },
///     "{\"hello\":\"world!\"}"
/// );
/// ```
///
/// Creating a response from a sequence of bytes:
///
/// ```rust
/// use azure_functions::bindings::HttpResponse;
/// use azure_functions::http::{Body, Status};
///
/// let response: HttpResponse = [1u8, 2u8, 3u8][..].into();
///
/// assert_eq!(response.status(), Status::Ok);
/// assert_eq!(
///     response
///         .headers()
///         .get("Content-Type")
///         .unwrap(),
///     "application/octet-stream"
/// );
/// assert_eq!(
///     &match response.body() {
///         Body::Bytes(bytes) => bytes,
///         _ => panic!("unexpected body.")
///     }[..],
///     [1u8, 2u8, 3u8]
/// );
/// ```
///
/// Building a custom response:
///
/// ```rust
/// use azure_functions::bindings::HttpResponse;
/// use azure_functions::http::{Body, Status};
///
/// let url = "http://example.com";
/// let body = format!("The requested resource has moved to: {}", url);
///
/// let response: HttpResponse = HttpResponse::build()
///     .status(Status::MovedPermanently)
///     .header("Location", url)
///     .body(body.as_str())
///     .into();
///
/// assert_eq!(response.status(), Status::MovedPermanently);
/// assert_eq!(
///     response
///         .headers()
///         .get("Location")
///         .unwrap(),
///     url
/// );
/// assert_eq!(
///     match response.body() {
///         Body::String(s) => s,
///         _ => panic!("unexpected body.")
///     },
///     body
/// );
/// ```
#[derive(Debug)]
pub struct HttpResponse {
    data: protocol::RpcHttp,
    status: Status,
}

impl HttpResponse {
    fn new() -> Self {
        HttpResponse {
            data: protocol::RpcHttp::new(),
            status: Status::Ok,
        }
    }

    /// Creates a new [HttpResponseBuilder](struct.HttpResponseBuilder.html) for building a response.
    pub fn build() -> HttpResponseBuilder {
        HttpResponseBuilder::new()
    }

    /// Gets the status code for the response.
    pub fn status(&self) -> Status {
        self.status
    }

    /// Gets the body of the response.
    pub fn body(&self) -> Body {
        if self.data.has_body() {
            Body::from(self.data.get_body())
        } else {
            Body::Empty
        }
    }

    /// Gets the headers of the response.
    pub fn headers(&self) -> &HashMap<String, String> {
        &self.data.headers
    }
}

impl Into<protocol::RpcHttp> for HttpResponse {
    fn into(mut self) -> protocol::RpcHttp {
        self.data.set_status_code(self.status.to_string());
        self.data
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

impl<'a> From<&'a mut HttpResponseBuilder> for HttpResponse {
    fn from(builder: &'a mut HttpResponseBuilder) -> Self {
        replace(&mut builder.0, HttpResponse::new())
    }
}

impl Into<protocol::TypedData> for HttpResponse {
    fn into(self) -> protocol::TypedData {
        let mut data = protocol::TypedData::new();
        data.set_http(self.data);
        data
    }
}

/// Represents a builder for HTTP responses.
#[derive(Debug)]
pub struct HttpResponseBuilder(HttpResponse);

impl HttpResponseBuilder {
    /// Creates a new `HttpResponseBuilder`.
    pub fn new() -> HttpResponseBuilder {
        HttpResponseBuilder(HttpResponse::new())
    }

    /// Sets the status for the response.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use azure_functions::bindings::HttpResponse;
    /// use azure_functions::http::Status;
    ///
    /// let response: HttpResponse = HttpResponse::build()
    ///     .status(Status::InternalServerError)
    ///     .into();
    ///
    /// assert_eq!(response.status(), Status::InternalServerError);
    /// ```
    pub fn status<S: Into<Status>>(&mut self, status: S) -> &mut Self {
        self.0.status = status.into();
        self
    }

    /// Sets a header for the response.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use azure_functions::bindings::HttpResponse;
    ///
    /// let value = "custom header value";
    ///
    /// let response: HttpResponse = HttpResponse::build()
    ///     .header("X-Custom-Header", value)
    ///     .into();
    ///
    /// assert_eq!(
    ///     response
    ///         .headers()
    ///         .get("X-Custom-Header")
    ///         .unwrap(),
    ///     value
    /// );
    /// ```
    pub fn header<T: Into<String>, U: Into<String>>(&mut self, name: T, value: U) -> &mut Self {
        self.0.data.mut_headers().insert(name.into(), value.into());
        self
    }

    /// Sets the body of the response.
    ///
    /// This will automatically set a `Content-Type` header for the response depending on the body type.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use azure_functions::bindings::HttpResponse;
    /// use azure_functions::http::{Body, Status};
    ///
    /// let message = "The resouce was created.";
    ///
    /// let response: HttpResponse = HttpResponse::build()
    ///     .status(Status::Created)
    ///     .body(message)
    ///     .into();
    ///
    /// assert_eq!(response.status(), Status::Created);
    /// assert_eq!(
    ///     match response.body() {
    ///         Body::String(s) => s,
    ///         _ => panic!("unexpected body.")
    ///     },
    ///     message
    /// );
    /// ```
    pub fn body<'a, B: Into<Body<'a>>>(&mut self, body: B) -> &mut Self {
        let body = body.into();
        match &body {
            Body::Empty => {
                self.0.data.clear_body();
                return self;
            }
            _ => {}
        };

        if !self.0.headers().contains_key("Content-Type") {
            self.header("Content-Type", body.default_content_type());
        }
        self.0.data.set_body(body.into());
        self
    }
}
