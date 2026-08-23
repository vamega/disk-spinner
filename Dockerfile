FROM rust:1-trixie AS builder

WORKDIR /app

# Fetch dependencies into a cacheable layer
COPY Cargo.toml Cargo.lock .cargo ./

RUN mkdir src \
 && echo 'fn main() { println!("dummy"); }' > src/main.rs \
 && cargo build --release --locked \
 && rm -Rf src \
 && rm -Rf target/release/.fingerprint/disk-spinner-*

# Now copy the code and build it
COPY src/ src/

RUN cargo build --release --locked


FROM debian:trixie-slim

COPY --from=builder /app/target/release/disk-spinner /usr/local/bin/disk-spinner

ENTRYPOINT ["/usr/local/bin/disk-spinner"]

CMD ["--help"]
