################## Frontend build ##################
FROM docker.io/node:18@sha256:196c5cfdd2f7f41a42c418b4fc6c553f9ab3575cfdc228199de8a465be788155 AS frontend-buid

WORKDIR /work

COPY ./frontend/package.json .
COPY ./frontend/package-lock.json .

ENV NODE_OPTIONS=--openssl-legacy-provider

RUN npm install

COPY  ./frontend /work/

RUN npm run build

################## Backend build ##################
FROM docker.io/golang:1.20.3-alpine@sha256:08e9c086194875334d606765bd60aa064abd3c215abfbcf5737619110d48d114 AS backend-build

RUN apk add bash make git ncurses yarn npm

WORKDIR /work

COPY ./backend/go.mod .
COPY ./backend/go.sum .

RUN go mod download

COPY . /work/
COPY --from=frontend-buid /work/dist/ /work/backend/frontend/dist/

RUN cd backend && make build-backend

################## Run ##################
FROM alpine:3.17@sha256:124c7d2707904eea7431fffe91522a01e5a861a624ee31d03372cc1d138a3126 AS release

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

COPY backend/docker/root /

COPY --from=backend-build /work/backend/webscp .
COPY backend/docker_config.json /settings.json

VOLUME /srv
EXPOSE 80

ENTRYPOINT ["/init"]
