use azure_functions::{
    bindings::{Blob, HttpRequest, HttpResponse},
    func,
    http::Status,
};

#[func]
#[binding(name = "req", route = "create/blob/{container}/{name}")]
#[binding(name = "output1", path = "{container}/{name}")]
pub fn create_blob(req: &HttpRequest) -> (HttpResponse, Blob) {
    (
        HttpResponse::build()
            .status(Status::Created)
            .body("blob has been created.")
            .into(),
        req.body().as_bytes().into(),
    )
}
