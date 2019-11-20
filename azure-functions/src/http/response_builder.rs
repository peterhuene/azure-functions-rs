use crate::bindings::HttpResponse;
use crate::http::{Body, Cookie, SameSitePolicy, Status};

/// Represents a builder for HTTP responses.
#[derive(Default, Debug)]
pub struct ResponseBuilder(pub(crate) HttpResponse);

impl ResponseBuilder {
    /// Creates a new `ResponseBuilder`.
    pub fn new() -> Self {
        Self(HttpResponse::new())
    }

    /// Sets the status for the response.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use azure_functions::http::ResponseBuilder;
    /// use azure_functions::http::Status;
    ///
    /// let response = ResponseBuilder::new()
    ///     .status(Status::InternalServerError)
    ///     .finish();
    ///
    /// assert_eq!(response.status, Status::InternalServerError);
    /// ```
    pub fn status<S: Into<Status>>(mut self, status: S) -> Self {
        self.0.status = status.into();
        self
    }

    /// Sets a header for the response.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use azure_functions::http::ResponseBuilder;
    ///
    /// let value = "custom header value";
    ///
    /// let response = ResponseBuilder::new()
    ///     .header("X-Custom-Header", value)
    ///     .finish();
    ///
    /// assert_eq!(
    ///     response
    ///         .headers
    ///         .get("X-Custom-Header")
    ///         .unwrap(),
    ///     value
    /// );
    /// ```
    pub fn header<T: Into<String>, U: Into<String>>(mut self, name: T, value: U) -> Self {
        self.0.headers.insert(name.into(), value.into());
        self
    }

    /// Sets the body of the response.
    ///
    /// This will automatically set a `Content-Type` header for the response depending on the body type.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use azure_functions::http::ResponseBuilder;
    /// use azure_functions::http::{Body, Status};
    ///
    /// let message = "The resouce was created.";
    ///
    /// let response = ResponseBuilder::new()
    ///     .status(Status::Created)
    ///     .body(message)
    ///     .finish();
    ///
    /// assert_eq!(response.status, Status::Created);
    /// assert_eq!(
    ///     response.body.to_str().unwrap(),
    ///     message
    /// );
    /// ```
    pub fn body<B>(mut self, body: B) -> Self
    where
        B: Into<Body>,
    {
        let body = body.into();

        if !self.0.headers.contains_key("Content-Type") {
            self.0.headers.insert(
                "Content-Type".to_string(),
                body.default_content_type().to_string(),
            );
        }

        self.0.body = body;
        self
    }

    /// Adds a cookie to the response.
    ///
    /// The cookie will be secure, HTTP-only, and use a strict same site policy.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use azure_functions::http::ResponseBuilder;
    ///
    /// let response = ResponseBuilder::new()
    ///     .cookie("hello", "world")
    ///     .finish();
    ///
    /// assert_eq!(response.cookies[0].name, "hello");
    /// assert_eq!(response.cookies[0].value, "world");
    /// ```
    pub fn cookie<T: Into<String>, U: Into<String>>(mut self, name: U, value: T) -> Self {
        self.0.cookies.push(Cookie {
            name: name.into(),
            value: value.into(),
            secure: Some(true),
            http_only: Some(true),
            same_site_policy: SameSitePolicy::Strict,
            ..Default::default()
        });
        self
    }

    /// Consumes the builder and returns the HTTP response.
    pub fn finish(self) -> HttpResponse {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_creates_an_empty_response() {
        let response: HttpResponse = ResponseBuilder::new().finish();
        assert_eq!(response.status, Status::Ok);
        assert_eq!(response.body.to_str().unwrap(), "");
    }

    #[test]
    fn it_sets_a_status() {
        let response: HttpResponse = ResponseBuilder::new().status(Status::BadRequest).finish();
        assert_eq!(response.status, Status::BadRequest);
        assert_eq!(response.body.to_str().unwrap(), "");
    }

    #[test]
    fn it_sets_a_header() {
        let response: HttpResponse = ResponseBuilder::new().header("foo", "bar").finish();
        assert_eq!(response.headers.get("foo").unwrap(), "bar");
        assert_eq!(response.body.to_str().unwrap(), "");
    }

    #[test]
    fn it_sets_a_body() {
        let response: HttpResponse = ResponseBuilder::new().body("test").finish();
        assert_eq!(response.headers.get("Content-Type").unwrap(), "text/plain");
        assert_eq!(response.body.to_str().unwrap(), "test");
    }
}
