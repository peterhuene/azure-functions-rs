use crate::rpc::{typed_data::Data, TypedData};
use serde::Deserialize;
use serde_json::{from_str, Number, Value};
use std::collections::HashMap;

const INSTANCE_ID_KEY: &str = "instanceId";

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
/// An activity that outputs a string:
///
/// ```rust
/// use azure_functions::{bindings::DurableActivityContext, durable::ActivityOutput, func};
///
/// #[func]
/// pub fn say_hello(context: DurableActivityContext) -> ActivityOutput {
///     format!(
///         "Hello {}!",
///         context.input.as_str().expect("expected a string input")
///     )
///     .into()
/// }
/// ```
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DurableActivityContext {
    /// The input to the activity function.
    pub input: Value,
    /// The orchestration instance identifier.
    pub instance_id: String,
}

impl DurableActivityContext {
    #[doc(hidden)]
    pub fn new(data: TypedData, mut metadata: HashMap<String, TypedData>) -> Self {
        Self {
            input: match data.data {
                Some(Data::String(s)) => Value::String(s),
                Some(Data::Json(s)) => from_str(&s).unwrap_or_default(),
                Some(Data::Int(i)) => Value::Number(i.into()),
                Some(Data::Double(d)) => Value::Number(Number::from_f64(d).unwrap()),
                _ => Value::Null,
            },
            instance_id: metadata
                .remove(INSTANCE_ID_KEY)
                .map(|data| match data.data {
                    Some(Data::String(s)) => s,
                    _ => panic!("expected a string for instance id"),
                })
                .expect("expected an instance id"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rpc::typed_data::Data;

    #[test]
    fn it_constructs() {
        let data = TypedData {
            data: Some(Data::String("bar".to_string())),
        };

        let mut metadata = HashMap::new();
        metadata.insert(
            INSTANCE_ID_KEY.to_string(),
            TypedData {
                data: Some(Data::String("foo".to_string())),
            },
        );

        let context = DurableActivityContext::new(data, metadata);
        assert_eq!(context.instance_id, "foo");
        assert_eq!(context.input, Value::String("bar".to_string()));
    }
}
