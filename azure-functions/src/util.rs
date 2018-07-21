use rpc::protocol;
use serde::de::IntoDeserializer;
use serde::Deserialize;
use serde_json::from_str;
use std::str::{from_utf8, FromStr};

pub fn convert_from<'a, T>(data: &'a protocol::TypedData) -> Result<T, ()>
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
