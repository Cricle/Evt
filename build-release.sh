#!/bin/sh

set -eu

if [ -f ./.env ]; then
  set -a
  . ./.env
  set +a
fi

make build-web
cargo build --release --bin paopao-ce
