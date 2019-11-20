use azure_functions::{
    bindings::{HttpRequest, HttpResponse},
    func,
};

#[func]
pub fn greet(req: HttpRequest) -> HttpResponse {
    format!(
        "Hello from Rust, {}!\n",
        req.query_params.get("name").map_or("stranger", |x| x)
    )
    .into()
}
