# Feature Status

The current repository is centered on the Rust backend and the existing Vue/Tauri clients.

- Web compatibility endpoints are implemented in Rust under `crates/http-api`.
- Database schema and compatibility helpers live in `migrations/`.
- Desktop packaging remains under `web/src-tauri` and uses the same HTTP API as the web client.
