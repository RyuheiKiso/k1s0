# Build stage
FROM rust:1.82-bookworm AS builder

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo 'fn main() {}' > src/main.rs && cargo build --release && rm -rf src

COPY . .
RUN touch src/main.rs && cargo build --release

# Runtime stage
FROM gcr.io/distroless/cc-debian12:nonroot

COPY --from=builder /app/target/release/config /config

USER nonroot:nonroot
EXPOSE 8080

ENTRYPOINT ["/config"]
