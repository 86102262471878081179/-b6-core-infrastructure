# ── Stage 1: Builder ────────────────────────────────────────────────────────
FROM rust:1.83-alpine AS builder

RUN apk add --no-cache musl-dev gcc pkgconfig

WORKDIR /usr/src/rust_kernel

COPY Cargo.toml ./
RUN mkdir src && echo "fn main() {}" > src/main.rs && \
    cargo build --release && \
    rm -rf src

COPY src ./src
RUN touch src/main.rs && cargo build --release

# ── Stage 2: Runtime ────────────────────────────────────────────────────────
FROM alpine:3.20

RUN apk add --no-cache ca-certificates libgcc netcat-openbsd curl

WORKDIR /app
# ACHTUNG: Hier steht jetzt b6_kernel_node, nicht rust_kernel!
COPY --from=builder /usr/src/rust_kernel/target/release/b6_kernel_node /app/b6_kernel_node

RUN mkdir -p /var/lib/worm_vault
VOLUME ["/var/lib/worm_vault"]

CMD ["/app/b6_kernel_node"]
