# API routes

```http request
GET    /users/<user-id>/temporary-access-token

POST   /users/<user-id>/connections/<host>/<port>/login (get-remote-user)
POST   /users/<user-id>/connections (get-token-user, exchange-keys)

GET    /agents/<agent-id>/resources/<path>
PATCH  /agents/<agent-id>/resources/[[<archive-name>]]

DELETE /agents/<agent-id>/transfers/<transfer-id>

GET    /agents/<agent-id>/version
GET    /agents/<agent-id>/ping
```
