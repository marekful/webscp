package http

import (
	"net/http"

	"github.com/filebrowser/filebrowser/v2/agents"

	"github.com/gorilla/mux"
)

var transferDeleteHandler = injectAgentWithUser(func(w http.ResponseWriter, r *http.Request, d *data) (int, error) {
	vars := mux.Vars(r)
	transferID := vars["transfer_id"]

	client := agents.AgentClient{
		Agent: d.agent,
	}

	status, err := client.CancelTransfer(transferID)
	if err != nil {
		return status, err
	}

	return http.StatusOK, nil
})
