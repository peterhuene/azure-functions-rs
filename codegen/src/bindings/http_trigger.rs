use proc_macro::Diagnostic;
use quote::ToTokens;
use std::convert::TryFrom;
use syn::spanned::Spanned;
use syn::{Ident, Lit};
use util::{AttributeArguments, QuotableOption};

#[derive(Debug)]
pub struct HttpTrigger {
    pub name: String,
    pub auth_level: Option<String>,
    pub methods: Vec<String>,
    pub route: Option<String>,
    pub web_hook_type: Option<String>,
}

impl<'a> TryFrom<&'a AttributeArguments> for HttpTrigger {
    type Error = Diagnostic;

    fn try_from(args: &'a AttributeArguments) -> Result<Self, Self::Error> {
        let mut name = None;
        let mut auth_level = None;
        let mut methods = Vec::new();
        let mut route = None;
        let mut web_hook_type = None;

        for (key, value) in args.0.iter() {
            let key_str = key.to_string();

            match key_str.as_str() {
                "name" => match value {
                    Lit::Str(s) => {
                        name = s
                            .parse::<Ident>()
                            .map(|x| Some(x.to_string()))
                            .map_err(|_| {
                                value.span().unstable().error(
                                "a legal parameter identifier is required for the 'name' argument",
                            )
                            })?;
                    }
                    _ => {
                        return Err(value
                            .span()
                            .unstable()
                            .error("expected a literal string value for the 'name' argument"));
                    }
                },
                "auth_level" => match value {
                    Lit::Str(s) => {
                        let v = s.value();
                        auth_level = match v.as_str() {
                            "anonymous" | "function" | "admin" => Some(v),
                            _ => {
                                return Err(value
                                .span()
                                .unstable()
                                .error("expected 'anonymous', 'function', or 'admin' for the 'auth_level' attribute argument"));
                            }
                        };
                    }
                    _ => {
                        return Err(value.span().unstable().error(
                            "expected a literal string value for the 'auth_level' argument",
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
                                    | "options" | "trace" => Some(x.to_string()),
                                    _ => {
                                        invalid.push(x.to_string());
                                        None
                                    }
                                }
                            })
                            .collect();

                        if !invalid.is_empty() {
                            return Err(value.span().unstable().error(format!(
                                "unsupported HTTP methods: {}",
                                invalid.join(", ")
                            )));
                        }
                    }
                    _ => {
                        return Err(value
                            .span()
                            .unstable()
                            .error("expected a comma-delimited literal string value for the 'methods' argument"));
                    }
                },
                "route" => match value {
                    Lit::Str(s) => route = Some(s.value()),
                    _ => {
                        return Err(value
                            .span()
                            .unstable()
                            .error("expected a literal string value for the 'route' argument"));
                    }
                },
                "web_hook_type" => match value {
                    Lit::Str(s) => {
                        let s = s.value();
                        web_hook_type = match s.trim() {
                            "generic" | "github" | "slack" => Some(s),
                            _ => {
                                return Err(value
                                .span()
                                .unstable()
                                .error("expected 'generic', 'github', or 'slack' for the 'web_hook_type' attribute argument"));
                            }
                        };
                    }
                    _ => {
                        return Err(value.span().unstable().error(
                            "expected a literal string value for the 'web_hook_type' argument",
                        ));
                    }
                },
                _ => {
                    return Err(key.span().unstable().error(format!(
                        "unsupported binding attribute argument '{}'",
                        key_str
                    )));
                }
            };
        }

        Ok(HttpTrigger {
            name: name.expect("expected a name for the HttpTrigger binding"),
            auth_level: auth_level,
            methods: methods,
            route: route,
            web_hook_type: web_hook_type,
        })
    }
}

impl ToTokens for HttpTrigger {
    fn to_tokens(&self, tokens: &mut ::proc_macro2::TokenStream) {
        let name = &self.name;
        let auth_level = QuotableOption(self.auth_level.clone());
        let methods = &self.methods;
        let route = QuotableOption(self.route.clone());
        let web_hook_type = QuotableOption(self.web_hook_type.clone());

        quote!(::azure_functions::codegen::bindings::HttpTrigger {
            name: #name,
            auth_level: &#auth_level,
            methods: &[#(#methods),*],
            route: &#route,
            web_hook_type: &#web_hook_type,
        }).to_tokens(tokens)
    }
}
