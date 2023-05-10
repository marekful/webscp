###############################################################
FROM rust:1.69-alpine@sha256:a1123af3383f0e71da8deca8afc1e506ff3ee48c7903f253a11b7b632d938190 AS build
ENV PTHREAD_STACK_MIN 8388608
ARG TARGETPLATFORM

#RUN rustup target add x86_64-unknown-linux-gnu
RUN if [ "${TARGETPLATFORM}" = "linux/amd64" ] || [ "${TARGETPLATFORM}" = "linux/amd64/v2" ] || [ "${TARGETPLATFORM}" = "linux/amd64/v3" ] || [ -z "${TARGETPLATFORM}" ]; then \
      # rustup target add x86_64-unknown-linux-gnu; \
      echo -n ""; \
    elif [ "${TARGETPLATFORM}" = "linux/arm64" ] || [ "${TARGETPLATFORM}" = "linux/arm64/v8" ]; then \
      rustup target add aarch64-unknown-linux-musl; \
    fi

##
COPY Cargo.toml /app/Cargo.toml
COPY Cargo.lock /app/Cargo.lock

WORKDIR /app

RUN nice -n 12 cargo fetch

##
COPY src /app/src

RUN apk add openssl-dev musl-dev

# RUN cargo build --release
#
#RUN RUSTFLAGS='-C target-feature=-crt-static' cargo build --release
RUN if [ "${TARGETPLATFORM}" = "linux/amd64" ] || [ "${TARGETPLATFORM}" = "linux/amd64/v2" ] || [ "${TARGETPLATFORM}" = "linux/amd64/v3" ] || [ -z "${TARGETPLATFORM}" ]; then \
      RUSTFLAGS='-C target-feature=-crt-static' nice -n 10 cargo build --release; \
    elif [ "${TARGETPLATFORM}" = "linux/arm64" ] || [ "${TARGETPLATFORM}" = "linux/arm64/v8" ]; then \
      RUSTFLAGS='-C target-feature=-crt-static' nice -n 10 cargo build --target aarch64-unknown-linux-musl --release; \
    fi

###############################################################
FROM alpine:3.17.3@sha256:124c7d2707904eea7431fffe91522a01e5a861a624ee31d03372cc1d138a3126 AS release

ARG TARGETPLATFORM

ENV DISTRO=alpine
ENV S6_OVERLAY_VERSION=3.1.4.1
ENV ROCKET_CONFIG=/app/Rocket.toml

ADD https://github.com/just-containers/s6-overlay/releases/download/v${S6_OVERLAY_VERSION}/s6-overlay-noarch.tar.xz /tmp/
ADD https://github.com/just-containers/s6-overlay/releases/download/v${S6_OVERLAY_VERSION}/s6-overlay-x86_64.tar.xz /tmp/
ADD https://github.com/just-containers/s6-overlay/releases/download/v${S6_OVERLAY_VERSION}/s6-overlay-aarch64.tar.xz /tmp/
ADD https://github.com/just-containers/s6-overlay/releases/download/v${S6_OVERLAY_VERSION}/s6-overlay-symlinks-noarch.tar.xz /tmp/
ADD https://github.com/just-containers/s6-overlay/releases/download/v${S6_OVERLAY_VERSION}/s6-overlay-symlinks-arch.tar.xz /tmp/

RUN apk update && \
    apk add openssh openssh-sftp-server openssl figlet bash libgcc gcompat rsync && \
    tar -C / -Jxpf /tmp/s6-overlay-noarch.tar.xz && \
    if [ "${TARGETPLATFORM}" = "linux/amd64" ] || [ "${TARGETPLATFORM}" = "linux/amd64/v2" ] || [ "${TARGETPLATFORM}" = "linux/amd64/v3" ] || [ -z "${TARGETPLATFORM}" ]; then \
      tar -C / -Jxpf /tmp/s6-overlay-x86_64.tar.xz; \
    elif [ "${TARGETPLATFORM}" = "linux/arm64" ] || [ "${TARGETPLATFORM}" = "linux/arm64/v8" ]; then \
      tar -C / -Jxpf /tmp/s6-overlay-aarch64.tar.xz; \
    fi && \
    tar -C / -Jxpf /tmp/s6-overlay-symlinks-noarch.tar.xz && \
    tar -C / -Jxpf /tmp/s6-overlay-symlinks-arch.tar.xz && \
    rm -f /tmp/s6-overlay-*.tar.xz && \
    ln -s /usr/lib/ssh /usr/lib/openssh

##
WORKDIR /app

COPY --from=build /app/target/*release/webserver .
COPY --from=build /app/target/*release/cli .
COPY --from=build /app/target/*aarch64-unknown-linux-musl/release/webserver .
COPY --from=build /app/target/*aarch64-unknown-linux-musl/release/cli .

COPY build/config/Rocket.toml /app/Rocket.toml

COPY build/s6 /

VOLUME ["/app/data"]

EXPOSE 80 22

ENTRYPOINT ["/init"]
