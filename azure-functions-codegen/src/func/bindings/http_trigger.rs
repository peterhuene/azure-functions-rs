use azure_functions_shared::codegen;
use proc_macro::Diagnostic;
use proc_macro2::TokenStream;
use quote::ToTokens;
use std::borrow::Cow;
use std::convert::TryFrom;
use syn::spanned::Spanned;
use syn::Lit;
use util::{to_camel_case, AttributeArguments, QuotableBorrowedStr, QuotableOption};

pub struct HttpTrigger<'a>(pub Cow<'a, codegen::bindings::HttpTrigger>);

impl<'a> TryFrom<&'a AttributeArguments> for HttpTrigger<'a> {
    type Error = Diagnostic;

    fn try_from(args: &AttributeArguments) -> Result<Self, Self::Error> {
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
                        name = Some(Cow::Owned(to_camel_case(&s.value())));
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
                            "anonymous" | "function" | "admin" => Some(Cow::Owned(v)),
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
                                    | "options" | "trace" => Some(Cow::Owned(x.to_string())),
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
                    Lit::Str(s) => route = Some(Cow::Owned(s.value())),
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
                            "generic" | "github" | "slack" => Some(Cow::Owned(s)),
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

        Ok(HttpTrigger(Cow::Owned(codegen::bindings::HttpTrigger {
            name: name.expect("expected a name for the HttpTrigger binding"),
            auth_level: auth_level,
            methods: Cow::Owned(methods),
            route: route,
            web_hook_type: web_hook_type,
        })))
    }
}

impl<'a> ToTokens for HttpTrigger<'a> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let name = QuotableBorrowedStr(&self.0.name);
        let auth_level = QuotableOption(self.0.auth_level.as_ref().map(|x| QuotableBorrowedStr(x)));
        let methods = self.0.methods.iter().map(|x| QuotableBorrowedStr(x));
        let route = QuotableOption(self.0.route.as_ref().map(|x| QuotableBorrowedStr(x)));
        let web_hook_type = QuotableOption(
            self.0
                .web_hook_type
                .as_ref()
                .map(|x| QuotableBorrowedStr(x)),
        );

        quote!(::azure_functions::codegen::bindings::HttpTrigger {
            name: #name,
            auth_level: #auth_level,
            methods: ::std::borrow::Cow::Borrowed(&[#(#methods),*]),
            route: #route,
            web_hook_type: #web_hook_type,
        }).to_tokens(tokens)
    }
}
