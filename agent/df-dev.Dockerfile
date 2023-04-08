FROM rust:1.68

RUN apt update && apt install -y libssl-dev openssh-server figlet

##

ENV S6_OVERLAY_VERSION=3.1.4.1
ENV DEVELOPMENT=1

ADD https://github.com/just-containers/s6-overlay/releases/download/v${S6_OVERLAY_VERSION}/s6-overlay-noarch.tar.xz /tmp
ADD https://github.com/just-containers/s6-overlay/releases/download/v${S6_OVERLAY_VERSION}/s6-overlay-x86_64.tar.xz /tmp
ADD https://github.com/just-containers/s6-overlay/releases/download/v${S6_OVERLAY_VERSION}/s6-overlay-symlinks-noarch.tar.xz /tmp
ADD https://github.com/just-containers/s6-overlay/releases/download/v${S6_OVERLAY_VERSION}/s6-overlay-symlinks-arch.tar.xz /tmp
RUN tar -C / -Jxpf /tmp/s6-overlay-noarch.tar.xz && \
    tar -C / -Jxpf /tmp/s6-overlay-x86_64.tar.xz && \
    tar -C / -Jxpf /tmp/s6-overlay-symlinks-noarch.tar.xz && \
    tar -C / -Jxpf /tmp/s6-overlay-symlinks-arch.tar.xz

##

#VOLUME ["/app/target", "/usr/local/cargo"]

COPY Cargo.toml /app/Cargo.toml
COPY Cargo.lock /app/Cargo.lock

WORKDIR /app

RUN cargo fetch

##

COPY src /app/src
COPY config/Rocket.toml /app/Rocket.toml

RUN cargo build
RUN ln -s target/debug/cli cli
RUN ln -s target/debug/webserver webserver
#RUN --mount=type=cache,target=/root/.cargo ["cargo", "build"]

##

COPY build/s6 /

EXPOSE 80 22

ENTRYPOINT ["/init"]
#ENTRYPOINT ["/command/s6-svscan", "/etc/services.d/"]
