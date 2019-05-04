//! Module for generic Azure Function bindings.
use crate::{
    http::Body,
    rpc::{typed_data::Data, TypedData},
};
use serde_json::from_str;
use std::borrow::Cow;

/// Represents a value passed to or from a generic binding.
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    /// No data is present.
    None,
    /// The value is a string.
    String(String),
    /// The value is a JSON value.
    Json(serde_json::Value),
    /// The value is a sequence of bytes.
    Bytes(Vec<u8>),
    /// The value is an integer.
    Integer(i64),
    /// The value is a double.
    Double(f64),
}

impl<'a> Into<Body<'a>> for Value {
    fn into(self) -> Body<'a> {
        match self {
            Value::None => Body::Empty,
            Value::String(s) => Body::String(Cow::Owned(s)),
            Value::Json(v) => Body::Json(Cow::Owned(v.to_string())),
            Value::Bytes(b) => Body::Bytes(Cow::Owned(b)),
            Value::Integer(i) => Body::String(Cow::Owned(i.to_string())),
            Value::Double(d) => Body::String(Cow::Owned(d.to_string())),
        }
    }
}

#[doc(hidden)]
impl From<TypedData> for Value {
    fn from(data: TypedData) -> Self {
        match data.data {
            None => Value::None,
            Some(Data::String(s)) => Value::String(s),
            Some(Data::Json(s)) => Value::Json(from_str(&s).unwrap()),
            Some(Data::Bytes(b)) => Value::Bytes(b),
            Some(Data::Stream(b)) => Value::Bytes(b),
            Some(Data::Http(_)) => panic!("generic bindings cannot contain HTTP data"),
            Some(Data::Int(i)) => Value::Integer(i),
            Some(Data::Double(d)) => Value::Double(d),
        }
    }
}

#[doc(hidden)]
impl Into<TypedData> for Value {
    fn into(self) -> TypedData {
        match self {
            Value::None => TypedData { data: None },
            Value::String(s) => TypedData {
                data: Some(Data::String(s)),
            },
            Value::Json(v) => TypedData {
                data: Some(Data::Json(v.to_string())),
            },
            Value::Bytes(b) => TypedData {
                data: Some(Data::Bytes(b)),
            },
            Value::Integer(i) => TypedData {
                data: Some(Data::Int(i)),
            },
            Value::Double(d) => TypedData {
                data: Some(Data::Double(d)),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rpc::typed_data::Data;
    use serde_json::json;

    #[test]
    fn it_converts_from_no_typed_data() {
        let value: Value = TypedData { data: None }.into();

        assert_eq!(value, Value::None);
    }

    #[test]
    fn it_converts_from_string_typed_data() {
        let value: Value = TypedData {
            data: Some(Data::String("hello world".into())),
        }
        .into();

        assert_eq!(value, Value::String("hello world".into()));
    }

    #[test]
    fn it_converts_from_json_typed_data() {
        let value: Value = TypedData {
            data: Some(Data::Json(r#"{ "foo": "bar" }"#.to_string())),
        }
        .into();

        assert_eq!(value, Value::Json(json!({ "foo": "bar" })));
    }

    #[test]
    fn it_converts_from_bytes_typed_data() {
        let value: Value = TypedData {
            data: Some(Data::Bytes(vec![1, 2, 3])),
        }
        .into();

        assert_eq!(value, Value::Bytes(vec![1, 2, 3]));
    }

    #[test]
    fn it_converts_from_stream_typed_data() {
        let value: Value = TypedData {
            data: Some(Data::Stream(vec![1, 2, 3])),
        }
        .into();

        assert_eq!(value, Value::Bytes(vec![1, 2, 3]));
    }

    #[test]
    fn it_converts_from_integer_typed_data() {
        let value: Value = TypedData {
            data: Some(Data::Int(12345)),
        }
        .into();

        assert_eq!(value, Value::Integer(12345));
    }

    #[test]
    fn it_converts_from_double_typed_data() {
        let value: Value = TypedData {
            data: Some(Data::Double(12345.6)),
        }
        .into();

        assert_eq!(value, Value::Double(12345.6));
    }

    #[test]
    fn it_converts_to_no_typed_data() {
        let data: TypedData = Value::None.into();

        assert_eq!(data.data, None);
    }

    #[test]
    fn it_converts_to_string_typed_data() {
        let data: TypedData = Value::String("hello world".to_owned()).into();

        assert_eq!(data.data, Some(Data::String("hello world".to_owned())));
    }

    #[test]
    fn it_converts_to_json_typed_data() {
        let data: TypedData = Value::Json(json!({ "foo": "bar"})).into();

        assert_eq!(data.data, Some(Data::Json(r#"{"foo":"bar"}"#.to_string())));
    }

    #[test]
    fn it_converts_to_bytes_typed_data() {
        let data: TypedData = Value::Bytes(vec![1, 2, 3]).into();

        assert_eq!(data.data, Some(Data::Bytes(vec![1, 2, 3])));
    }

    #[test]
    fn it_converts_to_integer_typed_data() {
        let data: TypedData = Value::Integer(12345).into();

        assert_eq!(data.data, Some(Data::Int(12345)));
    }

    #[test]
    fn it_converts_to_double_typed_data() {
        let data: TypedData = Value::Double(12345.6).into();

        assert_eq!(data.data, Some(Data::Double(12345.6)));
    }
}
