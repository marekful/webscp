
## Get resources (directory listing)


**local**
```mermaid
sequenceDiagram
    Browser->>+Files-API: HTTP GET<br>/api/resources/[path]
    Files-API->>+Files-API: [resource]
    Files-API->>+Browser: [json]
```

**remote**
```mermaid
sequenceDiagram
    Browser->>+Files-API: HTTP GET<br>/api/remote/resources/[agent]/[path]
    Files-API->>+Agent-API: HTTP GET<br>/api/resources/[host]/[port]/[path]
    Agent-API->>+Agent-CLI: SSH cmd exec<br>get-remote-resource [host] [port] [path]
    Agent-CLI->>+REMOTE Agent-CLI: LOCAL cmd exec<br>get-local-resource [path]
    REMOTE Agent-CLI->>REMOTE Files-API: HTTP GET<br>/api/agent/resources/[path]
    REMOTE Files-API->>+REMOTE Files-API: [resource]
    REMOTE Files-API->>+REMOTE Agent-CLI: [json]
    REMOTE Agent-CLI->>+Agent-CLI: [json]
    Agent-CLI->>+Agent-API: [json]
    Agent-API->>+Files-API: [json]
    Files-API->>+Browser: [json]
```
