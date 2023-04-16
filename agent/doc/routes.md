# API routes

```http request
GET    /users/<user_id>/temporary-access-token

POST   /users/<user_id>/connections/<host>/<port>/login (get-remote-user)
POST   /users/<user_id>/connections (register-public-key)

GET    /agents/<agent_id>/resources/<path>
PATCH  /agents/<agent_id>/resources/[[<archive_name>]]

DELETE /agents/<agent_id>/transfers/<transfer_id>

GET    /agents/<agent_id>/version
GET    /agents/<agent_id>/ping
```
