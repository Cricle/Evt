FROM scratch

ENV EVT_RS__SERVER__HTTP__HOST=0.0.0.0
ENV EVT_RS__SERVER__HTTP__PORT=8008
ENV EVT_RS__SERVER__GRPC__HOST=0.0.0.0
ENV EVT_RS__SERVER__GRPC__PORT=18020

WORKDIR /app

COPY target/release/evt ./evt
COPY config ./config
COPY migrations ./migrations
COPY web/dist ./web/dist
COPY docs/openapi ./docs/openapi
COPY docs/openapi.json ./docs/openapi.json
COPY docker-runtime/lib /lib
COPY docker-runtime/lib64 /lib64
COPY docker-runtime/etc/ssl/certs /etc/ssl/certs

VOLUME ["/app/custom"]
EXPOSE 8008 18020
ENTRYPOINT ["/app/evt"]
