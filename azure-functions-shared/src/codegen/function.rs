use codegen::Binding;
use rpc::protocol;
use serde::{ser::SerializeMap, Serialize, Serializer};
use std::borrow::Cow;

#[doc(hidden)]
#[derive(Clone)]
pub struct Function {
    pub name: Cow<'static, str>,
    pub disabled: bool,
    pub bindings: Cow<'static, [Binding]>,
    pub invoker_name: Option<Cow<'static, str>>,
    pub invoker: Option<fn(&str, &mut protocol::InvocationRequest) -> protocol::InvocationResponse>,
    pub manifest_dir: Option<Cow<'static, str>>,
    pub file: Option<Cow<'static, str>>,
}

// TODO: when https://github.com/serde-rs/serde/issues/760 is resolved, remove implementation in favor of custom Serialize derive
// The fix would allow us to set the constant `generatedBy` entry rather than having to emit them manually.
impl Serialize for Function {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(None)?;

        map.serialize_entry("generatedBy", "azure-functions-rs")?;
        map.serialize_entry("disabled", &self.disabled)?;
        map.serialize_entry("bindings", &self.bindings)?;

        map.end()
    }
}
