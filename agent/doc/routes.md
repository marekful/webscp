Now:

```text
GET    /temporary-access-token/<user_id>
POST   /get-remote-user/<host>/<port>
POST   /register-public-key
GET    /resources/<agent_id>/<path>
POST   /copy/<agent_id>/<archive_name>
DELETE /transfers/<transfer_id>
GET    /version/<host>/<port>
GET    /ping/<host>/<port>
```

Soon:

```http request
GET    /users/<user_id>/temporary-access-token

POST   /connections/<host>/<port>/login (get-remote-user)
POST   /connections (register-public-key)

GET    /agents/<agent_id>/resources/<path>
PATCH  /agents/<agent_id>/resources/[[<archive_name>]]

DELETE /agents/<agent_id>/transfers/<transfer_id>

GET    /agents/<agent_id>/version
GET    /agents/<agent_id>/ping
```

