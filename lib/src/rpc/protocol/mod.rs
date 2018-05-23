use azure_functions_codegen::mod_path;

#[mod_path]
mod FunctionRpc {}
#[mod_path]
mod FunctionRpc_grpc {}

pub use self::FunctionRpc::*;
pub use self::FunctionRpc_grpc::*;
