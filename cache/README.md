# Protobuf Cache

This directory exists to cache the output of a previous protobuf compilation.

This enables building the `azure-functions` crate without having a protobuf compiler installed (used to build crate documentation on docs.rs).

See the `cached_protobufs` feature of the crate.
