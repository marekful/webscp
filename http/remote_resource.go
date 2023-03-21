package http

import (
	"github.com/filebrowser/filebrowser/v2/agents"
	"github.com/gorilla/mux"
	"net/http"
	"strconv"
)

var remoteResourceGetHandler = withUser(func(w http.ResponseWriter, r *http.Request, d *data) (int, error) {

	vars := mux.Vars(r)
	id64, err := strconv.ParseUint(vars["agent_id"], 10, 64)
	if err != nil {
		return http.StatusBadRequest, err
	}

	agent, err := d.store.Agents.Get(uint(id64))
	if err != nil {
		return http.StatusBadRequest, err
	}

	client := agents.AgentClient{
		Agent: agent,
	}

	resp, err := client.GetResource(agent.Host, agent.Port, vars["url"])
	if err != nil {
		return http.StatusBadRequest, err
	}

	return renderJSON(w, r, resp)
})
