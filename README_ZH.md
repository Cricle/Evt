# PaoPao CE

PaoPao CE 现已切换为 Rust + Vue 3 技术栈。仓库内唯一后端实现是 Rust，Web 端使用 Vue 3，桌面端使用 Tauri 作为同一套 HTTP API 的壳。

## 目录结构

- `crates/`：Rust 后端工作区
- `config/`：后端默认配置与本地示例配置
- `migrations/`：SQLx 数据库迁移
- `proto/`：gRPC 使用的 protobuf 定义
- `web/`：Vue 3 Web 前端与 Tauri 桌面端
- `docs/openapi/`：OpenAPI 静态查看器资源

## 环境要求

- Rust stable
- Node.js 22+
- MySQL 8.0+

## 本地开发

1. 启动 MySQL，并创建 `paopao` 数据库。
2. 将 `config/local.example.toml` 复制为 `config/local.toml`，按实际环境修改连接信息。
3. 先构建前端静态资源：

```sh
make build-web
```

4. 启动后端：

```sh
make run
```

默认访问地址：

- Web：`http://127.0.0.1:8008/`
- API 根路径：`http://127.0.0.1:8008/v1`
- OpenAPI JSON：`http://127.0.0.1:8008/docs/openapi.json`

## 常用命令

```sh
make fmt
make test
make build
make tauri-build
```

## Docker

```sh
docker compose up --build
```

该方式会启动 MySQL 和 Rust 后端，并由同一个进程在 `8008` 端口提供 API 与已编译的 Web 静态资源。
