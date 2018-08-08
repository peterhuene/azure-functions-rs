use codegen::Direction;
use serde::{ser::SerializeMap, Serialize, Serializer};
use std::borrow::Cow;

#[derive(Debug, Clone)]
pub struct Table {
    pub name: Cow<'static, str>,
    pub direction: Direction,
    pub table_name: Cow<'static, str>,
    pub partition_key: Option<Cow<'static, str>>,
    pub row_key: Option<Cow<'static, str>>,
    pub filter: Option<Cow<'static, str>>,
    pub take: Option<u64>,
    pub connection: Option<Cow<'static, str>>,
}

// TODO: when https://github.com/serde-rs/serde/issues/760 is resolved, remove implementation in favor of custom Serialize derive
// The fix would allow us to set the constant `type` entry rather than having to emit it manually.
impl Serialize for Table {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(None)?;

        map.serialize_entry("name", &self.name)?;
        map.serialize_entry("type", "table")?;
        map.serialize_entry("direction", &self.direction)?;
        map.serialize_entry("tableName", &self.table_name)?;

        if let Some(partition_key) = self.partition_key.as_ref() {
            map.serialize_entry("partitionKey", partition_key)?;
        }

        if let Some(row_key) = self.row_key.as_ref() {
            map.serialize_entry("rowKey", row_key)?;
        }

        if let Some(filter) = self.filter.as_ref() {
            map.serialize_entry("filter", filter)?;
        }

        if let Some(take) = self.take.as_ref() {
            map.serialize_entry("take", take)?;
        }

        if let Some(connection) = self.connection.as_ref() {
            map.serialize_entry("connection", connection)?;
        }

        map.end()
    }
}
