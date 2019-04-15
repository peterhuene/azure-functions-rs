# Contributing

This project is open for contributions. See the [issues](https://github.com/peterhuene/azure-functions-rs/issues) page for ideas on how you can help.

## Submitting Changes

To make changes to this project, please [fork the repo](https://help.github.com/en/articles/fork-a-repo) and make your changes on a branch in this repo. When you are ready to contribute, make your pull-request into the `dev` branch.

### Before you make a Pull Request

- [ ] You have pushed all your code to the remote repo.
- [ ] Your fork is up-to-date with the source repository (`peterhuene/azure-functions-rs`)
- [ ] All tests pass.
- [ ] "TODO" comments are removed.
- [ ] Temp variables are given good names.
- [ ] Any merge conflicts have the potential to be reasonably sorted out.

If you are uncertain about your contribution, that's ok! You can always make a [draft pull request](https://help.github.com/en/articles/about-pull-requests#draft-pull-requests).

## Repository Layout

This repository is split into multiple Rust crates:

- [azure-functions](https://github.com/peterhuene/azure-functions-rs/tree/master/azure-functions) - The `azure-functions` crate that defines the types and functions that are used when writing Azure Functions with Rust.
- [azure-functions-codegen](https://github.com/peterhuene/azure-functions-rs/tree/master/azure-functions-codegen) - The `azure-functions-codegen` crate that defines the procedural macros that are used when writing Azure Functions with Rust.
- [azure-functions-sdk](https://github.com/peterhuene/azure-functions-rs/tree/master/azure-functions-sdk) - The `azure-functions-sdk` crate that implements the `cargo func` command.
- [azure-functions-shared](https://github.com/peterhuene/azure-functions-rs/tree/master/azure-functions-shared) - The `azure-functions-shared` crate that defines types and functions that are shared between the `azure-functions-codegen` and `azure-functions` crates.
  - Note: the `azure-functions-shared/protobuf` directory is the git submodule for [Azure Functions Language Worker Protocol](https://github.com/Azure/azure-functions-language-worker-protobuf).
- [azure-functions-shared-codegen](https://github.com/peterhuene/azure-functions-rs/tree/master/azure-functions-shared-codegen) - The `azure-functions-shared-codegen` crate that defines the procedural macros used by the shared `azure-functions-shared` crate.
- [examples](https://github.com/peterhuene/azure-functions-rs/tree/master/examples) - The directory containing example Azure Functions.

## Setting Up a Dev Environment

### Cloning the Repository

This repository uses a git submodule for defining the [Azure Functions Language Worker Protocol](https://github.com/Azure/azure-functions-language-worker-protobuf).

Use `--recurse-submodules` when cloning this repository:

Cloning with SSH:

``` bash
git clone --recurse-submodules git@github.com:<GITHUB-USERNAME>/azure-functions-rs.git
```

Cloning with HTTPS:

``` bash
git clone --recurse-submodules https://github.com/<GITHUB-USERNAME>/azure-functions-rs.git
```

If you want to clone the source repository, replace `<GITHUB-USERNAME>` with `peterhuene`. To clone your own fork, replace the value with your GitHub username.

### Building

Build at the root of the repository to build both the `azure-functions-codegen` and the `azure-functions` libraries using `cargo build`:

``` bash
cargo build
```

### Running tests

Use `cargo test` to run the tests:

``` bash
cargo test
```

### Updating Your Fork

As you work on your contributions, code on the source repository may get updated. You can keep your fork up-to-date and avoid merge conflict by adding the source repo as a remote, upstream branch.

``` bash
git remote add upstream https://github.com/peterhuene/azure-functions-rs
```

You only need to do this once. Then, to update your fork fetch all branches of the source repo

``` bash
git fetch upstream
```

Switch to `master` and rebase so that any commits you have made which aren't in upstream/master are replaced on top of that other branch:

``` bash
git checkout master
git rebase upstream/master
```

> :warning: Note that this will not update the Azure Functions Language Worker Protocol. However, that code is much less likely to change.