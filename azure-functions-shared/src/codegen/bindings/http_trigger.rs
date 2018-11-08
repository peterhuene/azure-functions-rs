use serde::{ser::SerializeMap, Serialize, Serializer};
use std::borrow::Cow;

pub const HTTP_TRIGGER_TYPE: &str = "httpTrigger";

#[derive(Debug, Clone)]
pub struct HttpTrigger {
    pub name: Cow<'static, str>,
    pub auth_level: Option<Cow<'static, str>>,
    pub methods: Cow<'static, [Cow<'static, str>]>,
    pub route: Option<Cow<'static, str>>,
    pub web_hook_type: Option<Cow<'static, str>>,
}

// TODO: when https://github.com/serde-rs/serde/issues/760 is resolved, remove implementation in favor of custom Serialize derive
// The fix would allow us to set the constant `type` and `direction` entries rather than having to emit them manually.
impl Serialize for HttpTrigger {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(None)?;

        map.serialize_entry("name", &self.name)?;
        map.serialize_entry("type", HTTP_TRIGGER_TYPE)?;
        map.serialize_entry("direction", "in")?;

        if let Some(auth_level) = self.auth_level.as_ref() {
            map.serialize_entry("authLevel", auth_level)?;
        }
        if !self.methods.is_empty() {
            map.serialize_entry("methods", &self.methods)?;
        }
        if let Some(route) = self.route.as_ref() {
            map.serialize_entry("route", route)?;
        }
        if let Some(web_hook_type) = self.web_hook_type.as_ref() {
            map.serialize_entry("webHookType", web_hook_type)?;
        }

        map.end()
    }
}
