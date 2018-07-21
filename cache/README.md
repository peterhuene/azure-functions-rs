# Protobuf Cache

This directory exists to cache the output of a previous protobuf compilation.

See the `cached_protobufs` feature of the crate of the `azure-functions-shared` crate.

The feature is primarily used for crate publishing so that [docs.rs](https://docs.rs) can
generate documentation without having a protobuf compiler installed.
