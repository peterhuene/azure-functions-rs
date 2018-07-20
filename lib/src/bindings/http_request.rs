use http::Body;
use rpc::protocol;
use std::collections::HashMap;

/// Represents a HTTP trigger binding.
///
/// # Examples
///
/// A function that responds with a friendly greeting:
///
/// ```rust
/// # #![feature(use_extern_macros)] extern crate azure_functions;
/// use azure_functions::func;
/// use azure_functions::bindings::{HttpRequest, HttpResponse};
///
/// #[func]
/// #[binding(name = "request", auth_level = "anonymous")]
/// pub fn greet(request: &HttpRequest) -> HttpResponse {
///     format!(
///         "Hello, {}!",
///         request.query_params().get("name").map_or("stranger", |x| x)
///     ).into()
/// }
/// ```
///
/// Invoking the above function as `https://<app-name>.azurewebsites.net/api/greet?name=John`
/// would result in a response of `Hello, John!`.
#[derive(Debug)]
pub struct HttpRequest<'a>(&'a protocol::RpcHttp);

impl<'a> HttpRequest<'a> {
    /// Gets the HTTP method (e.g. "GET") for the request.
    pub fn method(&self) -> &str {
        &self.0.method
    }

    /// Gets the URL of the request.
    pub fn url(&self) -> &str {
        &self.0.url
    }

    /// Gets the headers of the request.
    pub fn headers(&self) -> &HashMap<String, String> {
        &self.0.headers
    }

    /// Gets the route parameters of the request.
    ///
    /// Route parameters are specified through the `route` argument of a `HttpRequest` binding attribute.
    ///
    /// See [Route Containts](https://docs.microsoft.com/en-us/aspnet/web-api/overview/web-api-routing-and-actions/attribute-routing-in-web-api-2#route-constraints) for syntax.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # #![feature(use_extern_macros)] extern crate azure_functions;
    /// use azure_functions::func;
    /// use azure_functions::bindings::{HttpRequest, HttpResponse};
    ///
    /// #[func]
    /// #[binding(name = "request", route = "users/{id:int}")]
    /// pub fn users(request: &HttpRequest) -> HttpResponse {
    ///     format!(
    ///         "User ID requested: {}",
    ///         request.route_params().get("id").unwrap()
    ///     ).into()
    /// }
    /// ```
    ///
    /// Invoking the above function as `https://<app-name>.azurewebsites.net/api/users/1234`
    /// would result in a response of `User ID requested: 1234`.
    pub fn route_params(&self) -> &HashMap<String, String> {
        &self.0.params
    }

    /// Gets the URL query parameters of the request.
    pub fn query_params(&self) -> &HashMap<String, String> {
        &self.0.query
    }

    /// Gets the body of the request.
    pub fn body(&self) -> Body {
        if self.0.has_body() {
            Body::from(self.0.get_body())
        } else {
            Body::Empty
        }
    }
}

impl<'a> From<&'a protocol::TypedData> for HttpRequest<'a> {
    fn from(data: &'a protocol::TypedData) -> Self {
        match data.data.as_ref().expect("expected type data") {
            protocol::TypedData_oneof_data::http(http) => HttpRequest(http),
            _ => panic!("unexpected type data for HTTP request."),
        }
    }
}
