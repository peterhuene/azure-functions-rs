use serde::{ser::SerializeMap, Serialize, Serializer};
use std::borrow::Cow;

#[derive(Debug, Clone)]
pub struct BlobTrigger {
    pub name: Cow<'static, str>,
    pub path: Cow<'static, str>,
    pub connection: Option<Cow<'static, str>>,
}

// TODO: when https://github.com/serde-rs/serde/issues/760 is resolved, remove implementation in favor of custom Serialize derive
// The fix would allow us to set the constant `type` and `direction` entries rather than having to emit them manually.
impl Serialize for BlobTrigger {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(None)?;

        map.serialize_entry("name", &self.name)?;
        map.serialize_entry("type", "blobTrigger")?;
        map.serialize_entry("direction", "in")?;
        map.serialize_entry("path", &self.path)?;

        if let Some(connection) = self.connection.as_ref() {
            map.serialize_entry("connection", connection)?;
        }

        map.end()
    }
}
