use crate::{
    http::Body,
    rpc::{typed_data::Data, RpcHttp, TypedData},
};
use std::collections::HashMap;

/// Represents a HTTP trigger binding.
///
/// The following binding attributes are supported:
///
/// | Name         | Description                                                                                                                                                                                       |
/// |--------------|---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
/// | `name`       | The name of the parameter being bound.                                                                                                                                                            |
/// | `auth_level` | Determines what keys, if any, need to be present on the request in order to invoke the function. The authorization level can be one of the following values: `anonymous`, `function`, or `admin`. |
/// | `methods`    | A list of HTTP methods to which the function responds, separated by <code>&#124;</code> (e.g. <code>get&#124;post</code>). If not specified, the function responds to all HTTP methods.           |
/// | `route`      | The URL route to which the function responds. The default value is the name of the function.                                                                                                      |
///
/// # Examples
///
/// A function that responds with a friendly greeting:
///
/// ```rust
/// use azure_functions::{
///     bindings::{HttpRequest, HttpResponse},
///     func,
/// };
///
/// #[func]
/// pub fn greet(request: HttpRequest) -> HttpResponse {
///     format!(
///         "Hello, {}!",
///         request.query_params().get("name").map_or("stranger", |x| x)
///     ).into()
/// }
/// ```
#[derive(Debug)]
pub struct HttpRequest(RpcHttp);

impl HttpRequest {
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
    /// use azure_functions::func;
    /// use azure_functions::bindings::{HttpRequest, HttpResponse};
    ///
    /// #[func]
    /// #[binding(name = "request", route = "users/{id:int}")]
    /// pub fn users(request: HttpRequest) -> HttpResponse {
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
    /// use azure_functions::func;
    /// use azure_functions::bindings::{HttpRequest, HttpResponse};
    ///
    /// #[func]
    /// pub fn users(request: HttpRequest) -> HttpResponse {
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
        self.0
            .body
            .as_ref()
            .map(|b| Body::from(&**b))
            .unwrap_or(Body::Empty)
    }
}

impl HttpRequest {
    #[doc(hidden)]
    pub fn new(data: TypedData, _: &mut HashMap<String, TypedData>) -> Self {
        match data.data {
            Some(Data::Http(http)) => HttpRequest(*http),
            _ => panic!("unexpected type data for HTTP request."),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use matches::matches;
    use std::borrow::Cow;

    #[test]
    fn it_has_the_method() {
        const METHOD: &'static str = "GET";

        let mut http = RpcHttp::default();
        http.method = METHOD.to_string();

        let data = TypedData {
            data: Some(Data::Http(Box::new(http))),
        };

        let mut metadata = HashMap::new();

        let request = HttpRequest::new(data, &mut metadata);
        assert_eq!(request.method(), METHOD);
    }

    #[test]
    fn it_has_the_url() {
        const URL: &'static str = "http://example.com";

        let mut http = RpcHttp::default();
        http.url = URL.to_string();

        let data = TypedData {
            data: Some(Data::Http(Box::new(http))),
        };

        let mut metadata = HashMap::new();

        let request = HttpRequest::new(data, &mut metadata);
        assert_eq!(request.url(), URL);
    }

    #[test]
    fn it_has_a_header() {
        const KEY: &'static str = "Accept";
        const VALUE: &'static str = "application/json";

        let mut http = RpcHttp::default();
        http.headers.insert(KEY.to_string(), VALUE.to_string());

        let data = TypedData {
            data: Some(Data::Http(Box::new(http))),
        };

        let mut metadata = HashMap::new();

        let request = HttpRequest::new(data, &mut metadata);
        assert_eq!(request.headers().get(KEY).unwrap(), VALUE);
    }

    #[test]
    fn it_has_a_route_parameter() {
        const KEY: &'static str = "id";
        const VALUE: &'static str = "12345";

        let mut http = RpcHttp::default();
        http.params.insert(KEY.to_string(), VALUE.to_string());

        let data = TypedData {
            data: Some(Data::Http(Box::new(http))),
        };

        let mut metadata = HashMap::new();

        let request = HttpRequest::new(data, &mut metadata);
        assert_eq!(request.route_params().get(KEY).unwrap(), VALUE);
    }

    #[test]
    fn it_has_a_query_parameter() {
        const KEY: &'static str = "name";
        const VALUE: &'static str = "Peter";

        let mut http = RpcHttp::default();
        http.query.insert(KEY.to_string(), VALUE.to_string());

        let data = TypedData {
            data: Some(Data::Http(Box::new(http))),
        };

        let mut metadata = HashMap::new();

        let request = HttpRequest::new(data, &mut metadata);
        assert_eq!(request.query_params().get(KEY).unwrap(), VALUE);
    }

    #[test]
    fn it_has_an_empty_body() {
        let data = TypedData {
            data: Some(Data::Http(Box::new(RpcHttp::default()))),
        };

        let mut metadata = HashMap::new();

        let request = HttpRequest::new(data, &mut metadata);
        assert!(matches!(request.body(), Body::Empty));
    }

    #[test]
    fn it_has_a_string_body() {
        const BODY: &'static str = "TEXT BODY";

        let mut http = RpcHttp::default();
        http.body = Some(Box::new(TypedData {
            data: Some(Data::String(BODY.to_string())),
        }));

        let data = TypedData {
            data: Some(Data::Http(Box::new(http))),
        };

        let mut metadata = HashMap::new();

        let request = HttpRequest::new(data, &mut metadata);
        assert!(matches!(request.body(), Body::String(Cow::Borrowed(BODY))));
    }

    #[test]
    fn it_has_a_json_body() {
        const BODY: &'static str = r#"{ "json": "body" }"#;

        let mut http = RpcHttp::default();
        http.body = Some(Box::new(TypedData {
            data: Some(Data::Json(BODY.to_string())),
        }));

        let data = TypedData {
            data: Some(Data::Http(Box::new(http))),
        };

        let mut metadata = HashMap::new();

        let request = HttpRequest::new(data, &mut metadata);
        assert!(matches!(request.body(), Body::Json(Cow::Borrowed(BODY))));
    }

    #[test]
    fn it_has_a_bytes_body() {
        const BODY: &'static [u8] = &[0, 1, 2];

        let mut http = RpcHttp::default();
        http.body = Some(Box::new(TypedData {
            data: Some(Data::Bytes(BODY.to_vec())),
        }));

        let data = TypedData {
            data: Some(Data::Http(Box::new(http))),
        };

        let mut metadata = HashMap::new();

        let request = HttpRequest::new(data, &mut metadata);
        assert!(matches!(request.body(), Body::Bytes(Cow::Borrowed(BODY))));
    }

    #[test]
    fn it_has_a_stream_body() {
        const BODY: &'static [u8] = &[0, 1, 2];

        let mut http = RpcHttp::default();
        http.body = Some(Box::new(TypedData {
            data: Some(Data::Stream(BODY.to_vec())),
        }));

        let data = TypedData {
            data: Some(Data::Http(Box::new(http))),
        };

        let mut metadata = HashMap::new();

        let request = HttpRequest::new(data, &mut metadata);
        assert!(matches!(request.body(), Body::Bytes(Cow::Borrowed(BODY))));
    }
}
