use serde::{ser::SerializeMap, Serialize, Serializer};
use std::borrow::Cow;

pub const HTTP_TYPE: &str = "http";

#[derive(Debug, Clone)]
pub struct Http {
    pub name: Cow<'static, str>,
}

// TODO: when https://github.com/serde-rs/serde/issues/760 is resolved, remove implementation in favor of custom Serialize derive
// The fix would allow us to set the constant `type` and `direction` entries rather than having to emit them manually.
impl Serialize for Http {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(None)?;

        map.serialize_entry("name", &self.name)?;
        map.serialize_entry("type", HTTP_TYPE)?;
        map.serialize_entry("direction", "out")?;

        map.end()
    }
}
