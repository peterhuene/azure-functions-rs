use azure_functions::bindings::{HttpRequest, HttpResponse};
use azure_functions::func;

#[func]
#[binding(name = "req", auth_level = "anonymous")]
pub fn greet(req: &HttpRequest) -> HttpResponse {
    info!("Request: {:?}", req);

    format!(
        "Hello from Rust, {}!\n",
        req.query_params().get("name").map_or("stranger", |x| x)
    ).into()
}
