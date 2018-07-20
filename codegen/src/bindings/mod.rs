mod http;
mod http_trigger;
mod timer_trigger;

pub use self::http::*;
pub use self::http_trigger::*;
pub use self::timer_trigger::*;

use proc_macro::Diagnostic;
use quote::ToTokens;
use std::collections::HashMap;
use std::convert::TryFrom;
use util::AttributeArguments;

pub type BindingFactory = fn(&AttributeArguments) -> Result<Binding, Diagnostic>;

lazy_static! {
    pub static ref TRIGGERS: HashMap<&'static str, BindingFactory> = {
        let mut map: HashMap<&'static str, BindingFactory> = HashMap::new();
        map.insert("HttpRequest", |args| {
            Ok(Binding::HttpTrigger(HttpTrigger::try_from(args)?))
        });
        map.insert("TimerInfo", |args| {
            Ok(Binding::TimerTrigger(TimerTrigger::try_from(args)?))
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
            Ok(Binding::Http(Http::try_from(args)?))
        });
        map
    };
}

#[derive(Debug)]
pub enum Binding {
    Context,
    HttpTrigger(HttpTrigger),
    Http(Http),
    TimerTrigger(TimerTrigger),
}

impl Binding {
    pub fn name(&self) -> Option<&str> {
        match self {
            Binding::Context => None,
            Binding::HttpTrigger(b) => Some(&b.name),
            Binding::Http(b) => Some(&b.name),
            Binding::TimerTrigger(b) => Some(&b.name),
        }
    }

    pub fn is_context(&self) -> bool {
        match self {
            Binding::Context => true,
            _ => false,
        }
    }

    pub fn is_trigger(&self) -> bool {
        match self {
            Binding::HttpTrigger(_) | Binding::TimerTrigger(_) => true,
            Binding::Context | Binding::Http(_) => false,
        }
    }
}

impl ToTokens for Binding {
    fn to_tokens(&self, tokens: &mut ::proc_macro2::TokenStream) {
        match self {
            Binding::Context => panic!("context bindings should not be tokenized"),
            Binding::HttpTrigger(b) => quote!(
                ::azure_functions::codegen::Binding::HttpTrigger(#b)
            ).to_tokens(tokens),
            Binding::Http(b) => quote!(
                ::azure_functions::codegen::Binding::Http(#b)
            ).to_tokens(tokens),
            Binding::TimerTrigger(b) => quote!(
                ::azure_functions::codegen::Binding::TimerTrigger(#b)
            ).to_tokens(tokens),
        };
    }
}
