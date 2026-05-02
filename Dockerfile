ARG RUST_VERSION=rust:alpine3.22
ARG NODE_VERSION=22-alpine
ARG ALPINE_VERSION=3.22
ARG HTTP_PROXY=
ARG HTTPS_PROXY=
ARG NO_PROXY=
ARG ALL_PROXY=

FROM ${RUST_VERSION} AS rust-builder

ARG ALPINE_MIRROR=https://mirrors.tuna.tsinghua.edu.cn/alpine
ARG HTTP_PROXY=
ARG HTTPS_PROXY=
ARG NO_PROXY=
ARG ALL_PROXY=

ENV HTTP_PROXY=${HTTP_PROXY}
ENV HTTPS_PROXY=${HTTPS_PROXY}
ENV NO_PROXY=${NO_PROXY}
ENV ALL_PROXY=${ALL_PROXY}
ENV http_proxy=${HTTP_PROXY}
ENV https_proxy=${HTTPS_PROXY}
ENV no_proxy=${NO_PROXY}
ENV all_proxy=${ALL_PROXY}

RUN sed -i "s|https://dl-cdn.alpinelinux.org/alpine|${ALPINE_MIRROR}|g" /etc/apk/repositories \
    && HTTP_PROXY=${HTTP_PROXY} HTTPS_PROXY=${HTTPS_PROXY} http_proxy=${HTTP_PROXY} https_proxy=${HTTPS_PROXY} \
    apk add --no-cache musl-dev openssl-dev pkgconfig perl make g++ git

WORKDIR /app

COPY .cargo ./.cargo
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates
COPY config ./config
COPY migrations ./migrations
COPY proto ./proto
COPY docs ./docs

RUN cargo build --release --bin evt

FROM node:${NODE_VERSION} AS web-builder

ARG NPM_REGISTRY=https://mirrors.tuna.tsinghua.edu.cn/npm/
ARG VITE_HOST=
ARG HTTP_PROXY=
ARG HTTPS_PROXY=
ARG NO_PROXY=
ARG ALL_PROXY=

WORKDIR /app/web

ENV HTTP_PROXY=${HTTP_PROXY}
ENV HTTPS_PROXY=${HTTPS_PROXY}
ENV NO_PROXY=${NO_PROXY}
ENV ALL_PROXY=${ALL_PROXY}
ENV http_proxy=${HTTP_PROXY}
ENV https_proxy=${HTTPS_PROXY}
ENV no_proxy=${NO_PROXY}
ENV all_proxy=${ALL_PROXY}

RUN npm config set registry "${NPM_REGISTRY}" \
    && npm config set @opentiny:registry "https://registry.npmjs.org/" \
    && corepack enable

COPY web/package.json web/yarn.lock ./
COPY .yarnrc /app/.yarnrc

RUN yarn config set registry "${NPM_REGISTRY}" \
    && yarn install --frozen-lockfile

COPY web ./

RUN VITE_HOST="${VITE_HOST}" yarn build

FROM alpine:${ALPINE_VERSION} AS runtime

ARG ALPINE_MIRROR=https://mirrors.tuna.tsinghua.edu.cn/alpine
ARG HTTP_PROXY=
ARG HTTPS_PROXY=
ARG NO_PROXY=
ARG ALL_PROXY=

ENV EVT_RS__SERVER__HTTP__HOST=0.0.0.0
ENV EVT_RS__SERVER__HTTP__PORT=8008
ENV EVT_RS__SERVER__GRPC__HOST=0.0.0.0
ENV EVT_RS__SERVER__GRPC__PORT=18020
ENV HTTP_PROXY=${HTTP_PROXY}
ENV HTTPS_PROXY=${HTTPS_PROXY}
ENV NO_PROXY=${NO_PROXY}
ENV ALL_PROXY=${ALL_PROXY}
ENV http_proxy=${HTTP_PROXY}
ENV https_proxy=${HTTPS_PROXY}
ENV no_proxy=${NO_PROXY}
ENV all_proxy=${ALL_PROXY}

RUN sed -i "s|https://dl-cdn.alpinelinux.org/alpine|${ALPINE_MIRROR}|g" /etc/apk/repositories \
    && HTTP_PROXY=${HTTP_PROXY} HTTPS_PROXY=${HTTPS_PROXY} http_proxy=${HTTP_PROXY} https_proxy=${HTTPS_PROXY} \
    apk add --no-cache ca-certificates libgcc libstdc++

WORKDIR /app

COPY --from=rust-builder /app/target/release/evt ./evt
COPY --from=rust-builder /app/config ./config
COPY --from=rust-builder /app/migrations ./migrations
COPY --from=rust-builder /app/docs/openapi ./docs/openapi
COPY --from=rust-builder /app/docs/openapi.json ./docs/openapi.json
COPY --from=web-builder /app/web/dist ./web/dist

VOLUME ["/app/custom"]
EXPOSE 8008 18020

ENTRYPOINT ["/app/evt"]
