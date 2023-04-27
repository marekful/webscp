################## Frontend build ##################
FROM docker.io/node:18 AS frontend-buid

WORKDIR /work

COPY ./frontend/package.json .
COPY ./frontend/package-lock.json .

ENV NODE_OPTIONS=--openssl-legacy-provider

RUN npm install

COPY  ./frontend /work/

RUN npm run build

################## Backend build ##################
FROM docker.io/golang:1.20.1-alpine AS backend-build

RUN apk add bash make git ncurses yarn npm

WORKDIR /work

COPY ./go.mod .
COPY ./go.sum .

RUN go mod download

COPY . /work/
COPY --from=frontend-buid /work/dist/ /work/frontend/dist/

RUN make build-backend

################## Run ##################
FROM alpine:latest AS release

ARG TARGETPLATFORM

ENV S6_OVERLAY_VERSION=3.1.4.1
ENV NODE_OPTIONS=--openssl-legacy-provider

ADD https://github.com/just-containers/s6-overlay/releases/download/v${S6_OVERLAY_VERSION}/s6-overlay-noarch.tar.xz /tmp/
ADD https://github.com/just-containers/s6-overlay/releases/download/v${S6_OVERLAY_VERSION}/s6-overlay-x86_64.tar.xz /tmp/
ADD https://github.com/just-containers/s6-overlay/releases/download/v${S6_OVERLAY_VERSION}/s6-overlay-aarch64.tar.xz /tmp/
ADD https://github.com/just-containers/s6-overlay/releases/download/v${S6_OVERLAY_VERSION}/s6-overlay-symlinks-noarch.tar.xz /tmp/
ADD https://github.com/just-containers/s6-overlay/releases/download/v${S6_OVERLAY_VERSION}/s6-overlay-symlinks-arch.tar.xz /tmp/

RUN apk --update add ca-certificates \
        mailcap \
        curl \
        libcap \
        bash \
        uuidgen \
        figlet \
        xz && \
    tar -C / -Jxpf /tmp/s6-overlay-noarch.tar.xz && \
    if [ "${TARGETPLATFORM}" = "linux/amd64" ] || [ "${TARGETPLATFORM}" = "linux/amd64/v2" ] || [ "${TARGETPLATFORM}" = "linux/amd64/v3" ] || [ -z "${TARGETPLATFORM}" ]; then \
      tar -C / -Jxpf /tmp/s6-overlay-x86_64.tar.xz; echo "p0:${TARGETPLATFORM}" > /tmp/s6; \
    elif [ "${TARGETPLATFORM}" = "linux/arm64" ] || [ "${TARGETPLATFORM}" = "linux/arm64/v8" ]; then \
      tar -C / -Jxpf /tmp/s6-overlay-aarch64.tar.xz; echo "p0:${TARGETPLATFORM}" > /tmp/s6; \
    fi && \
    tar -C / -Jxpf /tmp/s6-overlay-symlinks-noarch.tar.xz && \
    tar -C / -Jxpf /tmp/s6-overlay-symlinks-arch.tar.xz && \
    rm -f /tmp/s6-overlay-*.tar.xz

HEALTHCHECK --start-period=2s --interval=5s --timeout=3s \
  CMD curl -f http://localhost/health || exit 1

WORKDIR /app

COPY docker/root /

COPY --from=backend-build /work/webscp .
COPY docker_config.json /settings.json

VOLUME /srv
EXPOSE 80

ENTRYPOINT ["/init"]
