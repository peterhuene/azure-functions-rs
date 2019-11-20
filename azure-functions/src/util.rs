use crate::rpc::{typed_data::Data, TypedData};
use chrono::{DateTime, FixedOffset, Utc};
use serde::{
    de::{
        value::{F64Deserializer, I64Deserializer},
        Error, IntoDeserializer,
    },
    Deserialize, Deserializer,
};
use serde_json::{error, from_str};
use std::str::{from_utf8, FromStr};

pub fn convert_from<'a, T>(data: &'a TypedData) -> Option<T>
where
    T: FromStr + Deserialize<'a>,
{
    match &data.data {
        Some(Data::String(s)) => s.parse::<T>().ok(),
        Some(Data::Json(s)) => from_str(s).ok(),
        Some(Data::Bytes(b)) => {
            if let Ok(s) = from_utf8(b) {
                return s.parse::<T>().ok();
            }
            None
        }
        Some(Data::Stream(s)) => {
            if let Ok(s) = from_utf8(s) {
                return s.parse::<T>().ok();
            }
            None
        }
        Some(Data::Int(i)) => {
            let deserializer: I64Deserializer<error::Error> = i.into_deserializer();
            T::deserialize(deserializer).ok()
        }
        Some(Data::Double(d)) => {
            let deserializer: F64Deserializer<error::Error> = d.into_deserializer();
            T::deserialize(deserializer).ok()
        }
        _ => None,
    }
}

pub fn deserialize_datetime<'a, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
where
    D: Deserializer<'a>,
{
    let mut s = String::deserialize(deserializer)?;

    // This exists because the Azure Functions Host serializes DateTime.MinValue without a timezone
    // However, chrono::DateTime requires one for DateTime<Utc>
    if s == "0001-01-01T00:00:00" {
        s += "Z";
    }

    s.parse::<DateTime<FixedOffset>>()
        .map_err(|e| Error::custom(format!("{}", e)))
        .map(|dt| dt.with_timezone(&Utc))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_converts_from_string_data() {
        const DATA: &'static str = "test";

        let data = TypedData {
            data: Some(Data::String(DATA.to_string())),
        };

        let s: String = convert_from(&data).unwrap();
        assert_eq!(s, DATA);
    }

    #[test]
    fn it_converts_from_json_data() {
        let data = TypedData {
            data: Some(Data::Json(r#""hello world""#.to_string())),
        };

        let s: String = convert_from(&data).unwrap();
        assert_eq!(s, "hello world");
    }

    #[test]
    fn it_converts_from_bytes_data() {
        let data = TypedData {
            data: Some(Data::Bytes(vec![
                0x68, 0x65, 0x6C, 0x6C, 0x6F, 0x20, 0x77, 0x6F, 0x72, 0x6C, 0x64,
            ])),
        };

        let s: String = convert_from(&data).unwrap();
        assert_eq!(s, "hello world");
    }

    #[test]
    fn it_converts_from_stream_data() {
        let data = TypedData {
            data: Some(Data::Stream(vec![
                0x68, 0x65, 0x6C, 0x6C, 0x6F, 0x20, 0x77, 0x6F, 0x72, 0x6C, 0x64,
            ])),
        };

        let s: String = convert_from(&data).unwrap();
        assert_eq!(s, "hello world");
    }

    #[test]
    fn it_converts_from_int_data() {
        const DATA: i64 = 42;

        let data = TypedData {
            data: Some(Data::Int(DATA)),
        };

        let d: i64 = convert_from(&data).unwrap();
        assert_eq!(d, DATA);
    }

    #[test]
    fn it_converts_from_double_data() {
        const DATA: f64 = 42.24;

        let data = TypedData {
            data: Some(Data::Double(DATA)),
        };

        let d: f64 = convert_from(&data).unwrap();
        assert_eq!(d, DATA);
    }
}
