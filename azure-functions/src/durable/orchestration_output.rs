use crate::rpc::{typed_data::Data, TypedData};
use serde_json::Value;

/// # Orchestration Output
/// 
/// Type returned by Orchetrator Functions for Durable Functions

struct OrchestrationOutput(serde_json::Value);

impl<T> From<T> for OrchestrationOutput
where 
    T: Into<Value>,
{
    fn from(t: T) -> Self {
        OrchestrationOutput(t.into())
    }
}

#[doc(hidden)]
impl Into<TypedData> for OrchestrationOutput {
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
        let orchestration_output: OrchestrationOutput = json!({"foo": "bar"}).into();

        let data: TypedData = activity_output.into();
        assert_eq!(data.data, Some(Data::Json(r#"{"foo":"bar"}"#.to_string())));
    }

    #[test]
    fn it_converts_from_bool() {
        let orchestration_output: OrchestrationOutput = true.into();

        let data: TypedData = Orchestration_output.into();
        assert_eq!(data.data, Some(Data::Json(true.to_string())));
    }

    #[test]
    fn it_converts_from_string() {
        let orchestration_output: OrchestrationOutput = "foo".into();

        let data: TypedData = orchestration_output.into();
        assert_eq!(data.data, Some(Data::Json("\"foo\"".to_string())));
    }
}