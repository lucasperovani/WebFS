ARG RUST_VERSION="1.81"

FROM rust:${RUST_VERSION}-slim-bookworm AS builder
WORKDIR /app
COPY . .
RUN \
    --mount=type=cache,target=/app/target \
    --mount=type=cache,target=/usr/local/cargo/registry \
    cargo build --release && \
    cp ./target/release/webfs /


FROM alpine:3.20.3 AS final

COPY --from=builder /webfs /usr/local/bin
COPY --from=builder /app/assets /opt/webfs/assets

RUN adduser \
    --disabled-password \
    --gecos "" \
    --home /webfs \
    --shell /sbin/nologin \
    --no-create-home \
    --uid 10001 \
    webfsuser && \
    chown webfsuser /usr/local/bin/webfs && \
    chown -R webfsuser /opt/webfs && \
    apk upgrade --no-cache && \
    apk add --no-cache libgcc libc6-compat

USER webfsuser
ENV PORT="3000/tcp"
WORKDIR /opt/webfs
ENTRYPOINT ["webfs"]
EXPOSE ${PORT}
