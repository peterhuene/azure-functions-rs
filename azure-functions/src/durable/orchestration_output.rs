use crate::durable::IntoValue;
use serde_json::Value;
use std::iter::FromIterator;

/// Represents the output of a Durable Functions orchestration function.
///
/// Supports conversion from JSON-compatible types.
pub struct OrchestrationOutput(Value);

impl<T> From<T> for OrchestrationOutput
where
    T: Into<Value>,
{
    fn from(t: T) -> Self {
        Self(t.into())
    }
}

impl FromIterator<Value> for OrchestrationOutput {
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = Value>,
    {
        Self(Value::from_iter(iter))
    }
}

impl IntoValue for OrchestrationOutput {
    fn into_value(self) -> Value {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn it_converts_from_json() {
        let orchestration_output: OrchestrationOutput = json!({"foo": "bar"}).into();

        let data: Value = orchestration_output.into_value();
        assert_eq!(data, json!({"foo": "bar"}));
    }

    #[test]
    fn it_converts_from_bool() {
        let orchestration_output: OrchestrationOutput = true.into();

        let data: Value = orchestration_output.into_value();
        assert_eq!(data, json!(true));
    }

    #[test]
    fn it_converts_from_string() {
        let orchestration_output: OrchestrationOutput = "foo".into();

        let data: Value = orchestration_output.into_value();
        assert_eq!(data, json!("foo"));
    }
}
