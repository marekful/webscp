# Install WebSCP on a Linux server and run it with Docker

## Prepare for first run
- Create an empty directory and copy these 3 files
- In `compose.yaml`, under `services/files`, edit the entry under `ports`
  - A ports entry should be defined as `[host-port]:[container-port]`
  - Leave the container-port as `80`
  - Set the host-port to the port you want to expose the Web UI on the host
  - E.g. if `ports` is `"7080:80"` then the Web UI should be accessible on `http://localhost:7080`
- Under `services/files`, edit `UUID` under `environment`
  - Set it to the `uid` of the linux user in the host systen who should run WebSCP
  - WebSCP will have the same permissions and access rights within the _file system_ as the specified user
- Optionally, under `services/files` edit the entry that reads `"/:/srv"` under `volumes`
  - This specifies the directory on the host that WebSCP will consider its file system root
  - You can set the path before the colon to any directory you want
  - Make sure the container-path, after the colon, remains `/srv`
- Under `services/agent`, edit the entry under `ports`
  - Leave the container-port as `22`
  - Set the host-port to the port WebSCP should listen on for remote connections
  - When creating a new connection from another WebSCP, this host-port should be used as `Agent Port`
- IMPORTANT: Set the same `UUID` under `servcies/agent/environment` as you set for `files`
- IMPORTANT: If you changed the `volumes` entry for `files`, set the same for `agent` 
- Save and close `compose.yaml`
- Make sure the empty file named `database.db` and the file named `settings.json` are also in this directory 

## Run
- In the directory where `compose.yaml` is located, execute `docker compose up -d`
- You may inspect logs to confirm normal startup with `docker compose logs --follow`
- The default user `admin` with password `admin` will be created when WebSCP runs for the first time


