use crate::rpc::protocol;
use chrono::{DateTime, FixedOffset, Utc};
use serde::{de::Error, de::IntoDeserializer, Deserialize, Deserializer};
use serde_json::from_str;
use std::str::{from_utf8, FromStr};

pub fn convert_from<'a, T>(data: &'a protocol::TypedData) -> Option<T>
where
    T: FromStr + Deserialize<'a>,
{
    if data.has_string() {
        return data.get_string().parse::<T>().ok();
    }

    if data.has_json() {
        return from_str(data.get_json()).ok();
    }

    if data.has_bytes() {
        if let Ok(s) = from_utf8(data.get_bytes()) {
            return s.parse::<T>().ok();
        }
        return None;
    }

    if data.has_stream() {
        if let Ok(s) = from_utf8(data.get_stream()) {
            return s.parse::<T>().ok();
        }
        return None;
    }

    if data.has_int() {
        let deserializer: ::serde::de::value::I64Deserializer<::serde_json::error::Error> =
            data.get_int().into_deserializer();
        return T::deserialize(deserializer).ok();
    }

    if data.has_double() {
        let deserializer: ::serde::de::value::F64Deserializer<::serde_json::error::Error> =
            data.get_double().into_deserializer();
        return T::deserialize(deserializer).ok();
    }

    None
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

        let mut data = protocol::TypedData::new();
        data.set_string(DATA.to_string());

        let s: String = convert_from(&data).unwrap();
        assert_eq!(s, DATA);
    }

    #[test]
    fn it_converts_from_json_data() {
        let mut data = protocol::TypedData::new();
        data.set_json(r#""hello world""#.to_string());

        let s: String = convert_from(&data).unwrap();
        assert_eq!(s, "hello world");
    }

    #[test]
    fn it_converts_from_bytes_data() {
        let mut data = protocol::TypedData::new();
        data.set_bytes(vec![
            0x68, 0x65, 0x6C, 0x6C, 0x6F, 0x20, 0x77, 0x6F, 0x72, 0x6C, 0x64,
        ]);

        let s: String = convert_from(&data).unwrap();
        assert_eq!(s, "hello world");
    }

    #[test]
    fn it_converts_from_stream_data() {
        let mut data = protocol::TypedData::new();
        data.set_stream(vec![
            0x68, 0x65, 0x6C, 0x6C, 0x6F, 0x20, 0x77, 0x6F, 0x72, 0x6C, 0x64,
        ]);

        let s: String = convert_from(&data).unwrap();
        assert_eq!(s, "hello world");
    }

    #[test]
    fn it_converts_from_int_data() {
        const DATA: i64 = 42;

        let mut data = protocol::TypedData::new();
        data.set_int(DATA);

        let d: i64 = convert_from(&data).unwrap();
        assert_eq!(d, DATA);
    }

    #[test]
    fn it_converts_from_double_data() {
        const DATA: f64 = 42.24;

        let mut data = protocol::TypedData::new();
        data.set_double(DATA);

        let d: f64 = convert_from(&data).unwrap();
        assert_eq!(d, DATA);
    }
}
