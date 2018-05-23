use serde::{ser::SerializeMap, Serialize, Serializer};

#[doc(hidden)]
pub struct HttpTrigger {
    pub name: &'static str,
    pub auth_level: &'static Option<&'static str>,
    pub methods: &'static [&'static str],
    pub route: &'static Option<&'static str>,
    pub web_hook_type: &'static Option<&'static str>,
}

// TODO: when https://github.com/serde-rs/serde/issues/760 is resolved, remove implementation in favor of custom Serialize derive
// The fix would allow us to set the constant `type` and `direction` entries rather than having to emit them manually.
impl Serialize for HttpTrigger {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(None)?;

        map.serialize_entry("name", self.name)?;
        map.serialize_entry("type", "httpTrigger")?;
        map.serialize_entry("direction", "in")?;
        if self.auth_level.is_some() {
            map.serialize_entry("authLevel", self.auth_level.as_ref().unwrap())?;
        }
        if !self.methods.is_empty() {
            map.serialize_entry("methods", self.methods)?;
        }
        if self.route.is_some() {
            map.serialize_entry("route", self.route.as_ref().unwrap())?;
        }
        if self.web_hook_type.is_some() {
            map.serialize_entry("webHookType", self.web_hook_type.as_ref().unwrap())?;
        }

        map.end()
    }
}
