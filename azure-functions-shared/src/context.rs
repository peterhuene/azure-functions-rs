use std::env;
use std::path::PathBuf;

/// Represents context about an Azure Function invocation.
#[derive(Debug)]
pub struct Context<'a> {
    invocation_id: &'a str,
    function_id: &'a str,
    name: &'a str,
}

impl Context<'a> {
    /// Creates a new function invocation context.
    pub fn new(invocation_id: &'a str, function_id: &'a str, name: &'a str) -> Self {
        Context {
            invocation_id: invocation_id,
            function_id: function_id,
            name: name,
        }
    }

    /// Gets the invocation identifier for the current Azure Function.
    pub fn invocation_id(&self) -> &str {
        self.invocation_id
    }

    /// Gets the function identifier for the current Azure Function.
    pub fn function_id(&self) -> &str {
        self.function_id
    }

    /// Gets the name of the current Azure Function.
    pub fn function_name(&self) -> &str {
        self.name
    }

    /// Gets the directory for the current Azure Function.
    pub fn function_directory(&self) -> Option<PathBuf> {
        self.app_directory().map(|p| p.join(self.name))
    }

    /// Gets the directory for the current Azure Function Application.
    pub fn app_directory(&self) -> Option<PathBuf> {
        env::current_exe()
            .map(|p| p.parent().map(|p| p.to_owned()))
            .ok()
            .unwrap_or(None)
    }
}
