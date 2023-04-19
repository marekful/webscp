###############################################################
FROM rust:1.68-alpine AS builder
ENV PTHREAD_STACK_MIN 8388608

##
COPY Cargo.toml /app/Cargo.toml
COPY Cargo.lock /app/Cargo.lock

WORKDIR /app

RUN cargo fetch

##
COPY src /app/src
COPY config/Rocket.toml /app/Rocket.toml

RUN apk add openssl-dev musl-dev

# RUN cargo build --release
#
RUN RUSTFLAGS='-C target-feature=-crt-static' cargo build --release

###############################################################
FROM alpine:latest

ENV S6_OVERLAY_VERSION=3.1.4.1

ADD https://github.com/just-containers/s6-overlay/releases/download/v${S6_OVERLAY_VERSION}/s6-overlay-noarch.tar.xz /tmp/
ADD https://github.com/just-containers/s6-overlay/releases/download/v${S6_OVERLAY_VERSION}/s6-overlay-x86_64.tar.xz /tmp/
ADD https://github.com/just-containers/s6-overlay/releases/download/v${S6_OVERLAY_VERSION}/s6-overlay-symlinks-noarch.tar.xz /tmp/
ADD https://github.com/just-containers/s6-overlay/releases/download/v${S6_OVERLAY_VERSION}/s6-overlay-symlinks-arch.tar.xz /tmp/

RUN apk update && \
    apk add openssh openssh-server-pam openssl figlet bash libgcc gcompat && \
    tar -C / -Jxpf /tmp/s6-overlay-noarch.tar.xz && \
    tar -C / -Jxpf /tmp/s6-overlay-x86_64.tar.xz && \
    tar -C / -Jxpf /tmp/s6-overlay-symlinks-noarch.tar.xz && \
    tar -C / -Jxpf /tmp/s6-overlay-symlinks-arch.tar.xz && \
    rm -f /tmp/s6-verlay-*.tar.xz

##
WORKDIR /app

COPY --from=builder /app/target/release/webserver .
COPY --from=builder /app/target/release/cli .
#
# COPY --from=builder /app/target/debug/webserver .
# COPY --from=builder /app/target/debug/cli .

COPY build/s6 /

EXPOSE 80 22

ENTRYPOINT ["/init"]
