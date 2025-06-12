#!/bin/sh
# This script updates the bindings for the docs.rs documentation.
# It should be run from the root of the repository.
#
# This script requires docker to be installed and running.
set -e

docker buildx build \
  --target docsrs_bindings \
  -o type=local,dest=. \
  --build-arg PHP_VERSION=8.3 \
  .
