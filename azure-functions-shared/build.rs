extern crate protoc_grpcio;
extern crate reqwest;

use std::env;
use std::fs;
use std::fs::File;
use std::path::PathBuf;

// TODO: revisit using this as the source for the cached files
const BASE_URL: &'static str =
    "https://raw.githubusercontent.com/peterhuene/azure-functions-rs/master/cache/";
const OUT_DIR_VAR: &'static str = "OUT_DIR";
const CACHE_DIR_NAME: &'static str = "cache";
const PROTOBUF_INPUT_FILE: &'static str = "FunctionRpc.proto";
const RUST_PROTOBUF_FILE: &'static str = "FunctionRpc.rs";
const RUST_GRPC_FILE: &'static str = "FunctionRpc_grpc.rs";

fn download_file(name: &str) {
    let out_dir = PathBuf::from(env::var(OUT_DIR_VAR).unwrap());

    let mut output = File::create(out_dir.join(name))
        .expect(&format!("failed to create '{}' in output directory", name));

    reqwest::get(&(BASE_URL.to_string() + name))
        .expect(&format!("failed to download '{}' from the cache", name))
        .error_for_status()
        .expect("server responded with an error")
        .copy_to(&mut output)
        .expect(&format!(
            "failed to copy response to output file '{}'",
            name
        ));
}

fn download_cached_files() {
    download_file(RUST_PROTOBUF_FILE);
    download_file(RUST_GRPC_FILE);
}

fn compile_protobufs() {
    let out_dir = PathBuf::from(env::var(OUT_DIR_VAR).unwrap());

    protoc_grpcio::compile_grpc_protos(&[PROTOBUF_INPUT_FILE], &["protobuf/src/proto"], &out_dir)
        .expect("Failed to compile gRPC definitions.");

    let cache_dir = env::current_dir()
        .expect("couldn't determine current working directory")
        .parent()
        .expect("expect working directory is not the root")
        .join(CACHE_DIR_NAME);

    fs::create_dir_all(&cache_dir).expect("failed to create cache directory");

    fs::copy(
        out_dir.join(RUST_PROTOBUF_FILE),
        cache_dir.join(RUST_PROTOBUF_FILE),
    ).expect(&format!("can't update cache file '{}'", RUST_PROTOBUF_FILE));
    fs::copy(out_dir.join(RUST_GRPC_FILE), cache_dir.join(RUST_GRPC_FILE))
        .expect(&format!("can't update cache file '{}'", RUST_GRPC_FILE));
}

fn main() {
    println!("cargo:rerun-if-changed=protobuf/src/proto/FunctionRpc.proto");

    if cfg!(feature = "cached_protobufs") {
        download_cached_files();
    } else {
        compile_protobufs();
    }
}
