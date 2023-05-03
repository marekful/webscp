################## Frontend build ##################
FROM docker.io/node:18@sha256:d9946ebbeb3ca08ccaa24a8220d7da1f9e9fd749d489913faeed89e02f70a202 AS frontend-buid

WORKDIR /work

COPY ./frontend/package.json .
COPY ./frontend/package-lock.json .

ENV NODE_OPTIONS=--openssl-legacy-provider

RUN npm install

COPY  ./frontend /work/

RUN npm run build

################## Backend build ##################
FROM docker.io/golang:1.20.3-alpine@sha256:48c87cd759e3342fcbc4241533337141e7d8457ec33ab9660abe0a4346c30b60 AS backend-build

RUN apk add bash make git ncurses yarn npm

WORKDIR /work

COPY ./backend/go.mod .
COPY ./backend/go.sum .

RUN go mod download

COPY ./backend/ /work/
COPY --from=frontend-buid /work/dist/ /work/frontend/dist/

RUN cod backend && make build-backend

################## Run ##################
FROM alpine:3.17@sha256:b6ca290b6b4cdcca5b3db3ffa338ee0285c11744b4a6abaa9627746ee3291d8d AS release
RUN apk --update add ca-certificates \
                     mailcap \
                     curl \
                     libcap \
                     bash \
                     uuidgen \
                     figlet

#RUN adduser -D -H -s /bin/ash webscp

HEALTHCHECK --start-period=2s --interval=5s --timeout=3s \
  CMD curl -f http://localhost/health || exit 1

VOLUME /srv
EXPOSE 80

WORKDIR /app

COPY --from=backend-build /work/backend/webscp .
COPY backend/docker_config.json /settings.json

ENV NODE_OPTIONS=--openssl-legacy-provider

ENTRYPOINT chown webscp:webscp /database.db && capsh --caps="cap_net_raw+eip cap_setpcap,cap_setuid,cap_setgid+ep" --keep=1 --user=webscp --addamb=cap_net_raw -- -c "/app/webscp"

