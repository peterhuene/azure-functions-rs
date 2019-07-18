use crate::rpc::TypedData;
use serde_derive::Deserialize;
use std::collections::HashMap;

/// Represents the Durable Functions activity context binding.
///
/// The following binding attributes are supported:
///
/// | Name       | Description                                                      |
/// |------------|------------------------------------------------------------------|
/// | `name`     | The name of the parameter being bound.                           |
/// | `activity` | The name of the activity.  Defaults to the name of the function. |
///
/// # Examples
///
/// TODO: IMPLEMENT
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DurableActivityContext {}

impl DurableActivityContext {
    #[doc(hidden)]
    pub fn new(data: TypedData, metadata: HashMap<String, TypedData>) -> Self {
        println!("{:#?}", data);
        println!("{:#?}", metadata);
        DurableActivityContext {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rpc::typed_data::Data;

    #[test]
    fn it_constructs() {
        let data = TypedData {
            data: Some(Data::String(r#"{ }"#.to_owned())),
        };

        let _ = DurableActivityContext::new(data, HashMap::new());

        // TODO: implement
    }
}
