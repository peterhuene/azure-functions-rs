use std::env;
use std::fs;
use std::path::PathBuf;

const OUT_DIR_VAR: &str = "OUT_DIR";
const CACHE_DIR_NAME: &str = "cache";
const PROTOBUF_INPUT_FILES: &[&str] = &["FunctionRpc.proto"];
const OUTPUT_FILES: &[&str] = &["azure_functions_rpc_messages.rs"];

fn compile_protobufs(out_dir: &PathBuf, cache_dir: &PathBuf) {
    tower_grpc_build::Config::new()
        .enable_client(true)
        .build(PROTOBUF_INPUT_FILES, &["protobuf/src/proto"])
        .unwrap_or_else(|e| panic!("protobuf compilation failed: {}", e));

    for file in OUTPUT_FILES.iter() {
        fs::copy(out_dir.join(file), cache_dir.join(file))
            .expect(&format!("can't update cache file '{}'", file));
    }
}

fn use_cached_files(out_dir: &PathBuf, cache_dir: &PathBuf) {
    for file in OUTPUT_FILES.iter() {
        fs::copy(cache_dir.join(file), out_dir.join(file)).expect(&format!(
            "can't copy cache file '{}' to output directory",
            file
        ));
    }
}

fn main() {
    for file in PROTOBUF_INPUT_FILES {
        println!("cargo:rerun-if-changed=protobuf/src/proto/{}", file);
    }

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
