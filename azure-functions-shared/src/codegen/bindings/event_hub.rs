use serde::{ser::SerializeMap, Serialize, Serializer};
use std::borrow::Cow;

pub const EVENT_HUB_TYPE: &str = "eventHub";

#[derive(Debug, Clone)]
pub struct EventHub {
    pub name: Cow<'static, str>,
    pub connection: Cow<'static, str>,
    pub event_hub_name: Option<Cow<'static, str>>,
}

// TODO: when https://github.com/serde-rs/serde/issues/760 is resolved, remove implementation in favor of custom Serialize derive
// The fix would allow us to set the constant `type` and `direction` entries rather than having to emit them manually.
impl Serialize for EventHub {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(None)?;

        map.serialize_entry("name", &self.name)?;
        map.serialize_entry("type", EVENT_HUB_TYPE)?;
        map.serialize_entry("direction", "out")?;
        map.serialize_entry("connection", &self.connection)?;

        if let Some(event_hub_name) = self.event_hub_name.as_ref() {
            map.serialize_entry("eventHubName", event_hub_name)?;
        }

        map.end()
    }
}
