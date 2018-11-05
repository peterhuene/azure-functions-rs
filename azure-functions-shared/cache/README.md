# Protobuf Cache

This directory exists to cache the output of a previous protobuf compilation.

If you do not have `protoc` on your PATH, the cached protobuf definitions will be used.

This is primarily used for crate publishing so that [docs.rs](https://docs.rs) can
generate documentation without having a protobuf compiler installed.
