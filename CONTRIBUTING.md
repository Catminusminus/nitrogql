Thank you for your interest in contributing to this project. Below are instructions for setting up your development environment.

## Prerequisites

This project is written in Rust. You will need to install Rust and Cargo to build this project. You can find instructions for installing Rust [here](https://www.rust-lang.org/tools/install).

Also, this project maintains Node.js integrations. You will need to install Node.js and npm to build the Node.js integrations. You can find instructions for installing Node.js [here](https://nodejs.org/en/download/).

## Build and Test

Usual commands like `cargo build`, `cargo test` or `cargo watch` (you need [cargo-watch](https://crates.io/crates/cargo-watch) for this) should work.

### Snapshot testing

When you want to update snapshot tests, [cargo-insta](https://crates.io/crates/cargo-insta) is required. Instead of `cargo test`, run `cargo insta test` for snapshot-testing-aware results. When snapshots are to be added or updated, run `cargo insta review` to review and apply the changes.

### Coverage

To generate coverage file, run `make coverage`. This will generate a `coverage/lcov.info` file which can be loaded by the [Coverage Gutters](https://marketplace.visualstudio.com/items?itemName=ryanluker.vscode-coverage-gutters) extension.

Note that this requires below dependencies to be installed:

- `rustup component add llvm-tools-preview`
- `cargo install grcov`

## Building the Node.js integration

We have a small set of scripts for building the Node.js integration. You need to have `wasm-opt` and `wasm-snip` installed.

1. `./build/build-rust.sh` to build Rust code into WASM.
2. `./build/build-node.sh` to copy the WASM file into the Node.js packages.