###############################################################
FROM rust:latest AS builder

RUN rustup target add x86_64-unknown-linux-gnu
RUN apt update && apt install -y libssl-dev
RUN update-ca-certificates

##
COPY Cargo.toml /app/Cargo.toml
COPY Cargo.lock /app/Cargo.lock

WORKDIR /app

RUN cargo fetch

##
COPY src /app/src
COPY config/Rocket.toml /app/Rocket.toml

RUN cargo build --release

###############################################################
FROM debian:bullseye-slim

ENV S6_OVERLAY_VERSION=3.1.4.1

ADD https://github.com/just-containers/s6-overlay/releases/download/v${S6_OVERLAY_VERSION}/s6-overlay-noarch.tar.xz /tmp/
ADD https://github.com/just-containers/s6-overlay/releases/download/v${S6_OVERLAY_VERSION}/s6-overlay-x86_64.tar.xz /tmp/
ADD https://github.com/just-containers/s6-overlay/releases/download/v${S6_OVERLAY_VERSION}/s6-overlay-symlinks-noarch.tar.xz /tmp/
ADD https://github.com/just-containers/s6-overlay/releases/download/v${S6_OVERLAY_VERSION}/s6-overlay-symlinks-arch.tar.xz /tmp/

RUN apt-get update && \
    apt-get install -y ssh openssh-server openssl figlet bash xz-utils && \
    apt-get clean && \
    apt-get autoremove -y && \
    tar -C / -Jxpf /tmp/s6-overlay-noarch.tar.xz && \
    tar -C / -Jxpf /tmp/s6-overlay-x86_64.tar.xz && \
    tar -C / -Jxpf /tmp/s6-overlay-symlinks-noarch.tar.xz && \
    tar -C / -Jxpf /tmp/s6-overlay-symlinks-arch.tar.xz && \
    rm -f /tmp/s6-verlay-*.tar.xz && \
    apt-get purge -y xz-utils

##
WORKDIR /app

COPY --from=builder /app/target/release/webserver .
COPY --from=builder /app/target/release/cli .

COPY build/s6 /

EXPOSE 80 22

ENTRYPOINT ["/init"]
