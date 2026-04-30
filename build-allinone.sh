#!/bin/sh

set -eu

if [ -f ./.env ]; then
  set -a
  . ./.env
  set +a
fi

IMAGE="${1:-${DOCKER_IMAGE:-bitbus/paopao-ce}}"
VERSION="${2:-${DOCKER_VERSION:-dev}}"

docker buildx build \
  --build-arg NODE_VERSION="${NODE_VERSION:-22-alpine}" \
  --build-arg RUST_VERSION="${RUST_VERSION:-rust:alpine3.22}" \
  --build-arg ALPINE_VERSION="${ALPINE_VERSION:-3.22}" \
  --build-arg NPM_REGISTRY="${NPM_REGISTRY:-https://mirrors.tuna.tsinghua.edu.cn/npm/}" \
  --build-arg VITE_HOST="${VITE_HOST:-http://127.0.0.1:8008}" \
  --tag "$IMAGE:all-in-one-${VERSION}" \
  --tag "$IMAGE:all-in-one-latest" \
  . -f Dockerfile.allinone
