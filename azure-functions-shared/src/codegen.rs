pub mod bindings;
mod function;
mod quotable;
mod value;

pub use self::function::*;
pub use self::value::*;

use proc_macro2::Span;
use syn::{spanned::Spanned, Ident, Lit, Meta, NestedMeta, Path, PathSegment};

pub fn last_segment_in_path(path: &Path) -> &PathSegment {
    path.segments
        .iter()
        .last()
        .expect("expected at least one segment in path")
}

pub fn iter_attribute_args<F>(args: &[NestedMeta], mut callback: F)
where
    F: FnMut(&Ident, &Lit) -> bool,
{
    for arg in args.iter() {
        match arg {
            NestedMeta::Meta(m) => {
                match m {
                    Meta::NameValue(nvp) => {
                        if !callback(&last_segment_in_path(&nvp.path).ident, &nvp.lit) {
                            return;
                        }
                    }
                    _ => macro_panic(m.span(), "expected name-value pair for an argument"),
                };
            }
            _ => macro_panic(arg.span(), "expected a name-vaule pair for an argument"),
        };
    }
}

pub fn get_string_value(name: &str, value: &Lit) -> String {
    if let Lit::Str(s) = value {
        return s.value();
    }

    macro_panic(
        value.span(),
        format!(
            "expected a literal string value for the '{}' argument",
            name
        ),
    )
}

pub fn get_boolean_value(name: &str, value: &Lit) -> bool {
    if let Lit::Bool(b) = value {
        return b.value;
    }

    macro_panic(
        value.span(),
        format!(
            "expected a literal boolean value for the '{}' argument",
            name
        ),
    )
}

pub fn get_integer_value(name: &str, value: &Lit) -> i64 {
    if let Lit::Int(i) = value {
        return i.base10_parse::<i64>().unwrap();
    }

    macro_panic(
        value.span(),
        format!(
            "expected a literal integer value for the '{}' argument",
            name
        ),
    )
}

#[cfg(feature = "unstable")]
pub fn macro_panic<T>(span: Span, message: T) -> !
where
    T: AsRef<str>,
{
    span.unstable().error(message.as_ref()).emit();
    panic!("aborting due to previous error");
}

#[cfg(not(feature = "unstable"))]
pub fn macro_panic<T>(_: Span, message: T) -> !
where
    T: AsRef<str>,
{
    panic!("{}", message.as_ref());
}

#[cfg(test)]
mod tests {
    use std::panic::{catch_unwind, UnwindSafe};

    pub fn should_panic<T>(callback: T, msg: &str)
    where
        T: FnOnce() + UnwindSafe,
    {
        let result = catch_unwind(|| callback());
        assert!(result.is_err(), "the function did not panic");

        if cfg!(feature = "unstable") {
            assert_eq!(
                result.unwrap_err().downcast_ref::<String>().unwrap(),
                "aborting due to previous error",
                "the panic message is not the expected one"
            );
        } else {
            assert_eq!(
                result.unwrap_err().downcast_ref::<String>().unwrap(),
                msg,
                "the panic message is not the expected one"
            );
        }
    }
}
