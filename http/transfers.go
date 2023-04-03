package http

import (
	"fmt"
	"net/http"
	"strconv"

	"github.com/gorilla/mux"

	"github.com/filebrowser/filebrowser/v2/agents"
)

var transferDeleteHandler = withUser(func(w http.ResponseWriter, r *http.Request, d *data) (int, error) {
	vars := mux.Vars(r)
	transferID := vars["transfer_id"]
	agentID := vars["agent_id"]

	id64, err := strconv.ParseUint(agentID, 10, 64)
	if err != nil {
		return http.StatusNotFound, err
	}

	agent, err := d.store.Agents.Get(uint(id64))
	if err != nil {
		return http.StatusNotFound, err
	}

	client := agents.AgentClient{
		Agent: agent,
	}

	resp, status, err := client.CancelTransfer(transferID)
	if err != nil {
		return status, err
	}

	if resp.Code != 0 {
		return http.StatusInternalServerError, fmt.Errorf("unexpected error %s", resp.Message)
	}

	return http.StatusOK, nil
})
