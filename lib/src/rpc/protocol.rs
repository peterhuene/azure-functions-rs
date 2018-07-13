use azure_functions_codegen::generated_mod;

#[generated_mod]
mod FunctionRpc {}
#[generated_mod]
mod FunctionRpc_grpc {}

pub use self::FunctionRpc::*;
pub use self::FunctionRpc_grpc::*;
