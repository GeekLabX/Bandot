#!/usr/bin/env bash
set -e

pushd .

# The following line ensure we run from the project root
PROJECT_ROOT=`git rev-parse --show-toplevel`
cd $PROJECT_ROOT

# Build the image
echo "Building ${GITUSER}/${GITREPO}:latest docker image, hang on!"
time docker build -f ./bandot-node/Dockerfile --build-arg PROFILE=release -t bandot/bandot:latest .

# Show the list of available images for this repo
echo "Image is ready"
docker images | grep bandot

popd
