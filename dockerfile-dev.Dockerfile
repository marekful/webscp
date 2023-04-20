####################################################
################## Frontend build ##################
####################################################
FROM docker.io/node:18 as vue-builder

WORKDIR /work

COPY ./frontend/package.json .
COPY ./frontend/package-lock.json .

ENV NODE_ENV=development
ENV NODE_OPTIONS=--openssl-legacy-provider

RUN npm install --include=dev

COPY  ./frontend /work/

RUN npm run build

####################################################
################## Backend build ###################
####################################################
FROM docker.io/golang:1.20.1-alpine AS go-builder

RUN apk add bash make git ncurses yarn npm

WORKDIR /work

COPY ./go.mod .
COPY ./go.sum .

RUN go mod download

COPY . /work/
COPY --from=vue-builder /work/dist/ /work/frontend/dist/

#RUN make lint-backend
#RUN make lint-frontend
#RUN make lint-commits
#RUN make test-backend
#RUN make test-frontend

RUN make build-backend

####################################################
#################### Dev image #####################
####################################################
FROM docker.io/golang:1.20.1-alpine
RUN apk --update add ca-certificates \
                     mailcap \
                     curl \
                     libcap \
                     bash \
                     npm \
                     uuidgen \
                     figlet

RUN adduser -D -H -s /bin/ash filebrowser

HEALTHCHECK --start-period=2s --interval=5s --timeout=3s \
  CMD curl -f http://localhost/health || exit 1

VOLUME /srv
EXPOSE 80 8080 44000 45000

WORKDIR /app

COPY --from=go-builder /work/filebrowser .
COPY --from=go-builder /work/frontend frontend
COPY docker_config.json /.filebrowser.json

RUN cd /app/frontend && npm ci

#RUN go install github.com/go-delve/delve/cmd/dlv@latest

ENV NODE_ENV=development
ENV NODE_OPTIONS=--openssl-legacy-provider

RUN echo "cd /app/frontend && npm run watch" > fewatch.sh && chmod u+x fewatch.sh

#ENTRYPOINT /app/filebrowser

ENTRYPOINT chown filebrowser:filebrowser /database.db && capsh --caps="cap_net_raw+eip cap_setpcap,cap_setuid,cap_setgid+ep" --keep=1 --user=filebrowser --addamb=cap_net_raw -- -c "/app/filebrowser"

#ENTRYPOINT dlv --listen=:44000 --headless=true --api-version=2 --accept-multiclient exec /filebrowser
