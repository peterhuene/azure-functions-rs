use crate::rpc::{typed_data::Data, TypedData};
use serde_json::Value;

/// # Activity Output
///
/// Type returned by Activity Functions for Durable Functions

struct ActivityOutput(serde_json::Value);

impl<T> From<T> for ActivityOutput
where
    T: Into<Value>,
{
    fn from(t: T) -> Self {
        ActivityOutput(t.into())
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
