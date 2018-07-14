use std::env;
use std::path::PathBuf;

/// Represents context about an Azure Function invocation.
///
/// # Examples
///
/// ```rust
/// # #![feature(proc_macro)] extern crate azure_functions;
/// # #[macro_use] extern crate log;
/// use azure_functions::{func, Context};
/// use azure_functions::bindings::HttpRequest;
///
/// #[func]
/// #[binding(name = "req", auth_level = "anonymous")]
/// pub fn log_context(context: &Context, req: &HttpRequest) {
///     info!("Context: {:?}", context);
/// }
/// ```
#[derive(Debug)]
pub struct Context<'a> {
    invocation_id: &'a str,
    function_id: &'a str,
    name: &'a str,
}

impl<'a> Context<'a> {
    pub(crate) fn new(invocation_id: &'a str, function_id: &'a str, name: &'a str) -> Self {
        Context {
            invocation_id: invocation_id,
            function_id: function_id,
            name: name,
        }
    }

    /// Gets the invocation identifier for the current Azure Function.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # #![feature(proc_macro)] extern crate azure_functions;
    /// # #[macro_use] extern crate log;
    /// use azure_functions::{func, Context};
    /// use azure_functions::bindings::HttpRequest;
    ///
    /// #[func]
    /// #[binding(name = "req", auth_level = "anonymous")]
    /// pub fn log_context(context: &Context, req: &HttpRequest) {
    ///     info!("Invocation ID: {}", context.invocation_id());
    /// }
    pub fn invocation_id(&self) -> &str {
        self.invocation_id
    }

    /// Gets the function identifier for the current Azure Function.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # #![feature(proc_macro)] extern crate azure_functions;
    /// # #[macro_use] extern crate log;
    /// use azure_functions::{func, Context};
    /// use azure_functions::bindings::HttpRequest;
    ///
    /// #[func]
    /// #[binding(name = "req", auth_level = "anonymous")]
    /// pub fn log_context(context: &Context, req: &HttpRequest) {
    ///     info!("Function ID: {}", context.function_id());
    /// }
    pub fn function_id(&self) -> &str {
        self.function_id
    }

    /// Gets the name of the current Azure Function.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # #![feature(proc_macro)] extern crate azure_functions;
    /// # #[macro_use] extern crate log;
    /// use azure_functions::{func, Context};
    /// use azure_functions::bindings::HttpRequest;
    ///
    /// #[func]
    /// #[binding(name = "req", auth_level = "anonymous")]
    /// pub fn log_context(context: &Context, req: &HttpRequest) {
    ///     info!("Function name: {}", context.function_name());
    /// }
    pub fn function_name(&self) -> &str {
        self.name
    }

    /// Gets the directory for the current Azure Function.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # #![feature(proc_macro)] extern crate azure_functions;
    /// # #[macro_use] extern crate log;
    /// use azure_functions::{func, Context};
    /// use azure_functions::bindings::HttpRequest;
    ///
    /// #[func]
    /// #[binding(name = "req", auth_level = "anonymous")]
    /// pub fn log_context(context: &Context, req: &HttpRequest) {
    ///     info!("Function directory: {}", context.function_directory().unwrap().display());
    /// }
    pub fn function_directory(&self) -> Option<PathBuf> {
        self.app_directory().map(|p| p.join(self.name))
    }

    /// Gets the directory for the current Azure Function Application.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # #![feature(proc_macro)] extern crate azure_functions;
    /// # #[macro_use] extern crate log;
    /// use azure_functions::{func, Context};
    /// use azure_functions::bindings::HttpRequest;
    ///
    /// #[func]
    /// #[binding(name = "req", auth_level = "anonymous")]
    /// pub fn log_context(context: &Context, req: &HttpRequest) {
    ///     info!("App directory: {}", context.app_directory().unwrap().display());
    /// }
    pub fn app_directory(&self) -> Option<PathBuf> {
        env::current_exe()
            .map(|p| p.parent().map(|p| p.to_owned()))
            .ok()
            .unwrap_or(None)
    }
}
