use azure_functions_shared::codegen;
use func::bindings::{Blob, BlobTrigger, Http, HttpTrigger, Queue, QueueTrigger, TimerTrigger};
use proc_macro::Diagnostic;
use proc_macro2::TokenStream;
use quote::ToTokens;
use std::borrow::Cow;
use std::collections::HashMap;
use std::convert::TryFrom;
use util::AttributeArguments;

pub struct Binding<'a>(pub &'a codegen::Binding);

impl ToTokens for Binding<'_> {
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
            codegen::Binding::BlobTrigger(b) => {
                let b = BlobTrigger(Cow::Borrowed(b));
                quote!(::azure_functions::codegen::Binding::BlobTrigger(#b)).to_tokens(tokens)
            }
            codegen::Binding::Blob(b) => {
                let b = Blob(Cow::Borrowed(b));
                quote!(::azure_functions::codegen::Binding::Blob(#b)).to_tokens(tokens)
            }
        };
    }
}

pub type BindingFactory = fn(AttributeArguments) -> Result<codegen::Binding, Diagnostic>;
type BindingMap = HashMap<&'static str, BindingFactory>;

lazy_static! {
    pub static ref TRIGGERS: BindingMap = {
        let mut map: BindingMap = HashMap::new();
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
        map.insert("BlobTrigger", |args| {
            Ok(codegen::Binding::BlobTrigger(
                BlobTrigger::try_from(args)?.0.into_owned(),
            ))
        });
        map
    };
    pub static ref INPUT_BINDINGS: BindingMap = {
        let mut map: BindingMap = HashMap::new();
        map.insert("Blob", |args| {
            let mut binding = Blob::try_from(args)?.0.into_owned();
            binding.direction = Cow::Borrowed("in");
            Ok(codegen::Binding::Blob(binding))
        });
        map
    };
    pub static ref INPUT_OUTPUT_BINDINGS: BindingMap = {
        let map: BindingMap = HashMap::new();
        // TODO: properly implement inout bindings
        // map.insert("Blob", |args| {
        //     let mut binding = Blob::try_from(args)?.0.into_owned();
        //     binding.direction = Cow::Borrowed("inout");
        //     Ok(codegen::Binding::Blob(binding))
        // });

        map
    };
    pub static ref OUTPUT_BINDINGS: BindingMap = {
        let mut map: BindingMap = HashMap::new();
        map.insert("HttpResponse", |args| {
            Ok(codegen::Binding::Http(Http::try_from(args)?.0.into_owned()))
        });
        map.insert("QueueMessage", |args| {
            Ok(codegen::Binding::Queue(
                Queue::try_from(args)?.0.into_owned(),
            ))
        });
        map.insert("Blob", |args| {
            let mut binding = Blob::try_from(args)?.0.into_owned();
            binding.direction = Cow::Borrowed("out");
            Ok(codegen::Binding::Blob(binding))
        });
        map
    };
}
