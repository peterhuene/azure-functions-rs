extern crate protoc_grpcio;

use std::env;
use std::fs;
use std::path::PathBuf;

const OUT_DIR_VAR: &'static str = "OUT_DIR";
const CACHE_DIR_NAME: &'static str = "cache";
const PROTOBUF_INPUT_FILE: &'static str = "FunctionRpc.proto";
const RUST_PROTOBUF_FILE: &'static str = "FunctionRpc.rs";
const RUST_GRPC_FILE: &'static str = "FunctionRpc_grpc.rs";

fn compile_protobufs(out_dir: &PathBuf, cache_dir: &PathBuf) {
    protoc_grpcio::compile_grpc_protos(
        &[PROTOBUF_INPUT_FILE],
        &["protobuf/src/proto"],
        &out_dir,
        None,
    )
    .expect("Failed to compile gRPC definitions.");

    fs::copy(
        out_dir.join(RUST_PROTOBUF_FILE),
        cache_dir.join(RUST_PROTOBUF_FILE),
    )
    .expect(&format!("can't update cache file '{}'", RUST_PROTOBUF_FILE));

    fs::copy(out_dir.join(RUST_GRPC_FILE), cache_dir.join(RUST_GRPC_FILE))
        .expect(&format!("can't update cache file '{}'", RUST_GRPC_FILE));
}

fn use_cached_files(out_dir: &PathBuf, cache_dir: &PathBuf) {
    fs::copy(
        cache_dir.join(RUST_PROTOBUF_FILE),
        out_dir.join(RUST_PROTOBUF_FILE),
    )
    .expect(&format!(
        "can't copy cache file '{}' to output directory",
        RUST_PROTOBUF_FILE
    ));

    fs::copy(cache_dir.join(RUST_GRPC_FILE), out_dir.join(RUST_GRPC_FILE)).expect(&format!(
        "can't copy cache file '{}' to output directory",
        RUST_GRPC_FILE
    ));
}

fn main() {
    println!("cargo:rerun-if-changed=protobuf/src/proto/FunctionRpc.proto");

    let out_dir = PathBuf::from(env::var(OUT_DIR_VAR).unwrap());

    let cache_dir = env::current_dir()
        .expect("couldn't determine current working directory")
        .join(CACHE_DIR_NAME);

    fs::create_dir_all(&cache_dir).expect("failed to create cache directory");

    if cfg!(feature = "compile_protobufs") {
        compile_protobufs(&out_dir, &cache_dir);
    } else {
        use_cached_files(&out_dir, &cache_dir);
    }
}
