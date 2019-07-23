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
