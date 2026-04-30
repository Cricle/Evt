.PHONY: all build build-web run test clean fmt tauri-build docker-build docker-build-allinone help

APP_BIN = target/release/paopao-ce
WEB_DIR = web

-include .env
export

all: fmt test build

build:
	@cargo build --release --bin paopao-ce

build-web:
	@cd $(WEB_DIR) && npm config set registry "$(NPM_REGISTRY)" && npm config set @opentiny:registry "https://registry.npmjs.org/" && corepack yarn config set registry "$(NPM_REGISTRY)" && (corepack yarn install || (npm config set registry "https://registry.npmjs.org/" && corepack yarn config set registry "https://registry.npmjs.org/" && corepack yarn install)) && VITE_HOST="$(VITE_HOST)" corepack yarn build

run:
	@cargo run --bin paopao-ce

test:
	@cargo test --workspace

clean:
	@cargo clean

fmt:
	@cargo fmt --all

tauri-build:
	@cd $(WEB_DIR) && npm config set registry "$(NPM_REGISTRY)" && npm config set @opentiny:registry "https://registry.npmjs.org/" && corepack yarn config set registry "$(NPM_REGISTRY)" && (corepack yarn install || (npm config set registry "https://registry.npmjs.org/" && corepack yarn config set registry "https://registry.npmjs.org/" && corepack yarn install)) && corepack yarn tauri build

docker-build:
	@docker build \
		--build-arg NODE_VERSION="$(NODE_VERSION)" \
		--build-arg RUST_VERSION="$(RUST_VERSION)" \
		--build-arg ALPINE_VERSION="$(ALPINE_VERSION)" \
		--build-arg NPM_REGISTRY="$(NPM_REGISTRY)" \
		--build-arg VITE_HOST="$(VITE_HOST)" \
		-f Dockerfile .

docker-build-allinone:
	@docker build \
		--build-arg NODE_VERSION="$(NODE_VERSION)" \
		--build-arg RUST_VERSION="$(RUST_VERSION)" \
		--build-arg ALPINE_VERSION="$(ALPINE_VERSION)" \
		--build-arg NPM_REGISTRY="$(NPM_REGISTRY)" \
		--build-arg VITE_HOST="$(VITE_HOST)" \
		-f Dockerfile.allinone .

help:
	@echo "make build: build the Rust backend"
	@echo "make build-web: build the Vue web bundle for same-origin serving"
	@echo "make run: start the Rust backend"
	@echo "make test: run the Rust workspace tests"
	@echo "make tauri-build: build the desktop shell"
