extern crate protoc_grpcio;

use std::env;

fn main() {
    println!(
        "cargo:rerun-if-changed={}",
        "protobuf/src/proto/FunctionRpc.proto"
    );
    protoc_grpcio::compile_grpc_protos(
        &["FunctionRpc.proto"],
        &["protobuf/src/proto"],
        env::var("OUT_DIR").unwrap(),
    ).expect("Failed to compile gRPC definitions.");
}
