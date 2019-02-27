use crate::codegen::{
    quotable::{QuotableBorrowedStr, QuotableOption},
    AttributeArguments, TryFrom,
};
use crate::util::to_camel_case;
use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use serde::{ser::SerializeMap, Serialize, Serializer};
use std::borrow::Cow;
use syn::{spanned::Spanned, Lit};

pub const HTTP_TRIGGER_TYPE: &str = "httpTrigger";

#[derive(Debug, Clone)]
pub struct HttpTrigger {
    pub name: Cow<'static, str>,
    pub auth_level: Option<Cow<'static, str>>,
    pub methods: Cow<'static, [Cow<'static, str>]>,
    pub route: Option<Cow<'static, str>>,
    pub web_hook_type: Option<Cow<'static, str>>,
}

// TODO: when https://github.com/serde-rs/serde/issues/760 is resolved, remove implementation in favor of custom Serialize derive
// The fix would allow us to set the constant `type` and `direction` entries rather than having to emit them manually.
impl Serialize for HttpTrigger {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(None)?;

        map.serialize_entry("name", &self.name)?;
        map.serialize_entry("type", HTTP_TRIGGER_TYPE)?;
        map.serialize_entry("direction", "in")?;

        if let Some(auth_level) = self.auth_level.as_ref() {
            map.serialize_entry("authLevel", auth_level)?;
        }
        if !self.methods.is_empty() {
            map.serialize_entry("methods", &self.methods)?;
        }
        if let Some(route) = self.route.as_ref() {
            map.serialize_entry("route", route)?;
        }
        if let Some(web_hook_type) = self.web_hook_type.as_ref() {
            map.serialize_entry("webHookType", web_hook_type)?;
        }

        map.end()
    }
}

impl TryFrom<AttributeArguments> for HttpTrigger {
    type Error = (Span, String);

    fn try_from(args: AttributeArguments) -> Result<Self, Self::Error> {
        let mut name = None;
        let mut auth_level = None;
        let mut methods = Vec::new();
        let mut route = None;
        let mut web_hook_type = None;

        for (key, value) in args.list.iter() {
            let key_str = key.to_string();

            match key_str.as_str() {
                "name" => match value {
                    Lit::Str(s) => {
                        name = Some(Cow::Owned(to_camel_case(&s.value())));
                    }
                    _ => {
                        return Err((
                            value.span(),
                            "expected a literal string value for the 'name' argument".to_string(),
                        ));
                    }
                },
                "auth_level" => match value {
                    Lit::Str(s) => {
                        let v = s.value();
                        auth_level = match v.as_str() {
                            "anonymous" | "function" | "admin" => Some(Cow::Owned(v)),
                            _ => {
                                return Err((value
                                    .span(),
                                    "expected 'anonymous', 'function', or 'admin' for the 'auth_level' attribute argument".to_string()));
                            }
                        };
                    }
                    _ => {
                        return Err((
                            value.span(),
                            "expected a literal string value for the 'auth_level' argument"
                                .to_string(),
                        ));
                    }
                },
                "methods" => match value {
                    Lit::Str(s) => {
                        let mut invalid = Vec::new();
                        methods = s
                            .value()
                            .split(',')
                            .filter_map(|x| {
                                let x = x.trim();
                                match x {
                                    "get" | "post" | "delete" | "head" | "patch" | "put"
                                    | "options" | "trace" => Some(Cow::Owned(x.to_string())),
                                    _ => {
                                        invalid.push(x.to_string());
                                        None
                                    }
                                }
                            })
                            .collect();

                        if !invalid.is_empty() {
                            return Err((
                                value.span(),
                                format!("unsupported HTTP methods: {}", invalid.join(", ")),
                            ));
                        }
                    }
                    _ => {
                        return Err((value
                                .span(),
                                "expected a comma-delimited literal string value for the 'methods' argument".to_string()));
                    }
                },
                "route" => match value {
                    Lit::Str(s) => route = Some(Cow::Owned(s.value())),
                    _ => {
                        return Err((
                            value.span(),
                            "expected a literal string value for the 'route' argument".to_string(),
                        ));
                    }
                },
                "web_hook_type" => match value {
                    Lit::Str(s) => {
                        let s = s.value();
                        web_hook_type = match s.trim() {
                            "generic" | "github" | "slack" => Some(Cow::Owned(s)),
                            _ => {
                                return Err((value
                                    .span(),
                                    "expected 'generic', 'github', or 'slack' for the 'web_hook_type' attribute argument".to_string()));
                            }
                        };
                    }
                    _ => {
                        return Err((
                            value.span(),
                            "expected a literal string value for the 'web_hook_type' argument"
                                .to_string(),
                        ));
                    }
                },
                _ => {
                    return Err((
                        key.span(),
                        format!("unsupported binding attribute argument '{}'", key_str),
                    ));
                }
            };
        }

        Ok(HttpTrigger {
            name: name.unwrap(),
            auth_level,
            methods: Cow::Owned(methods),
            route,
            web_hook_type,
        })
    }
}

impl ToTokens for HttpTrigger {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let name = QuotableBorrowedStr(&self.name);
        let auth_level = QuotableOption(self.auth_level.as_ref().map(|x| QuotableBorrowedStr(x)));
        let methods = self.methods.iter().map(|x| QuotableBorrowedStr(x));
        let route = QuotableOption(self.route.as_ref().map(|x| QuotableBorrowedStr(x)));
        let web_hook_type =
            QuotableOption(self.web_hook_type.as_ref().map(|x| QuotableBorrowedStr(x)));

        quote!(::azure_functions::codegen::bindings::HttpTrigger {
            name: #name,
            auth_level: #auth_level,
            methods: ::std::borrow::Cow::Borrowed(&[#(#methods),*]),
            route: #route,
            web_hook_type: #web_hook_type,
        })
        .to_tokens(tokens)
    }
}
