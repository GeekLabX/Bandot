# bandot-node 2.0

A new SRML-based Substrate node, ready for hacking.

# Building

Install Rust:

```bash
curl https://sh.rustup.rs -sSf | sh
```

Install required tools:

```bash
./scripts/init.sh
```

# Bandot Docker

## Building the image

To build your own image from the source, you can run the following command:
```bash
./docker/build.sh
```

NOTE: Building the image takes a while. Count at least 30min on a good machine.

## Start a Bandot docker container

Run the following command
```
docker-compose -f docker/docker-compose-local.yml up -d
```
You can access the UI via http://localhost:3000

## Operation Manual

[Operation Manual](https://github.com/bandotorg/Bandot/blob/master/Operation_Manual.md)
