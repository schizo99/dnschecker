# First stage: Build the binary
FROM rust:buster as builder
LABEL maintainer="schizo99@gmail.com"

WORKDIR /usr/src

# RUN rustup target add aarch64-unknown-linux-gnu
# RUN rustup toolchain install stable-aarch64-unknown-linux-gnu

# Copy over your manifest
COPY ./Cargo.toml ./Cargo.toml

# Create a guest user
RUN adduser --disabled-password --gecos '' guest


# Cache your dependencies
#RUN mkdir src && echo "fn main() {println!(\"if you see this, the build broke\")}" > src/main.rs && cargo build --release

# Now, remove the dummy src/main.rs, and replace with your actual source code
#RUN rm -f ./src/*.rs
COPY ./src ./src

# Build for release.
#RUN cargo build --target aarch64-unknown-linux-gnu --release
RUN cargo build --release

# Second stage: Copy the binary from the first stage
FROM debian:buster-slim

RUN apt update && apt install -y openssl ca-certificates && rm -rf /var/lib/apt/lists/*

#COPY --from=builder /usr/src/target/aarch64-unknown-linux-gnu/release/dnschecker /dnschecker
COPY --from=builder /usr/src/target/release/dnschecker /dnschecker
COPY --from=builder /etc/passwd /etc/passwd

USER guest
# Set the binary as the entrypoint of the container
ENTRYPOINT ["/dnschecker"]