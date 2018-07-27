use bindings::HttpResponse;
use http::{Body, Status};

/// Represents a builder for HTTP responses.
#[derive(Debug)]
pub struct ResponseBuilder(pub(crate) HttpResponse);

impl ResponseBuilder {
    /// Creates a new `ResponseBuilder`.
    pub fn new() -> ResponseBuilder {
        ResponseBuilder(HttpResponse::new())
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
    pub fn body<B>(&mut self, body: B) -> &mut Self
    where
        B: Into<Body<'a>>,
    {
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
