use crate::codegen::Direction;
use serde::{ser::SerializeMap, Serialize, Serializer};
use std::borrow::Cow;

pub const BLOB_TYPE: &str = "blob";

#[derive(Debug, Clone)]
pub struct Blob {
    pub name: Cow<'static, str>,
    pub path: Cow<'static, str>,
    pub connection: Option<Cow<'static, str>>,
    pub direction: Direction,
}

// TODO: when https://github.com/serde-rs/serde/issues/760 is resolved, remove implementation in favor of custom Serialize derive
// The fix would allow us to set the constant `type` entry rather than having to emit it manually.
impl Serialize for Blob {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(None)?;

        map.serialize_entry("name", &self.name)?;
        map.serialize_entry("type", BLOB_TYPE)?;
        map.serialize_entry("direction", &self.direction)?;
        map.serialize_entry("path", &self.path)?;

        if let Some(connection) = self.connection.as_ref() {
            map.serialize_entry("connection", connection)?;
        }

        map.end()
    }
}
