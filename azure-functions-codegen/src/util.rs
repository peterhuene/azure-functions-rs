use azure_functions_shared::codegen::TryFrom;
#[cfg(feature = "unstable")]
use proc_macro::Diagnostic;
use proc_macro::TokenStream;
use proc_macro2::Span;
use std::convert::Into;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{parse, Path, PathSegment, Token};

cfg_if::cfg_if! {
    if #[cfg(feature = "unstable")] {
        pub struct MacroError {
            inner: Diagnostic,
        }

        impl MacroError {
            pub fn emit(self) {
                self.inner.emit()
            }
        }

        impl Into<MacroError> for String {
            fn into(self) -> MacroError {
                MacroError {
                    inner: Span::call_site().unstable().error(self)
                }
            }
        }

        impl Into<MacroError> for (Span, String) {
            fn into(self) -> MacroError {
                MacroError {
                    inner: self.0.unstable().error(self.1)
                }
            }
        }

    } else {
        pub struct MacroError {
            message: String,
        }

        impl MacroError {
            pub fn emit(self) {
                panic!("{}", &self.message)
            }
        }

        impl Into<MacroError> for String {
            fn into(self) -> MacroError {
                MacroError { message: self }
            }
        }

        impl Into<MacroError> for (Span, String) {
            fn into(self) -> MacroError {
                MacroError {
                    message: self.1,
                }
            }
        }
    }
}

#[derive(Default)]
pub struct PathVec(Vec<Path>);

impl Parse for PathVec {
    fn parse(input: ParseStream) -> parse::Result<Self> {
        let paths = Punctuated::<Path, Token![,]>::parse_terminated(input)?;

        Ok(PathVec(paths.into_iter().collect()))
    }
}

impl IntoIterator for PathVec {
    type Item = Path;
    type IntoIter = std::vec::IntoIter<Path>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl TryFrom<TokenStream> for PathVec {
    type Error = MacroError;

    fn try_from(stream: TokenStream) -> Result<Self, Self::Error> {
        if stream.is_empty() {
            return Ok(Self::default());
        }

        parse::<PathVec>(stream).map_err(|e| e.to_string().into())
    }
}

pub fn path_to_string(path: &Path) -> String {
    let mut s = String::new();

    for segment in path.segments.iter() {
        if !s.is_empty() {
            s += "::";
        }

        s += &segment.ident.to_string();
    }

    s
}

pub fn last_segment_in_path(path: &Path) -> &PathSegment {
    path.segments
        .iter()
        .last()
        .expect("expected at least one segment in path")
}
