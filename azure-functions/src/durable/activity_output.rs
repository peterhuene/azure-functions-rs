use crate::rpc::{typed_data::Data, TypedData};
use serde_json::Value;
use std::iter::FromIterator;

/// Represents the output of a Durable Functions activity function.
///
/// Supports conversion from JSON-compatible types.
pub struct ActivityOutput(Value);

impl<T> From<T> for ActivityOutput
where
    T: Into<Value>,
{
    fn from(t: T) -> Self {
        ActivityOutput(t.into())
    }
}

impl FromIterator<Value> for ActivityOutput {
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = Value>,
    {
        ActivityOutput(Value::from_iter(iter))
    }
}

#[doc(hidden)]
impl Into<TypedData> for ActivityOutput {
    fn into(self) -> TypedData {
        TypedData {
            data: Some(Data::Json(self.0.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn it_converts_from_json() {
        let activity_output: ActivityOutput = json!({ "foo": "bar" }).into();

        let data: TypedData = activity_output.into();
        assert_eq!(data.data, Some(Data::Json(r#"{"foo":"bar"}"#.to_string())));
    }

    #[test]
    fn it_converts_from_bool() {
        let activity_output: ActivityOutput = true.into();

        let data: TypedData = activity_output.into();
        assert_eq!(data.data, Some(Data::Json(true.to_string())));
    }

    #[test]
    fn it_converts_from_string() {
        let activity_output: ActivityOutput = "foo".into();

        let data: TypedData = activity_output.into();
        assert_eq!(data.data, Some(Data::Json("\"foo\"".to_string())));
    }
}
