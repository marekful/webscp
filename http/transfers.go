package http

import (
	"net/http"

	"github.com/marekful/webscp/agents"

	"github.com/gorilla/mux"
)

var transferDeleteHandler = injectAgentWithUser(func(w http.ResponseWriter, r *http.Request, d *data) (int, error) {
	vars := mux.Vars(r)
	transferID := vars["transfer_id"]

	client := agents.AgentClient{
		Agent: d.agent,
	}

	authCookie, _ := r.Cookie("auth")

	status, err := client.CancelTransfer(transferID, authCookie.Value)
	if err != nil {
		return status, err
	}

	return http.StatusOK, nil
})
