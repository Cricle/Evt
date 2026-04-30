ARG NODE_VERSION=22-alpine
ARG RUST_VERSION=rust:alpine3.22
ARG ALPINE_VERSION=3.22

FROM node:${NODE_VERSION} AS frontend
ARG VITE_HOST=
ARG NPM_REGISTRY=https://mirrors.tuna.tsinghua.edu.cn/npm/
WORKDIR /app/web
COPY .npmrc /root/.npmrc
COPY web/package.json web/yarn.lock ./
RUN env -u HTTP_PROXY -u HTTPS_PROXY -u ALL_PROXY -u http_proxy -u https_proxy -u all_proxy sh -c '\
    corepack enable && \
    npm config set registry "$NPM_REGISTRY" && \
    npm config set @opentiny:registry "https://registry.npmjs.org/" && \
    yarn config set registry "$NPM_REGISTRY" && \
    yarn install --network-timeout 600000 || \
    (npm config set registry "https://registry.npmjs.org/" && yarn config set registry "https://registry.npmjs.org/" && yarn install --network-timeout 600000)'
COPY web/ ./
RUN env -u HTTP_PROXY -u HTTPS_PROXY -u ALL_PROXY -u http_proxy -u https_proxy -u all_proxy sh -c '\
    printf "VITE_HOST=%s\n" "$VITE_HOST" > .env.local && \
    yarn build'

FROM ${RUST_VERSION} AS backend
WORKDIR /app
RUN printf '%s\n%s\n' \
    'https://mirrors.tuna.tsinghua.edu.cn/alpine/v3.22/main' \
    'https://mirrors.tuna.tsinghua.edu.cn/alpine/v3.22/community' \
    > /etc/apk/repositories
RUN env -u HTTP_PROXY -u HTTPS_PROXY -u ALL_PROXY -u http_proxy -u https_proxy -u all_proxy apk add --no-cache build-base pkgconfig ca-certificates
COPY .cargo ./.cargo
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates
COPY config ./config
COPY migrations ./migrations
COPY proto ./proto
COPY docs/openapi.json ./docs/openapi.json
COPY docs/openapi ./docs/openapi
COPY web ./web
COPY --from=frontend /app/web/dist ./web/dist
RUN env -u HTTP_PROXY -u HTTPS_PROXY -u ALL_PROXY -u http_proxy -u https_proxy -u all_proxy \
    cargo build --locked --release --bin evt

FROM scratch
ENV EVT_RS__SERVER__HTTP__HOST=0.0.0.0
ENV EVT_RS__SERVER__HTTP__PORT=8008
ENV EVT_RS__SERVER__GRPC__HOST=0.0.0.0
ENV EVT_RS__SERVER__GRPC__PORT=18020
WORKDIR /app
COPY --from=backend /app/target/release/evt ./evt
COPY --from=backend /app/config ./config
COPY --from=backend /app/migrations ./migrations
COPY --from=backend /app/web/dist ./web/dist
COPY --from=backend /app/docs/openapi ./docs/openapi
COPY --from=backend /app/docs/openapi.json ./docs/openapi.json
COPY --from=backend /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/ca-certificates.crt
VOLUME ["/app/custom"]
EXPOSE 8008 18020
ENTRYPOINT ["/app/evt"]
