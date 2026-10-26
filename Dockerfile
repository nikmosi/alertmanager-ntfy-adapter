FROM rust:1.99.1 AS builder

RUN apt-get update && apt-get install -y --no-install-recommends \
    musl-tools \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

RUN rustup target add x86_64-unknown-linux-musl

RUN echo "adapter:x:10001:10001:adapter:/nonexistent:/bin/false" > /etc/passwd_adapter

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY src ./src

RUN cargo build --release --target x86_64-unknown-linux-musl --bin alertmanager-ntfy-adapter

FROM scratch

COPY --from=builder /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/ca-certificates.crt
COPY --from=builder /etc/passwd_adapter /etc/passwd
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/alertmanager-ntfy-adapter /alertmanager-ntfy-adapter

USER 10001:10001
EXPOSE 8080

ENTRYPOINT ["/alertmanager-ntfy-adapter"]
