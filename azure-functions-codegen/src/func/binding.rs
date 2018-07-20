use azure_functions_shared::codegen;
use func::bindings::{Http, HttpTrigger, Queue, QueueTrigger, TimerTrigger};
use proc_macro::Diagnostic;
use proc_macro2::TokenStream;
use quote::ToTokens;
use std::borrow::Cow;
use std::collections::HashMap;
use std::convert::TryFrom;
use util::AttributeArguments;

pub struct Binding<'a>(pub &'a codegen::Binding);

impl<'a> ToTokens for Binding<'a> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self.0 {
            codegen::Binding::Context => panic!("context bindings cannot be tokenized"),
            codegen::Binding::HttpTrigger(b) => {
                let b = HttpTrigger(Cow::Borrowed(b));
                quote!(::azure_functions::codegen::Binding::HttpTrigger(#b)).to_tokens(tokens)
            }
            codegen::Binding::Http(b) => {
                let b = Http(Cow::Borrowed(b));
                quote!(::azure_functions::codegen::Binding::Http(#b)).to_tokens(tokens)
            }
            codegen::Binding::TimerTrigger(b) => {
                let b = TimerTrigger(Cow::Borrowed(b));
                quote!(::azure_functions::codegen::Binding::TimerTrigger(#b)).to_tokens(tokens)
            }
            codegen::Binding::QueueTrigger(b) => {
                let b = QueueTrigger(Cow::Borrowed(b));
                quote!(::azure_functions::codegen::Binding::QueueTrigger(#b)).to_tokens(tokens)
            }
            codegen::Binding::Queue(b) => {
                let b = Queue(Cow::Borrowed(b));
                quote!(::azure_functions::codegen::Binding::Queue(#b)).to_tokens(tokens)
            }
        };
    }
}

pub type BindingFactory = fn(&AttributeArguments) -> Result<codegen::Binding, Diagnostic>;

lazy_static! {
    pub static ref TRIGGERS: HashMap<&'static str, BindingFactory> = {
        let mut map: HashMap<&'static str, BindingFactory> = HashMap::new();
        map.insert("HttpRequest", |args| {
            Ok(codegen::Binding::HttpTrigger(
                HttpTrigger::try_from(args)?.0.into_owned(),
            ))
        });
        map.insert("TimerInfo", |args| {
            Ok(codegen::Binding::TimerTrigger(
                TimerTrigger::try_from(args)?.0.into_owned(),
            ))
        });
        map.insert("QueueTrigger", |args| {
            Ok(codegen::Binding::QueueTrigger(
                QueueTrigger::try_from(args)?.0.into_owned(),
            ))
        });
        map
    };
    pub static ref INPUT_BINDINGS: HashMap<&'static str, BindingFactory> = {
        let map: HashMap<&'static str, BindingFactory> = HashMap::new();
        map
    };
    pub static ref INPUT_OUTPUT_BINDINGS: HashMap<&'static str, BindingFactory> = {
        let map: HashMap<&'static str, BindingFactory> = HashMap::new();
        map
    };
    pub static ref OUTPUT_BINDINGS: HashMap<&'static str, BindingFactory> = {
        let mut map: HashMap<&'static str, BindingFactory> = HashMap::new();
        map.insert("HttpResponse", |args| {
            Ok(codegen::Binding::Http(Http::try_from(args)?.0.into_owned()))
        });
        map.insert("QueueMessage", |args| {
            Ok(codegen::Binding::Queue(
                Queue::try_from(args)?.0.into_owned(),
            ))
        });
        map
    };
}
