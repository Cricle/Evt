.PHONY: all build build-web run test test-web coverage-web coverage-rust coverage clean fmt tauri-build docker-build help

APP_BIN = target/release/evt
WEB_DIR = web

-include .env
export

all: fmt test build

build:
	@cargo build --release --bin evt

build-web:
	@cd $(WEB_DIR) && npm config set registry "$(NPM_REGISTRY)" && npm config set @opentiny:registry "https://registry.npmjs.org/" && corepack yarn config set registry "$(NPM_REGISTRY)" && (corepack yarn install || (npm config set registry "https://registry.npmjs.org/" && corepack yarn config set registry "https://registry.npmjs.org/" && corepack yarn install)) && VITE_HOST="$(VITE_HOST)" corepack yarn build

run:
	@cargo run --bin evt

test:
	@cargo test --workspace

test-web:
	@cd $(WEB_DIR) && corepack yarn test:unit && corepack yarn e2e

coverage-web:
	@cd $(WEB_DIR) && corepack yarn test:unit:coverage

coverage-rust:
	@cargo llvm-cov --workspace --html

coverage: coverage-rust coverage-web

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
		--build-arg ALPINE_MIRROR="$(ALPINE_MIRROR)" \
		--build-arg CARGO_REGISTRY_INDEX="$(CARGO_REGISTRY_INDEX)" \
		--build-arg VITE_HOST="$(VITE_HOST)" \
		-f Dockerfile .

help:
	@echo "make build: build the Rust backend"
	@echo "make build-web: build the Vue web bundle for same-origin serving"
	@echo "make run: start the Rust backend"
	@echo "make test: run the Rust workspace tests"
	@echo "make test-web: run frontend unit tests and browser E2E"
	@echo "make coverage-rust: generate Rust HTML coverage report"
	@echo "make coverage-web: generate frontend unit coverage report"
	@echo "make coverage: generate Rust and frontend coverage reports"
	@echo "make tauri-build: build the desktop shell"
