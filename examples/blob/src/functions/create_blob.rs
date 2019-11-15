use azure_functions::{
    bindings::{Blob, HttpRequest, HttpResponse},
    func,
    http::Status,
};

#[func]
#[binding(name = "output1", path = "{container}/{name}")]
pub fn create_blob(
    #[binding(route = "create/blob/{container}/{name}")] req: HttpRequest,
) -> (HttpResponse, Blob) {
    let data: Vec<u8> = req.body.into();
    (
        HttpResponse::build()
            .status(Status::Created)
            .body("blob has been created.")
            .finish(),
        data.into(),
    )
}
