FROM rust:1.89-slim AS builder

WORKDIR /build

COPY Cargo.toml Cargo.lock* ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release

COPY . .

RUN cargo build --release

RUN BIN=target/release/io && \
    SIZE=$(stat -c%s "$BIN") && \
    echo "Binary size: $SIZE bytes"


FROM debian:stable-slim AS flight

WORKDIR /flight

COPY --from=builder /build/target/release/io /flight/io

CMD ["/flight/io"]
