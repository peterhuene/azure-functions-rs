use rpc::protocol;
use serde::de::IntoDeserializer;
use serde::Deserialize;
use serde_json::from_str;
use std::str::{from_utf8, FromStr};

pub fn convert_from<T>(data: &'a protocol::TypedData) -> Result<T, ()>
where
    T: FromStr + Deserialize<'a>,
{
    if data.has_string() {
        return data.get_string().parse::<T>().map_err(|_| ());
    }

    if data.has_json() {
        return from_str(data.get_json()).map_err(|_| ());
    }

    if data.has_bytes() {
        let s = from_utf8(data.get_bytes()).map_err(|_| ())?;
        return s.parse::<T>().map_err(|_| ());
    }

    if data.has_stream() {
        let s = from_utf8(data.get_stream()).map_err(|_| ())?;
        return s.parse::<T>().map_err(|_| ());
    }

    if data.has_int() {
        let deserializer: ::serde::de::value::I64Deserializer<::serde_json::error::Error> =
            data.get_int().into_deserializer();
        return T::deserialize(deserializer).map_err(|_| ());
    }

    if data.has_double() {
        let deserializer: ::serde::de::value::F64Deserializer<::serde_json::error::Error> =
            data.get_double().into_deserializer();
        return T::deserialize(deserializer).map_err(|_| ());
    }

    Err(())
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
