################## Frontend build ##################
FROM docker.io/node:18 as vue-builder

WORKDIR /work

COPY ./frontend/package.json .
COPY ./frontend/package-lock.json .

ENV NODE_ENV=development
ENV NODE_OPTIONS=--openssl-legacy-provider

RUN npm install --include=dev

COPY  ./frontend /work/

RUN npm run build

################## Backend build ##################
FROM docker.io/golang:1.20.1-alpine AS go-builder

RUN apk add bash make git ncurses yarn npm

WORKDIR /work

COPY ./backend/go.mod .
COPY ./backend/go.sum .

RUN go mod download

COPY ./backend/ /work/
COPY --from=vue-builder /work/dist/ /work/frontend/dist/

#RUN make lint-backend
#RUN make lint-frontend
#RUN make lint-commits
#RUN make test-backend
#RUN make test-frontend

RUN make build-backend-dev

################## Run ##################
FROM docker.io/golang:1.20.1-alpine
RUN apk --update add ca-certificates \
                     mailcap \
                     curl \
                     libcap \
                     bash \
                     npm \
                     uuidgen

RUN adduser -D -H -s /bin/ash webscp

HEALTHCHECK --start-period=2s --interval=5s --timeout=3s \
  CMD curl -f http://localhost/health || exit 1

VOLUME /srv
EXPOSE 80 8080 44000

WORKDIR /app

COPY --from=go-builder /work/webscp .
COPY --from=go-builder /work/frontend frontend
COPY docker_config.json filebrowser.json

##RUN cd /app/frontend && npm ci

RUN go install github.com/go-delve/delve/cmd/dlv@latest

ENV NODE_ENV=development
ENV NODE_OPTIONS=--openssl-legacy-provider

#ENTRYPOINT chown filebrowser:filebrowser /database.db && capsh --caps="cap_net_raw+eip cap_setpcap,cap_setuid,cap_setgid+ep" --keep=1 --user=filebrowser --addamb=cap_net_raw -- -c "/app/filebrowser"

RUN echo "cd /app/frontend && npm run watch" > fewatch.sh && \
    chown webscp fewatch.sh && chmod u+x fewatch.sh

RUN chown webscp frontend/dist && rm -rf frontend/dist/*

USER webscp

ENTRYPOINT dlv --listen=:44000 --headless=true --api-version=2 --accept-multiclient exec /app/webscp
