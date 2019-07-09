//! Module for function invocation context.
use std::{cell::RefCell, env, path::PathBuf};

pub(crate) const UNKNOWN_FUNCTION: &str = "<unknown>";

thread_local!(pub(crate) static CURRENT: RefCell<Context> = RefCell::new(
    Context{
        invocation_id: String::new(),
        function_id: String::new(),
        function_name: UNKNOWN_FUNCTION
    }
));

/// Represents context about an Azure Function invocation.
#[derive(Debug, Clone)]
pub struct Context {
    pub(crate) invocation_id: String,
    pub(crate) function_id: String,
    pub(crate) function_name: &'static str,
}

pub(crate) struct ContextGuard;

impl Drop for ContextGuard {
    fn drop(&mut self) {
        Context::clear();
    }
}

impl Context {
    /// Gets the current invocation context.
    ///
    /// Returns None if there is no invocation context.
    pub fn current() -> Option<Self> {
        let mut current = None;

        CURRENT.with(|c| {
            let c = c.borrow();
            if !c.invocation_id.is_empty() {
                current = Some(c.clone());
            }
        });

        current
    }

    #[must_use]
    pub(crate) fn set(
        invocation_id: &str,
        function_id: &str,
        function_name: &'static str,
    ) -> ContextGuard {
        CURRENT.with(|c| {
            let mut c = c.borrow_mut();
            c.invocation_id.replace_range(.., invocation_id);
            c.function_id.replace_range(.., function_id);
            c.function_name = function_name;
        });

        ContextGuard {}
    }

    pub(crate) fn clear() {
        CURRENT.with(|c| {
            let mut c = c.borrow_mut();
            c.invocation_id.clear();
            c.function_id.clear();
            c.function_name = UNKNOWN_FUNCTION;
        });
    }

    /// Gets the invocation identifier for the current Azure Function.
    pub fn invocation_id(&self) -> &str {
        &self.invocation_id
    }

    /// Gets the function identifier for the current Azure Function.
    pub fn function_id(&self) -> &str {
        &self.function_id
    }

    /// Gets the name of the current Azure Function.
    pub fn function_name(&self) -> &str {
        self.function_name
    }

    /// Gets the directory for the current Azure Function.
    pub fn function_directory(&self) -> Option<PathBuf> {
        self.app_directory().map(|p| p.join(self.function_name))
    }

    /// Gets the directory for the current Azure Function Application.
    pub fn app_directory(&self) -> Option<PathBuf> {
        env::current_exe()
            .map(|p| p.parent().map(ToOwned::to_owned))
            .ok()
            .unwrap_or(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_returns_none_without_context() {
        assert_eq!(Context::current().is_none(), true);
    }

    #[test]
    fn it_returns_current_context() {
        let _guard = Context::set("1234", "5678", "foo");

        let context = Context::current().unwrap();

        assert_eq!(context.invocation_id(), "1234");
        assert_eq!(context.function_id(), "5678");
        assert_eq!(context.function_name(), "foo");
    }
}
