package http

import (
	"encoding/json"
	"net/http"
	"sort"
	"strconv"

	"github.com/filebrowser/filebrowser/v2/agents"
	"github.com/filebrowser/filebrowser/v2/errors"
)

type modifyAgentRequest struct {
	modifyRequest
	Data *agents.Agent `json:"data"`
}

/*func getAgentID(r *http.Request) (uint, error) {
	vars := mux.Vars(r)
	i, err := strconv.ParseUint(vars["id"], 10, 0)
	if err != nil {
		return 0, err
	}
	return uint(i), err
}*/

func getAgent(_ http.ResponseWriter, r *http.Request) (*modifyAgentRequest, error) {
	if r.Body == nil {
		return nil, errors.ErrEmptyRequest
	}

	req := &modifyAgentRequest{}
	err := json.NewDecoder(r.Body).Decode(req)
	if err != nil {
		return nil, err
	}

	if req.What != "agent" {
		return nil, errors.ErrInvalidDataType
	}

	return req, nil
}

var agentsGetHandler = withAdmin(func(w http.ResponseWriter, r *http.Request, d *data) (int, error) {
	agents, err := d.store.Agents.Gets()
	if err != nil {
		return http.StatusInternalServerError, err
	}

	sort.Slice(agents, func(i, j int) bool {
		return agents[i].ID < agents[j].ID
	})

	return renderJSON(w, r, agents)
})

var agentGetHandler = withSelfOrAdmin(func(w http.ResponseWriter, r *http.Request, d *data) (int, error) {
	a, err := d.store.Agents.Get(d.raw.(uint))
	if err == errors.ErrNotExist {
		return http.StatusNotFound, err
	}

	if err != nil {
		return http.StatusInternalServerError, err
	}

	return renderJSON(w, r, a)
})

var agentDeleteHandler = withSelfOrAdmin(func(w http.ResponseWriter, r *http.Request, d *data) (int, error) {
	err := d.store.Agents.Delete(d.raw.(uint))
	if err != nil {
		return errToStatus(err), err
	}

	return http.StatusOK, nil
})

var agentPostHandler = withAdmin(func(w http.ResponseWriter, r *http.Request, d *data) (int, error) {
	req, err := getAgent(w, r)
	if err != nil {
		return http.StatusBadRequest, err
	}

	if len(req.Which) != 0 {
		return http.StatusBadRequest, nil
	}

	if req.Data.Secret == "" {
		return http.StatusBadRequest, errors.ErrEmptyAgentSecret
	}

	client := agents.AgentClient{
		Agent: req.Data,
	}
	err = client.ExchangeKeys(req.Data.Host, req.Data.Port, req.Data.Secret)
	if err != nil {
		return http.StatusBadRequest, err
	}

	err = d.store.Agents.Save(req.Data)
	if err != nil {
		return http.StatusInternalServerError, err
	}

	w.Header().Set("Location", "/settings/agents/"+strconv.FormatUint(uint64(req.Data.ID), 10))
	return http.StatusCreated, nil
})

var agentGetVersionHandler = withSelfOrAdmin(func(w http.ResponseWriter, r *http.Request, d *data) (int, error) {
	agent, err := d.store.Agents.Get(d.raw.(uint))
	if err == errors.ErrNotExist {
		return http.StatusNotFound, err
	}

	client := agents.AgentClient{Agent: agent}
	version := client.GetVersion()

	versionResponse := agents.GetVersionResponse{
		Latency: version.Latency,
		Version: version.Version,
		Error:   version.Error,
	}

	return renderJSON(w, r, versionResponse)
})
