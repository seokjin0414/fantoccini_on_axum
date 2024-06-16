ARG RUST_VERSION=1.78.0
ARG APP_NAME=bems-svr
FROM rust:${RUST_VERSION}-alpine AS build
ARG APP_NAME
WORKDIR /app
RUN apk add --no-cache build-base ca-certificates

RUN --mount=type=bind,source=src,target=src \
    --mount=type=bind,source=Cargo.toml,target=Cargo.toml \
    --mount=type=bind,source=Cargo.lock,target=Cargo.lock \
    --mount=type=cache,target=/app/target/ \
    --mount=type=cache,target=/usr/local/cargo/registry/ \
    <<EOF
set -e
cargo build --locked --release
cp ./target/release/$APP_NAME /bin/server
EOF

FROM scratch
COPY --from=build /bin/server /server
COPY --from=build /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/
EXPOSE 30737
CMD ["/server"]