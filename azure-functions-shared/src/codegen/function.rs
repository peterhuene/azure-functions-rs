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

#[doc(hidden)]
#[derive(Clone)]
pub struct Function {
    pub name: Cow<'static, str>,
    pub disabled: bool,
    pub bindings: Cow<'static, [Binding]>,
    pub invoker_name: Option<Cow<'static, str>>,
    pub invoker: Option<fn(&str, rpc::InvocationRequest) -> rpc::InvocationResponse>,
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
            invoker_name: None,
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
        let bindings = self.bindings.iter().filter(|x| !x.is_context());
        let invoker_name =
            QuotableOption(self.invoker_name.as_ref().map(|x| QuotableBorrowedStr(x)));
        let invoker = Ident::new(
            self.invoker_name
                .as_ref()
                .expect("function must have an invoker"),
            Span::call_site(),
        );

        quote!(
        ::azure_functions::codegen::Function {
            name: #name,
            disabled: #disabled,
            bindings: ::std::borrow::Cow::Borrowed(&[#(#bindings),*]),
            invoker_name: #invoker_name,
            invoker: Some(#invoker),
            manifest_dir: Some(::std::borrow::Cow::Borrowed(env!("CARGO_MANIFEST_DIR"))),
            file: Some(::std::borrow::Cow::Borrowed(file!())),
        }
        )
        .to_tokens(tokens)
    }
}
