# Stage 1: Build the Rust Node
FROM rust:1.85-slim AS builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    protobuf-compiler \
    git \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /usr/src/aamn
COPY . .

# Build for release
RUN cargo build --release

# Stage 2: Final Runtime Image
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    libssl3 \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /usr/src/aamn/target/release/aamn /usr/local/bin/aamn

# Ports:
# 9000: Node P2P
# 1080: SOCKS5 Proxy
# 50051: gRPC API
# 50052: REST API
EXPOSE 9000 1080 50051 50052

ENTRYPOINT ["aamn"]
CMD ["start", "--port", "9000", "--socks5-port", "1080", "--grpc-port", "50051"]
