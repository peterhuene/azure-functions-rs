use serde::Deserialize;

/// Represents a HTTP status code.
#[derive(Default, Debug, Clone, Copy, Hash, Deserialize, PartialEq, Eq)]
pub struct Status(u16);

macro_rules! statuses {
    ($($name:ident => $code:expr => $code_str:expr),+) => {
        $(
            #[doc="[Status](struct.Status.html) with code <b>"]
            #[doc=$code_str]
            #[doc="</b>."]
            #[allow(non_upper_case_globals)]
            pub const $name: Self = Self($code);
         )+

        /// Returns a `Status` given a status code `code`.
        ///
        /// # Examples
        ///
        /// Create a `Status` from a status code `code`:
        ///
        /// ```rust
        /// use azure_functions::http::Status;
        ///
        /// assert_eq!(Status::from_code(404), Status::NotFound);
        /// ```
        pub fn from_code(code: u16) -> Self {
            match code {
                $($code => Self::$name,)+
                _ => Self(code)
            }
        }
    };
}

impl Status {
    statuses! {
        Continue => 100 => "100",
        SwitchingProtocols => 101 => "101",
        Processing => 102 => "102",
        Ok => 200 => "200",
        Created => 201 => "201",
        Accepted => 202 => "202",
        NonAuthoritativeInformation => 203 => "203",
        NoContent => 204 => "204",
        ResetContent => 205 => "205",
        PartialContent => 206 => "206",
        MultiStatus => 207 => "207",
        AlreadyReported => 208 => "208",
        ImUsed => 226 => "226",
        MultipleChoices => 300 => "300",
        MovedPermanently => 301 => "301",
        Found => 302 => "302",
        SeeOther => 303 => "303",
        NotModified => 304 => "304",
        UseProxy => 305 => "305",
        TemporaryRedirect => 307 => "307",
        PermanentRedirect => 308 => "308",
        BadRequest => 400 => "400",
        Unauthorized => 401 => "401",
        PaymentRequired => 402 => "402",
        Forbidden => 403 => "403",
        NotFound => 404 => "404",
        MethodNotAllowed => 405 => "405",
        NotAcceptable => 406 => "406",
        ProxyAuthenticationRequired => 407 => "407",
        RequestTimeout => 408 => "408",
        Conflict => 409 => "409",
        Gone => 410 => "410",
        LengthRequired => 411 => "411",
        PreconditionFailed => 412 => "412",
        PayloadTooLarge => 413 => "413",
        UriTooLong => 414 => "414",
        UnsupportedMediaType => 415 => "415",
        RangeNotSatisfiable => 416 => "416",
        ExpectationFailed => 417 => "417",
        ImATeapot => 418 => "418",
        MisdirectedRequest => 421 => "421",
        UnprocessableEntity => 422 => "422",
        Locked => 423 => "423",
        FailedDependency => 424 => "424",
        UpgradeRequired => 426 => "426",
        PreconditionRequired => 428 => "428",
        TooManyRequests => 429 => "429",
        RequestHeaderFieldsTooLarge => 431 => "431",
        UnavailableForLegalReasons => 451 => "451",
        InternalServerError => 500 => "500",
        NotImplemented => 501 => "501",
        BadGateway => 502 => "502",
        ServiceUnavailable => 503 => "503",
        GatewayTimeout => 504 => "504",
        HttpVersionNotSupported => 505 => "505",
        VariantAlsoNegotiates => 506 => "506",
        InsufficientStorage => 507 => "507",
        LoopDetected => 508 => "508",
        NotExtended => 510 => "510",
        NetworkAuthenticationRequired => 511 => "511"
    }
}

impl ToString for Status {
    fn to_string(&self) -> String {
        self.0.to_string()
    }
}

impl From<u16> for Status {
    fn from(code: u16) -> Self {
        Self::from_code(code)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_converts_to_string() {
        assert_eq!(Status::Ok.to_string(), "200");
        assert_eq!(Status::NotFound.to_string(), "404");
    }

    #[test]
    fn it_converts_from_code() {
        let status: Status = 200.into();
        assert_eq!(status, Status::Ok);
        assert_eq!(Status::from_code(404), Status::NotFound);
    }
}
