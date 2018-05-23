use codegen::bindings;

#[doc(hidden)]
#[derive(Serialize)]
#[serde(untagged, rename_all = "camelCase")]
pub enum Binding {
    HttpTrigger(bindings::HttpTrigger),
    Http(bindings::Http),
    TimerTrigger(bindings::TimerTrigger),
}
