# Bandot Node

A new SRML-based Bandot node, ready for hacking.

## Build

Install Rust:

```bash
curl https://sh.rustup.rs -sSf | sh
```

Build Wasm and native code:

```bash
cargo build --release
```

### Single node development chain

Purge any existing developer chain state:

```bash
./target/release/bandot2 purge-chain --dev
```

Start a development chain with:

```bash
./target/release/bandot2 --dev
```

Detailed logs may be shown by running the node with the following environment variables set: `RUST_LOG=debug RUST_BACKTRACE=1 cargo run -- --dev`.

### Building the bandot node image

To build your own image from the source, you can run the following command:
```bash
../docker/build-node.sh
```
NOTE: Building the image takes a while. Count at least 30min on a good machine.

## Run in docker container

```
cd ..
docker-compose -f docker/docker-compose.yml up -d
```
