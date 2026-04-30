# PaoPao CE

PaoPao CE is a Rust + Vue 3 community platform. The repository now uses Rust as the only backend implementation, Vue 3 for the web client, and Tauri as a desktop shell over the same HTTP API.

## Repository layout

- `crates/`: Rust backend workspace
- `config/`: backend configuration defaults and local example overrides
- `migrations/`: SQLx migrations
- `proto/`: shared protobuf definitions used by the gRPC service
- `web/`: Vue 3 web app and Tauri desktop frontend
- `docs/openapi/`: static OpenAPI viewer assets

## Requirements

- Rust stable
- Node.js 22+
- MySQL 8.0+

## Local development

1. Start MySQL and create a database named `paopao`.
2. Copy `config/local.example.toml` to `config/local.toml` and update credentials.
3. Build the web bundle for same-origin serving:

```sh
make build-web
```

4. Start the backend:

```sh
make run
```

The backend serves:

- Web UI: `http://127.0.0.1:8008/`
- API root: `http://127.0.0.1:8008/v1`
- OpenAPI JSON: `http://127.0.0.1:8008/docs/openapi.json`

## Common commands

```sh
make fmt
make test
make build
make tauri-build
```

## Docker

Build and run the integrated image:

```sh
docker compose up --build
```

This starts MySQL and the Rust backend, with the compiled web assets served from the same process on port `8008`.
