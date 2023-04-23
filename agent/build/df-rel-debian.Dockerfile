###############################################################
FROM rust:latest AS build

ARG TARGETPLATFORM

#RUN rustup target add x86_64-unknown-linux-gnu
RUN if [ "${TARGETPLATFORM}" = "linux/amd64" ] ||[ "${TARGETPLATFORM}" = "linux/amd64/v2" ] ||[ "${TARGETPLATFORM}" = "linux/amd64/v3" ] || [ -z "${TARGETPLATFORM}" ]; then \
      rustup target add x86_64-unknown-linux-gnu; \
    elif [ "${TARGETPLATFORM}" = "linux/arm64" ] || [ "${TARGETPLATFORM}" = "linux/arm64/v8" ]; then \
      rustup target add aarch64-unknown-linux-gnu; \
    fi

RUN apt-get update && apt-get install -y libssl-dev
RUN update-ca-certificates

##
COPY Cargo.toml /app/Cargo.toml
COPY Cargo.lock /app/Cargo.lock

WORKDIR /app

RUN cargo fetch

##
COPY src /app/src

#RUN cargo build --release
RUN if [ "${TARGETPLATFORM}" = "linux/amd64" ] || [ "${TARGETPLATFORM}" = "linux/amd64/v2" ] || [ "${TARGETPLATFORM}" = "linux/amd64/v3" ] || [ -z "${TARGETPLATFORM}" ]; then \
      cargo build --release; \
    elif [ "${TARGETPLATFORM}" = "linux/arm64" ] || [ "${TARGETPLATFORM}" = "linux/arm64/v8" ]; then \
      cargo build --target aarch64-unknown-linux-gnu --release; \
    fi

###############################################################
FROM debian:bullseye-slim AS release

ARG TARGETPLATFORM

ENV DISTRO=debian
ENV S6_OVERLAY_VERSION=3.1.4.1
ENV ROCKET_CONFIG=/app/Rocket.toml

ADD https://github.com/just-containers/s6-overlay/releases/download/v${S6_OVERLAY_VERSION}/s6-overlay-noarch.tar.xz /tmp/
ADD https://github.com/just-containers/s6-overlay/releases/download/v${S6_OVERLAY_VERSION}/s6-overlay-x86_64.tar.xz /tmp/
ADD https://github.com/just-containers/s6-overlay/releases/download/v${S6_OVERLAY_VERSION}/s6-overlay-aarch64.tar.xz /tmp/
ADD https://github.com/just-containers/s6-overlay/releases/download/v${S6_OVERLAY_VERSION}/s6-overlay-symlinks-noarch.tar.xz /tmp/
ADD https://github.com/just-containers/s6-overlay/releases/download/v${S6_OVERLAY_VERSION}/s6-overlay-symlinks-arch.tar.xz /tmp/

RUN apt-get update && \
    apt-get install -y ssh openssh-server openssl figlet bash xz-utils && \
    apt-get clean && \
    apt-get autoremove -y && \
    tar -C / -Jxpf /tmp/s6-overlay-noarch.tar.xz && \
    if [ "${TARGETPLATFORM}" = "linux/amd64" ] || [ "${TARGETPLATFORM}" = "linux/amd64/v2" ] || [ "${TARGETPLATFORM}" = "linux/amd64/v3" ] || [ -z "${TARGETPLATFORM}" ]; then \
      tar -C / -Jxpf /tmp/s6-overlay-x86_64.tar.xz; \
    elif [ "${TARGETPLATFORM}" = "linux/arm64" ] || [ "${TARGETPLATFORM}" = "linux/arm64/v8" ]; then \
      tar -C / -Jxpf /tmp/s6-overlay-aarch64.tar.xz; \
    fi && \
    tar -C / -Jxpf /tmp/s6-overlay-symlinks-noarch.tar.xz && \
    tar -C / -Jxpf /tmp/s6-overlay-symlinks-arch.tar.xz && \
    rm -f /tmp/s6-overlay-*.tar.xz && \
    apt-get purge -y xz-utils

##
WORKDIR /app

COPY --from=build /app/target/*release/webserver .
COPY --from=build /app/target/*release/cli .
COPY --from=build /app/target/*aarch64-unknown-linux-gnu/release/webserver .
COPY --from=build /app/target/*aarch64-unknown-linux-gnu/release/cli .

COPY build/config/Rocket.toml /app/Rocket.toml

COPY build/s6 /

VOLUME ["/app/data"]

EXPOSE 80 22

ENTRYPOINT ["/init"]
