FROM rust:1.81 AS builder
WORKDIR /usr/src/webfs
COPY . .
RUN cargo install --path .

FROM debian:bookworm
# alpine:3.20.3
RUN apt-get update && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/local/cargo/bin/webfs /usr/local/bin/webfs
ENV PORT=3000
EXPOSE ${PORT}
CMD ["webfs"]