mod attribute_arguments;
pub mod bindings;
mod function;
mod quotable;

pub use self::attribute_arguments::*;
pub use self::function::*;

pub trait TryFrom<T>: std::marker::Sized {
    type Error;

    fn try_from(item: T) -> Result<Self, Self::Error>;
}
