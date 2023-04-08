################## Frontend build ##################
FROM docker.io/node:18 as vue-builder

WORKDIR /work

COPY ./frontend/package.json .
COPY ./frontend/package-lock.json .

ENV NODE_OPTIONS=--openssl-legacy-provider

RUN npm install

COPY  ./frontend /work/

RUN npm run build

################## Backend build ##################
FROM docker.io/golang:1.20.1-alpine AS go-builder

RUN apk add bash make git ncurses yarn npm

WORKDIR /work

COPY ./go.mod .
COPY ./go.sum .

RUN go mod download

COPY . /work/
COPY --from=vue-builder /work/dist/ /work/frontend/dist/

RUN make build-backend

################## Run ##################
FROM alpine:latest
RUN apk --update add ca-certificates \
                     mailcap \
                     curl \
                     libcap \
                     bash \
                     uuidgen

RUN adduser -D -H -s /bin/ash filebrowser

HEALTHCHECK --start-period=2s --interval=5s --timeout=3s \
  CMD curl -f http://localhost/health || exit 1

VOLUME /srv
EXPOSE 80 8080 44000 45000

WORKDIR /app

COPY --from=go-builder /work/filebrowser .
COPY docker_config.json /.filebrowser.json

ENV NODE_OPTIONS=--openssl-legacy-provider

ENTRYPOINT chown filebrowser:filebrowser /database.db && capsh --caps="cap_net_raw+eip cap_setpcap,cap_setuid,cap_setgid+ep" --keep=1 --user=filebrowser --addamb=cap_net_raw -- -c "/app/filebrowser"

