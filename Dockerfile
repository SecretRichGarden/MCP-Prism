FROM rust:1.92-bookworm AS builder
WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY docs ./docs
COPY .env.example ./

RUN cargo build --release

FROM debian:bookworm-slim
WORKDIR /app

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates curl \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/mcp-prism /usr/local/bin/mcp-prism
COPY .env.example /app/.env.example

EXPOSE 8787

CMD ["mcp-prism", "serve"]

