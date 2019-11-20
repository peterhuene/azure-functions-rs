use azure_functions::{
    bindings::{HttpRequest, HttpResponse},
    func,
};
use futures::future::ready;

#[func]
pub async fn greet_async(req: HttpRequest) -> HttpResponse {
    let response = format!(
        "Hello from Rust, {}!\n",
        req.query_params.get("name").map_or("stranger", |x| x)
    );

    // Use ready().await to simply demonstrate the async/await feature
    ready(response).await.into()
}
