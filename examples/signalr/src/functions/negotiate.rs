use azure_functions::{
    bindings::{HttpRequest, HttpResponse, SignalRConnectionInfo},
    func,
};

#[func]
pub fn negotiate(
    #[binding(auth_level = "anonymous")] _req: HttpRequest,
    #[binding(
        hub_name = "simplechat",
        user_id = "{headers.x-ms-signalr-userid}",
        connection = "connection"
    )]
    info: SignalRConnectionInfo,
) -> HttpResponse {
    info.into()
}
