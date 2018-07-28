use bindings::Trigger;
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
#[derive(Debug)]
pub struct HttpRequest<'a>(&'a protocol::RpcHttp);

impl HttpRequest<'_> {
    /// Gets the HTTP method (e.g. "GET") for the request.
    pub fn method(&self) -> &str {
        &self.0.method
    }

    /// Gets the URL of the request.
    pub fn url(&self) -> &str {
        &self.0.url
    }

    /// Gets the headers of the request.
    ///
    /// The header keys are lower-cased.
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

    /// Gets the query parameters of the request.
    ///
    /// The query parameter keys are case-sensative.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # #![feature(use_extern_macros)] extern crate azure_functions;
    /// use azure_functions::func;
    /// use azure_functions::bindings::{HttpRequest, HttpResponse};
    ///
    /// #[func]
    /// pub fn users(request: &HttpRequest) -> HttpResponse {
    ///     format!(
    ///         "The 'name' query parameter is: {}",
    ///         request.query_params().get("name").map_or("undefined", |x| x)
    ///     ).into()
    /// }
    /// ```
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

impl From<&'a protocol::TypedData> for HttpRequest<'a> {
    fn from(data: &'a protocol::TypedData) -> Self {
        if !data.has_http() {
            panic!("unexpected type data for HTTP request.");
        }
        HttpRequest(data.get_http())
    }
}

impl Trigger<'a> for HttpRequest<'a> {
    fn read_metadata(&mut self, _: &'a HashMap<String, protocol::TypedData>) {}
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::borrow::Cow;

    #[test]
    fn it_has_the_method() {
        const METHOD: &'static str = "GET";

        let mut data = protocol::TypedData::new();
        let mut http = protocol::RpcHttp::new();
        http.set_method(METHOD.to_string());
        data.set_http(http);

        let request: HttpRequest = (&data).into();
        assert_eq!(request.method(), METHOD);
    }

    #[test]
    fn it_has_the_url() {
        const URL: &'static str = "http://example.com";

        let mut data = protocol::TypedData::new();
        let mut http = protocol::RpcHttp::new();
        http.set_url(URL.to_string());
        data.set_http(http);

        let request: HttpRequest = (&data).into();
        assert_eq!(request.url(), URL);
    }

    #[test]
    fn it_has_a_header() {
        const KEY: &'static str = "Accept";
        const VALUE: &'static str = "application/json";

        let mut data = protocol::TypedData::new();
        let mut http = protocol::RpcHttp::new();
        let mut headers = HashMap::new();
        headers.insert(KEY.to_string(), VALUE.to_string());
        http.set_headers(headers);
        data.set_http(http);

        let request: HttpRequest = (&data).into();
        assert_eq!(request.headers().get(KEY).unwrap(), VALUE);
    }

    #[test]
    fn it_has_a_route_parameter() {
        const KEY: &'static str = "id";
        const VALUE: &'static str = "12345";

        let mut data = protocol::TypedData::new();
        let mut http = protocol::RpcHttp::new();
        let mut params = HashMap::new();
        params.insert(KEY.to_string(), VALUE.to_string());
        http.set_params(params);
        data.set_http(http);

        let request: HttpRequest = (&data).into();
        assert_eq!(request.route_params().get(KEY).unwrap(), VALUE);
    }

    #[test]
    fn it_has_a_query_parameter() {
        const KEY: &'static str = "name";
        const VALUE: &'static str = "Peter";

        let mut data = protocol::TypedData::new();
        let mut http = protocol::RpcHttp::new();
        let mut params = HashMap::new();
        params.insert(KEY.to_string(), VALUE.to_string());
        http.set_query(params);
        data.set_http(http);

        let request: HttpRequest = (&data).into();
        assert_eq!(request.query_params().get(KEY).unwrap(), VALUE);
    }

    #[test]
    fn it_has_an_empty_body() {
        let mut data = protocol::TypedData::new();
        let http = protocol::RpcHttp::new();

        data.set_http(http);

        let request: HttpRequest = (&data).into();
        assert!(matches!(request.body(), Body::Empty));
    }

    #[test]
    fn it_has_a_string_body() {
        const BODY: &'static str = "TEXT BODY";

        let mut data = protocol::TypedData::new();
        let mut http = protocol::RpcHttp::new();
        let mut body = protocol::TypedData::new();

        body.set_string(BODY.to_string());
        http.set_body(body);
        data.set_http(http);

        let request: HttpRequest = (&data).into();
        assert!(matches!(request.body(), Body::String(Cow::Borrowed(BODY))));
    }

    #[test]
    fn it_has_a_json_body() {
        const BODY: &'static str = r#"{ "json": "body" }"#;

        let mut data = protocol::TypedData::new();
        let mut http = protocol::RpcHttp::new();
        let mut body = protocol::TypedData::new();

        body.set_json(BODY.to_string());
        http.set_body(body);
        data.set_http(http);

        let request: HttpRequest = (&data).into();
        assert!(matches!(request.body(), Body::Json(Cow::Borrowed(BODY))));
    }

    #[test]
    fn it_has_a_bytes_body() {
        const BODY: &'static [u8] = &[0, 1, 2];

        let mut data = protocol::TypedData::new();
        let mut http = protocol::RpcHttp::new();
        let mut body = protocol::TypedData::new();

        body.set_bytes(BODY.to_owned());
        http.set_body(body);
        data.set_http(http);

        let request: HttpRequest = (&data).into();
        assert!(matches!(request.body(), Body::Bytes(Cow::Borrowed(BODY))));
    }

    #[test]
    fn it_has_a_stream_body() {
        const BODY: &'static [u8] = &[0, 1, 2];

        let mut data = protocol::TypedData::new();
        let mut http = protocol::RpcHttp::new();
        let mut body = protocol::TypedData::new();

        body.set_stream(BODY.to_owned());
        http.set_body(body);
        data.set_http(http);

        let request: HttpRequest = (&data).into();
        assert!(matches!(request.body(), Body::Bytes(Cow::Borrowed(BODY))));
    }
}
