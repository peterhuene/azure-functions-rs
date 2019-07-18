use crate::codegen::{
    bindings::Binding,
    get_boolean_value, get_string_value, iter_attribute_args, macro_panic,
    quotable::{QuotableBorrowedStr, QuotableOption},
};
use crate::rpc;
use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use serde::{ser::SerializeMap, Serialize, Serializer};
use std::borrow::Cow;
use syn::{parse_str, spanned::Spanned, AttributeArgs, Ident};

pub type SyncFn = fn(rpc::InvocationRequest) -> rpc::InvocationResponse;

#[cfg(feature = "unstable")]
pub type InvocationFuture =
    std::pin::Pin<Box<dyn std::future::Future<Output = rpc::InvocationResponse> + Send>>;

#[cfg(feature = "unstable")]
pub type AsyncFn = fn(rpc::InvocationRequest) -> InvocationFuture;

#[cfg(not(feature = "unstable"))]
pub type AsyncFn = fn(rpc::InvocationRequest) -> !;

pub enum InvokerFn {
    Sync(Option<SyncFn>),
    Async(Option<AsyncFn>),
}

struct InvokerFnTokens<'a> {
    ident: Ident,
    invoker_fn: &'a InvokerFn,
}

impl<'a> InvokerFnTokens<'a> {
    pub fn new(name: &str, invoker_fn: &'a InvokerFn) -> Self {
        InvokerFnTokens {
            ident: Ident::new(name, Span::call_site()),
            invoker_fn,
        }
    }
}

impl<'a> ToTokens for InvokerFnTokens<'a> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let ident = &self.ident;
        match self.invoker_fn {
            InvokerFn::Sync(_) => quote!(::azure_functions::codegen::InvokerFn::Sync(Some(#ident))),
            InvokerFn::Async(_) => {
                quote!(::azure_functions::codegen::InvokerFn::Async(Some(#ident)))
            }
        }
        .to_tokens(tokens);
    }
}

pub struct Invoker {
    pub name: Cow<'static, str>,
    pub invoker_fn: InvokerFn,
}

impl ToTokens for Invoker {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let name = QuotableBorrowedStr(&self.name);
        let invoker_fn = InvokerFnTokens::new(&self.name, &self.invoker_fn);

        quote!(::azure_functions::codegen::Invoker { name: #name, invoker_fn: #invoker_fn, })
            .to_tokens(tokens);
    }
}

pub struct Function {
    pub name: Cow<'static, str>,
    pub disabled: bool,
    pub bindings: Cow<'static, [Binding]>,
    pub invoker: Option<Invoker>,
    pub manifest_dir: Option<Cow<'static, str>>,
    pub file: Option<Cow<'static, str>>,
}

// TODO: when https://github.com/serde-rs/serde/issues/760 is resolved, remove implementation in favor of custom Serialize derive
// The fix would allow us to set the constant `generatedBy` entry rather than having to emit them manually.
impl Serialize for Function {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(None)?;

        map.serialize_entry("generatedBy", "azure-functions-rs")?;
        map.serialize_entry("disabled", &self.disabled)?;
        map.serialize_entry("bindings", &self.bindings)?;

        map.end()
    }
}

impl From<AttributeArgs> for Function {
    fn from(args: AttributeArgs) -> Self {
        let mut name = None;
        let mut disabled = None;

        iter_attribute_args(&args, |key, value| {
            let key_name = key.to_string();

            match key_name.as_str() {
                "name" => {
                    name = {
                        let name = get_string_value("name", value);
                        parse_str::<Ident>(&name)
                            .map_err(|_| {
                                macro_panic(
                                value.span(),
                                "a legal function identifier is required for the 'name' argument",
                            )
                            })
                            .unwrap();
                        Some(Cow::from(name))
                    }
                }
                "disabled" => disabled = Some(get_boolean_value("disabled", value)),
                _ => macro_panic(
                    key.span(),
                    format!("unsupported attribue argument '{}'", key_name),
                ),
            };

            true
        });

        Function {
            name: name.unwrap_or(Cow::Borrowed("")),
            disabled: disabled.unwrap_or(false),
            bindings: Cow::Owned(Vec::new()),
            invoker: None,
            manifest_dir: None,
            file: None,
        }
    }
}

impl ToTokens for Function {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let name = QuotableBorrowedStr(&self.name);
        let disabled = self.disabled;
        let bindings = self.bindings.iter();
        let invoker = QuotableOption(self.invoker.as_ref());

        quote!(
            ::azure_functions::codegen::Function {
                name: #name,
                disabled: #disabled,
                bindings: ::std::borrow::Cow::Borrowed(&[#(#bindings),*]),
                invoker: #invoker,
                manifest_dir: Some(::std::borrow::Cow::Borrowed(env!("CARGO_MANIFEST_DIR"))),
                file: Some(::std::borrow::Cow::Borrowed(file!())),
            }
        )
        .to_tokens(tokens)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codegen::{
        bindings::{Binding, Http, HttpTrigger},
        tests::should_panic,
    };
    use proc_macro2::TokenStream;
    use quote::ToTokens;
    use serde_json::to_string;
    use syn::{parse_str, NestedMeta};

    #[test]
    fn it_serializes_to_json() {
        let func = Function {
            name: Cow::from("name"),
            disabled: false,
            bindings: Cow::Owned(vec![
                Binding::HttpTrigger(HttpTrigger {
                    name: Cow::from("foo"),
                    auth_level: Some(Cow::from("bar")),
                    methods: Cow::from(vec![Cow::from("foo"), Cow::from("bar"), Cow::from("baz")]),
                    route: Some(Cow::from("baz")),
                }),
                Binding::Http(Http {
                    name: Cow::from("bar"),
                }),
            ]),
            invoker: Some(Invoker {
                name: Cow::Borrowed("invoker"),
                invoker_fn: InvokerFn::Async(None),
            }),
            manifest_dir: None,
            file: None,
        };

        assert_eq!(
            to_string(&func).unwrap(),
            r#"{"generatedBy":"azure-functions-rs","disabled":false,"bindings":[{"type":"httpTrigger","direction":"in","name":"foo","authLevel":"bar","methods":["foo","bar","baz"],"route":"baz"},{"type":"http","direction":"out","name":"bar"}]}"#
        );
    }

    #[test]
    fn it_parses_attribute_arguments() {
        let func: Function = vec![
            parse_str::<NestedMeta>(r#"name = "foo""#).unwrap(),
            parse_str::<NestedMeta>(r#"disabled = true"#).unwrap(),
        ]
        .into();

        assert_eq!(func.name, "foo");
        assert_eq!(func.disabled, true);
        assert_eq!(func.bindings.len(), 0);
        assert_eq!(func.invoker.is_none(), true);
        assert_eq!(func.manifest_dir.is_none(), true);
        assert_eq!(func.file.is_none(), true);
    }

    #[test]
    fn it_requires_an_identifier_for_name() {
        should_panic(
            || {
                let _: Function = vec![parse_str::<NestedMeta>(r#"name = "123""#).unwrap()].into();
            },
            "a legal function identifier is required for the \'name\' argument",
        );
    }

    #[test]
    fn it_requires_the_name_attribute_be_a_string() {
        should_panic(
            || {
                let _: Function = vec![parse_str::<NestedMeta>(r#"name = false"#).unwrap()].into();
            },
            "expected a literal string value for the 'name' argument",
        );
    }

    #[test]
    fn it_requires_the_disabled_attribute_be_a_boolean() {
        should_panic(
            || {
                let _: Function =
                    vec![parse_str::<NestedMeta>(r#"disabled = "false""#).unwrap()].into();
            },
            "expected a literal boolean value for the 'disabled' argument",
        );
    }

    #[test]
    fn it_converts_to_tokens() {
        let func = Function {
            name: Cow::from("name"),
            disabled: false,
            bindings: Cow::Owned(vec![
                Binding::HttpTrigger(HttpTrigger {
                    name: Cow::from("foo"),
                    auth_level: Some(Cow::from("bar")),
                    methods: Cow::from(vec![Cow::from("foo"), Cow::from("bar"), Cow::from("baz")]),
                    route: Some(Cow::from("baz")),
                }),
                Binding::Http(Http {
                    name: Cow::from("bar"),
                }),
            ]),
            invoker: Some(Invoker {
                name: Cow::Borrowed("invoker"),
                invoker_fn: InvokerFn::Async(None),
            }),
            manifest_dir: None,
            file: None,
        };

        let mut stream = TokenStream::new();
        func.to_tokens(&mut stream);
        let mut tokens = stream.to_string();
        tokens.retain(|c| c != ' ');

        assert_eq!(tokens, r#"::azure_functions::codegen::Function{name:::std::borrow::Cow::Borrowed("name"),disabled:false,bindings:::std::borrow::Cow::Borrowed(&[::azure_functions::codegen::bindings::Binding::HttpTrigger(::azure_functions::codegen::bindings::HttpTrigger{name:::std::borrow::Cow::Borrowed("foo"),auth_level:Some(::std::borrow::Cow::Borrowed("bar")),methods:::std::borrow::Cow::Borrowed(&[::std::borrow::Cow::Borrowed("foo"),::std::borrow::Cow::Borrowed("bar"),::std::borrow::Cow::Borrowed("baz"),]),route:Some(::std::borrow::Cow::Borrowed("baz")),}),::azure_functions::codegen::bindings::Binding::Http(::azure_functions::codegen::bindings::Http{name:::std::borrow::Cow::Borrowed("bar"),})]),invoker:Some(::azure_functions::codegen::Invoker{name:::std::borrow::Cow::Borrowed("invoker"),invoker_fn:::azure_functions::codegen::InvokerFn::Async(Some(invoker)),}),manifest_dir:Some(::std::borrow::Cow::Borrowed(env!("CARGO_MANIFEST_DIR"))),file:Some(::std::borrow::Cow::Borrowed(file!())),}"#);
    }
}
