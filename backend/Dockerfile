FROM rust:1.88-slim-bookworm AS builder

WORKDIR /app
RUN apt-get update \
    && apt-get install -y --no-install-recommends build-essential pkg-config \
    && rm -rf /var/lib/apt/lists/*
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo build --release --locked

FROM debian:bookworm-slim AS runtime

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates curl \
    && rm -rf /var/lib/apt/lists/*

RUN useradd --create-home --uid 10001 hookpop
COPY --from=builder /app/target/release/amazon-tiktok-studio-api /usr/local/bin/hookpop-api

USER hookpop
ENV BIND_ADDR=0.0.0.0:8787
EXPOSE 8787

CMD ["hookpop-api"]
