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

impl OrchestrationOutput {
    pub(crate) fn to_value(self) -> Value {
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

        let data: Value = orchestration_output.to_value();
        assert_eq!(data, json!({"foo": "bar"}));
    }

    #[test]
    fn it_converts_from_bool() {
        let orchestration_output: OrchestrationOutput = true.into();

        let data: Value = orchestration_output.to_value();
        assert_eq!(data, json!(true));
    }

    #[test]
    fn it_converts_from_string() {
        let orchestration_output: OrchestrationOutput = "foo".into();

        let data: Value = orchestration_output.to_value();
        assert_eq!(data, json!("\"foo\"".to_string()));
    }
}
