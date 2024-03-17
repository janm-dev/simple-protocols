# Builder image
FROM rust:latest AS builder

# Install musl libc for static linking
RUN rustup target add x86_64-unknown-linux-musl
RUN apt update && apt upgrade -y && apt install -y musl-tools musl-dev

WORKDIR /sp

COPY ./ .

# Build with statically-linked musl libc
RUN cargo build --target x86_64-unknown-linux-musl --release

# Final image
FROM scratch

WORKDIR /sp

# Copy the build
COPY --from=builder /sp/target/x86_64-unknown-linux-musl/release/simple-protocols ./

# Configure logging
ENV SIMPLE_PROTOCOLS_LOG="info"

# Expose ports
EXPOSE 1/tcp
EXPOSE 7/tcp
EXPOSE 7/udp
EXPOSE 9/tcp
EXPOSE 9/udp
EXPOSE 11/tcp
EXPOSE 11/udp
EXPOSE 13/tcp
EXPOSE 13/udp
EXPOSE 17/tcp
EXPOSE 17/udp
EXPOSE 18/tcp
EXPOSE 18/udp
EXPOSE 19/tcp
EXPOSE 19/udp
EXPOSE 20/tcp
EXPOSE 21/tcp
EXPOSE 23/tcp
EXPOSE 37/tcp
EXPOSE 37/udp
EXPOSE 53/udp
EXPOSE 69/udp
EXPOSE 70/tcp
EXPOSE 79/tcp
EXPOSE 80/tcp
EXPOSE 101/tcp
EXPOSE 105/tcp
EXPOSE 113/tcp
EXPOSE 115/tcp
EXPOSE 119/tcp
EXPOSE 123/udp
EXPOSE 194/tcp
EXPOSE 218/tcp
EXPOSE 389/tcp
EXPOSE 427/udp
EXPOSE 444/tcp

ENTRYPOINT [ "/sp/simple-protocols" ]
