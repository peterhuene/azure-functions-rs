use crate::util::{
    to_camel_case, AttributeArguments, MacroError, QuotableBorrowedStr, QuotableOption, TryFrom,
};
use azure_functions_shared::codegen;
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use std::borrow::Cow;
use syn::spanned::Spanned;
use syn::Lit;

pub struct HttpTrigger<'a>(pub Cow<'a, codegen::bindings::HttpTrigger>);

impl TryFrom<AttributeArguments> for HttpTrigger<'_> {
    type Error = MacroError;

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
                            "expected a literal string value for the 'name' argument",
                        )
                            .into());
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
                                    "expected 'anonymous', 'function', or 'admin' for the 'auth_level' attribute argument").into());
                            }
                        };
                    }
                    _ => {
                        return Err((
                            value.span(),
                            "expected a literal string value for the 'auth_level' argument",
                        )
                            .into());
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
                                format!("unsupported HTTP methods: {}", invalid.join(", "))
                                    .as_ref(),
                            )
                                .into());
                        }
                    }
                    _ => {
                        return Err((value
                                .span(),
                                "expected a comma-delimited literal string value for the 'methods' argument").into());
                    }
                },
                "route" => match value {
                    Lit::Str(s) => route = Some(Cow::Owned(s.value())),
                    _ => {
                        return Err((
                            value.span(),
                            "expected a literal string value for the 'route' argument",
                        )
                            .into());
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
                                    "expected 'generic', 'github', or 'slack' for the 'web_hook_type' attribute argument").into());
                            }
                        };
                    }
                    _ => {
                        return Err((
                            value.span(),
                            "expected a literal string value for the 'web_hook_type' argument",
                        )
                            .into());
                    }
                },
                _ => {
                    return Err((
                        key.span(),
                        format!("unsupported binding attribute argument '{}'", key_str).as_ref(),
                    )
                        .into());
                }
            };
        }

        Ok(HttpTrigger(Cow::Owned(codegen::bindings::HttpTrigger {
            name: name.expect("expected a name for a http trigger binding"),
            auth_level,
            methods: Cow::Owned(methods),
            route,
            web_hook_type,
        })))
    }
}

impl ToTokens for HttpTrigger<'_> {
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
        })
        .to_tokens(tokens)
    }
}
