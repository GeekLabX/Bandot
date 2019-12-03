# bandot-node 2.0

A new SRML-based Substrate node, ready for hacking.

# Bandot Docker

## Building the bandot node image

To build your own image from the source, you can run the following command:
```bash
./docker/build-node.sh
```
NOTE: Building the image takes a while. Count at least 30min on a good machine.

## Building the bandot ui image

To build your own image from the source, you can run the following command:
```bash
./docker/build-ui.sh
```

## Start bandot docker container

Run the following command
```
docker-compose -f docker/docker-compose.yml up -d
```
You can access the UI via http://localhost:3000
