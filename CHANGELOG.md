# Changelog

## 2026-04-30

- Replaced the legacy backend tree with the Rust workspace at the repository root.
- Kept the web frontend and Tauri shell at the repository root, both targeting the same Rust HTTP API.
- Switched repository build, Docker, and CI entry points to Rust.
