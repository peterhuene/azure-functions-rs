use crate::rpc::{
    nullable_bool::Bool, nullable_double::Double, nullable_string::String as StringValue,
    nullable_timestamp::Timestamp, NullableBool, NullableDouble, NullableString, NullableTimestamp,
    RpcHttpCookie,
};
use chrono::{DateTime, NaiveDateTime, Utc};

/// The same site policy for HTTP cookies.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SameSitePolicy {
    /// Allow the cookie to be sent in cross-site browsing contexts for any methods.
    None = 0,
    /// Allow the cookie to be sent in cross-site browsing contexts, but block CSRF request methods.
    Lax = 1,
    /// Prevents the cookie from being sent in all cross-site browsing contexts.
    Strict = 2,
}

impl Default for SameSitePolicy {
    fn default() -> Self {
        Self::None
    }
}

/// Represents a HTTP cookie.
#[derive(Default, Clone, Debug)]
pub struct Cookie {
    /// The name of the cookie.
    pub name: String,
    /// The value of the cookie.
    pub value: String,
    /// The domain of the cookie.
    pub domain: Option<String>,
    /// The path of the cookie.
    pub path: Option<String>,
    /// The expiration time of the cookie.
    pub expires: Option<DateTime<Utc>>,
    /// Whether or not the cookie is secure.
    pub secure: Option<bool>,
    /// Whether or not the cookie is HTTP only.
    pub http_only: Option<bool>,
    /// The same sity policy of the cookie.
    pub same_site_policy: SameSitePolicy,
    /// The maximum age of the cookie.
    pub max_age: Option<f64>,
}

#[doc(hidden)]
impl From<RpcHttpCookie> for Cookie {
    fn from(cookie: RpcHttpCookie) -> Self {
        Self {
            name: cookie.name,
            value: cookie.value,
            domain: cookie.domain.and_then(|s| {
                s.string.map(|s| match s {
                    StringValue::Value(s) => s,
                })
            }),
            path: cookie.path.and_then(|s| {
                s.string.map(|s| match s {
                    StringValue::Value(s) => s,
                })
            }),
            expires: cookie.expires.and_then(|t| {
                t.timestamp.map(|t| match t {
                    Timestamp::Value(t) => DateTime::<Utc>::from_utc(
                        NaiveDateTime::from_timestamp(t.seconds, t.nanos as u32),
                        Utc,
                    ),
                })
            }),
            secure: cookie.secure.and_then(|b| {
                b.r#bool.map(|b| match b {
                    Bool::Value(b) => b,
                })
            }),
            http_only: cookie.http_only.and_then(|b| {
                b.r#bool.map(|b| match b {
                    Bool::Value(b) => b,
                })
            }),
            same_site_policy: match cookie.same_site {
                0 => SameSitePolicy::None,
                1 => SameSitePolicy::Lax,
                2 => SameSitePolicy::Strict,
                _ => panic!("unsupported cookie same site policy value"),
            },
            max_age: cookie.max_age.and_then(|d| {
                d.double.map(|d| match d {
                    Double::Value(b) => b,
                })
            }),
        }
    }
}

impl Into<RpcHttpCookie> for Cookie {
    fn into(self) -> RpcHttpCookie {
        RpcHttpCookie {
            name: self.name,
            value: self.value,
            domain: self.domain.map(|s| NullableString {
                string: Some(StringValue::Value(s)),
            }),
            path: self.path.map(|s| NullableString {
                string: Some(StringValue::Value(s)),
            }),
            expires: self.expires.map(|t| NullableTimestamp {
                timestamp: Some(Timestamp::Value(prost_types::Timestamp {
                    seconds: t.timestamp(),
                    nanos: t.timestamp_subsec_nanos() as i32,
                })),
            }),
            secure: self.secure.map(|b| NullableBool {
                r#bool: Some(Bool::Value(b)),
            }),
            http_only: self.http_only.map(|b| NullableBool {
                r#bool: Some(Bool::Value(b)),
            }),
            same_site: self.same_site_policy as i32,
            max_age: self.max_age.map(|d| NullableDouble {
                double: Some(Double::Value(d)),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_converts_from_rpc_cookie() {
        let data = RpcHttpCookie {
            name: "name".into(),
            value: "value".into(),
            domain: Some(NullableString {
                string: Some(StringValue::Value("domain".into())),
            }),
            path: Some(NullableString {
                string: Some(StringValue::Value("path".into())),
            }),
            expires: Some(NullableTimestamp {
                timestamp: Some(Timestamp::Value(prost_types::Timestamp {
                    seconds: 1,
                    nanos: 2,
                })),
            }),
            secure: Some(NullableBool {
                r#bool: Some(Bool::Value(true)),
            }),
            http_only: Some(NullableBool {
                r#bool: Some(Bool::Value(true)),
            }),
            same_site: 1,
            max_age: Some(NullableDouble {
                double: Some(Double::Value(11.4)),
            }),
        };

        let cookie: Cookie = data.into();
        assert_eq!(cookie.name, "name");
        assert_eq!(cookie.value, "value");
        assert_eq!(cookie.domain, Some("domain".into()));
        assert_eq!(cookie.path, Some("path".into()));
        assert_eq!(
            cookie.expires,
            Some(DateTime::<Utc>::from_utc(
                NaiveDateTime::from_timestamp(1, 2),
                Utc
            ))
        );
        assert_eq!(cookie.secure, Some(true));
        assert_eq!(cookie.http_only, Some(true));
        assert_eq!(cookie.same_site_policy, SameSitePolicy::Lax);
        assert_eq!(cookie.max_age, Some(11.4));
    }

    #[test]
    fn it_convert_to_rpc_cookie() {
        let cookie = Cookie {
            name: "name".into(),
            value: "value".into(),
            domain: Some("domain".into()),
            path: Some("path".into()),
            expires: Some(DateTime::<Utc>::from_utc(
                NaiveDateTime::from_timestamp(1, 2),
                Utc,
            )),
            secure: Some(true),
            http_only: Some(true),
            same_site_policy: SameSitePolicy::Lax,
            max_age: Some(11.4),
        };

        let data: RpcHttpCookie = cookie.into();
        assert_eq!(data.name, "name");
        assert_eq!(data.value, "value");
        assert_eq!(
            data.domain,
            Some(NullableString {
                string: Some(StringValue::Value("domain".into())),
            })
        );
        assert_eq!(
            data.path,
            Some(NullableString {
                string: Some(StringValue::Value("path".into())),
            })
        );
        assert_eq!(
            data.expires,
            Some(NullableTimestamp {
                timestamp: Some(Timestamp::Value(prost_types::Timestamp {
                    seconds: 1,
                    nanos: 2,
                })),
            })
        );
        assert_eq!(
            data.secure,
            Some(NullableBool {
                r#bool: Some(Bool::Value(true)),
            })
        );
        assert_eq!(
            data.http_only,
            Some(NullableBool {
                r#bool: Some(Bool::Value(true)),
            })
        );
        assert_eq!(data.same_site, 1);
        assert_eq!(
            data.max_age,
            Some(NullableDouble {
                double: Some(Double::Value(11.4)),
            })
        );
    }
}
