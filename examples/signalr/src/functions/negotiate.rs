use azure_functions::{
    bindings::{HttpRequest, HttpResponse, SignalRConnectionInfo},
    func,
};

#[func]
#[binding(name = "_req", auth_level = "anonymous")]
#[binding(
    name = "info",
    hub_name = "simplechat",
    user_id = "{headers.x-ms-signalr-userid}",
    connection = "connection"
)]
pub fn negotiate(_req: HttpRequest, info: SignalRConnectionInfo) -> HttpResponse {
    info.into()
}
