ARG RUST_VERSION="1.81"

FROM rust:${RUST_VERSION}-slim-bookworm AS builder
WORKDIR /app
COPY . .
RUN \
    --mount=type=cache,target=/app/target \
    --mount=type=cache,target=/usr/local/cargo/registry \
    cargo build --release && \
    cp ./target/release/webfs /

FROM debian:bookworm-slim AS final
RUN apt-get update && rm -rf /var/lib/apt/lists/*
RUN adduser \
    --disabled-password \
    --gecos "" \
    --home /webfs \
    --shell /sbin/nologin \
    --no-create-home \
    --uid 10001 \
    webfsuser

COPY --from=builder /webfs /usr/local/bin
COPY --from=builder /app/assets /opt/webfs/assets

RUN chown webfsuser /usr/local/bin/webfs
RUN chown -R webfsuser /opt/webfs

USER webfsuser
ENV PORT="3000/tcp"
WORKDIR /opt/webfs
ENTRYPOINT ["webfs"]
EXPOSE ${PORT}
