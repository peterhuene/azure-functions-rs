use crate::bindings::HttpResponse;
use crate::http::{Body, Status};

/// Represents a builder for HTTP responses.
#[derive(Default, Debug)]
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
    pub fn status<S: Into<Status>>(mut self, status: S) -> Self {
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
    pub fn header<T: Into<String>, U: Into<String>>(mut self, name: T, value: U) -> Self {
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
    ///     response.body().as_str().unwrap(),
    ///     message
    /// );
    /// ```
    pub fn body<'a, B>(mut self, body: B) -> Self
    where
        B: Into<Body<'a>>,
    {
        let body = body.into();
        if let Body::Empty = &body {
            self.0.data.clear_body();
            return self;
        }

        if !self.0.headers().contains_key("Content-Type") {
            self.0.data.mut_headers().insert(
                "Content-Type".to_string(),
                body.default_content_type().to_string(),
            );
        }
        self.0.data.set_body(body.into());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_creates_an_empty_response() {
        let response: HttpResponse = ResponseBuilder::new().into();
        assert_eq!(response.status(), Status::Ok);
        assert_eq!(response.body().as_str().unwrap(), "");
    }

    #[test]
    fn it_sets_a_status() {
        let response: HttpResponse = ResponseBuilder::new().status(Status::BadRequest).into();
        assert_eq!(response.status(), Status::BadRequest);
        assert_eq!(response.body().as_str().unwrap(), "");
    }

    #[test]
    fn it_sets_a_header() {
        let response: HttpResponse = ResponseBuilder::new().header("foo", "bar").into();
        assert_eq!(response.headers().get("foo").unwrap(), "bar");
        assert_eq!(response.body().as_str().unwrap(), "");
    }

    #[test]
    fn it_sets_a_body() {
        let response: HttpResponse = ResponseBuilder::new().body("test").into();
        assert_eq!(
            response.headers().get("Content-Type").unwrap(),
            "text/plain"
        );
        assert_eq!(response.body().as_str().unwrap(), "test");
    }
}
