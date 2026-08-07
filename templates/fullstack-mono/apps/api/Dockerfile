FROM rust:1.96.1-slim-bookworm AS builder
WORKDIR /app
COPY . .
RUN cargo build --locked --release -p api

FROM debian:bookworm-slim AS runtime
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates curl \
    && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/api /usr/local/bin/api
ENV HOST=0.0.0.0 PORT=3001 RUST_LOG=api=info,tower_http=info
EXPOSE 3001
USER 65532:65532
ENTRYPOINT ["api"]
