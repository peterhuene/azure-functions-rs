use crate::{
    http::{Body, Cookie},
    rpc::{typed_data::Data, TypedData},
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
///         request.query_params.get("name").map_or("stranger", |x| x)
///     ).into()
/// }
/// ```
#[derive(Debug)]
pub struct HttpRequest {
    /// The HTTP method (e.g. "GET") for the request.
    pub method: String,
    /// The URL of the request.
    pub url: String,
    /// The headers of the request.
    pub headers: HashMap<String, String>,
    /// The body of the request.
    pub body: Body,
    /// The route parameters of the request.
    pub route_params: HashMap<String, String>,
    /// The query parameters of the request.
    pub query_params: HashMap<String, String>,
    /// The cookies of the request.
    pub cookies: Vec<Cookie>,
}

impl HttpRequest {
    #[doc(hidden)]
    pub fn new(data: TypedData, _: HashMap<String, TypedData>) -> Self {
        match data.data {
            Some(Data::Http(http)) => Self {
                method: http.method,
                url: http.url,
                body: http.body.map_or("".into(), |b| Body::from(*b)),
                headers: http.headers,
                route_params: http.params,
                query_params: http.query,
                cookies: http.cookies.into_iter().map(|c| c.into()).collect(),
            },
            _ => panic!("unexpected type data for HTTP request."),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rpc::RpcHttp;
    #[test]
    fn it_has_a_method() {
        const METHOD: &'static str = "GET";

        let mut http = RpcHttp::default();
        http.method = METHOD.to_string();

        let data = TypedData {
            data: Some(Data::Http(Box::new(http))),
        };

        let request = HttpRequest::new(data, HashMap::new());
        assert_eq!(request.method, METHOD);
    }

    #[test]
    fn it_has_a_url() {
        const URL: &'static str = "http://example.com";

        let mut http = RpcHttp::default();
        http.url = URL.to_string();

        let data = TypedData {
            data: Some(Data::Http(Box::new(http))),
        };

        let request = HttpRequest::new(data, HashMap::new());
        assert_eq!(request.url, URL);
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

        let request = HttpRequest::new(data, HashMap::new());
        assert_eq!(request.headers.get(KEY).unwrap(), VALUE);
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

        let request = HttpRequest::new(data, HashMap::new());
        assert_eq!(request.route_params.get(KEY).unwrap(), VALUE);
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

        let request = HttpRequest::new(data, HashMap::new());
        assert_eq!(request.query_params.get(KEY).unwrap(), VALUE);
    }

    #[test]
    fn it_has_an_empty_body() {
        let data = TypedData {
            data: Some(Data::Http(Box::new(RpcHttp::default()))),
        };

        let request = HttpRequest::new(data, HashMap::new());
        assert_eq!(request.body.to_str().unwrap(), "");
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

        let request = HttpRequest::new(data, HashMap::new());
        assert_eq!(request.body.to_str().unwrap(), BODY);
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

        let request = HttpRequest::new(data, HashMap::new());
        assert_eq!(request.body.to_str().unwrap(), BODY);
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

        let request = HttpRequest::new(data, HashMap::new());
        assert_eq!(request.body.as_bytes(), BODY);
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

        let request = HttpRequest::new(data, HashMap::new());
        assert_eq!(request.body.as_bytes(), BODY);
    }
}
