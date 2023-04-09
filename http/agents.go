package http

import (
	"encoding/json"
	"fmt"
	"net/http"
	"net/url"
	"os"
	"sort"
	"strconv"

	"github.com/gorilla/mux"

	"github.com/filebrowser/filebrowser/v2/agents"
	"github.com/filebrowser/filebrowser/v2/errors"
	"github.com/filebrowser/filebrowser/v2/users"
)

type modifyAgentRequest struct {
	modifyRequest
	Data *agents.Agent `json:"data"`
}

type getUserRequest struct {
	Name     string `json:"name"`
	Password string `json:"password"`
}

type getUserResponse struct {
	ID   uint   `json:"id"`
	Root string `json:"root"`
}

func getAgentID(r *http.Request) (uint, error) {
	vars := mux.Vars(r)
	i, err := strconv.ParseUint(vars["agent_id"], 10, 0)
	if err != nil {
		return 0, err
	}
	return uint(i), err
}

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

func injectAgent(r *http.Request, d *data) {
	id64, err := getAgentID(r)
	if err == nil {
		agent, dErr := d.store.Agents.Get(id64)
		if dErr == nil {
			d.agent = agent
		}
	}

	vars := mux.Vars(r)
	if len(vars["url"]) > 0 {
		decodedURL, err := url.QueryUnescape(vars["url"])
		if err == nil {
			r.URL.Path = decodedURL
		}
	}
}

func withAgent(fn handleFunc) handleFunc {
	return func(w http.ResponseWriter, r *http.Request, d *data) (int, error) {
		// Deny access to requests whose 'Host' header doesn't match the internal docker address (e.g. http://filebrowser:80).
		internalHost := os.Getenv("INTERNAL_ADDRESS")
		requestHost := "http://" + r.Host
		if requestHost != internalHost {
			return http.StatusUnauthorized, fmt.Errorf("error: %s does not match %s", internalHost, requestHost)
		}

		injectAgent(r, d)

		return fn(w, r, d)
	}
}

func withAgentAdmin(fn handleFunc) handleFunc {
	return withAgent(func(w http.ResponseWriter, r *http.Request, d *data) (int, error) {
		user, err := d.store.Users.Get(d.server.Root, uint(1))
		if err != nil {
			return http.StatusInternalServerError, nil
		}
		d.user = user

		return fn(w, r, d)
	})
}

func withAgentUser(fn handleFunc) handleFunc {
	return withAgent(func(w http.ResponseWriter, r *http.Request, d *data) (int, error) {
		vars := mux.Vars(r)
		userID := vars["user_id"]
		id64, err := strconv.ParseUint(userID, 10, 64)
		if err != nil {
			return http.StatusUnauthorized, nil
		}

		user, dErr := d.store.Users.Get(d.server.Root, uint(id64))
		if dErr != nil {
			return http.StatusUnauthorized, nil
		}

		d.user = user

		return fn(w, r, d)
	})
}

func injectAgentWithUser(fn handleFunc) handleFunc {
	return withUser(func(w http.ResponseWriter, r *http.Request, d *data) (int, error) {
		injectAgent(r, d)

		if d.agent.UserID != d.user.ID {
			return http.StatusForbidden, nil
		}

		return fn(w, r, d)
	})
}

var agentsGetHandler = withUser(func(w http.ResponseWriter, r *http.Request, d *data) (int, error) {
	var find func() ([]*agents.Agent, error)
	if d.user.Perm.Admin {
		find = func() ([]*agents.Agent, error) {
			return d.store.Agents.Gets()
		}
	} else {
		find = func() ([]*agents.Agent, error) {
			return d.store.Agents.FindByUserID(d.user.ID)
		}
	}

	agents, err := find()
	if err != nil {
		return http.StatusInternalServerError, err
	}

	sort.Slice(agents, func(i, j int) bool {
		return agents[i].ID < agents[j].ID
	})

	return renderJSON(w, r, agents)
})

var agentGetHandler = withUser(func(w http.ResponseWriter, r *http.Request, d *data) (int, error) {
	id, err := getAgentID(r)
	if err != nil {
		return http.StatusInternalServerError, err
	}

	agent, err := d.store.Agents.Get(id)
	if err == errors.ErrNotExist {
		return http.StatusNotFound, err
	}

	if !d.user.Perm.Admin && agent.UserID != d.user.ID {
		return http.StatusForbidden, nil
	}

	if err != nil {
		return http.StatusInternalServerError, err
	}

	return renderJSON(w, r, agent)
})

var agentDeleteHandler = withUser(func(w http.ResponseWriter, r *http.Request, d *data) (int, error) {
	id, err := getAgentID(r)
	if err != nil {
		return http.StatusInternalServerError, err
	}

	agent, err := d.store.Agents.Get(id)
	if err == errors.ErrNotExist {
		return http.StatusNotFound, err
	}

	if !d.user.Perm.Admin && agent.UserID != d.user.ID {
		return http.StatusForbidden, nil
	}

	aErr := d.store.Agents.Delete(id)
	if aErr != nil {
		return errToStatus(err), err
	}

	return http.StatusOK, nil
})

var agentPostHandler = withUser(func(w http.ResponseWriter, r *http.Request, d *data) (int, error) {
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

	httpStatus, err := client.GetRemoteUserID(&req.Data.RemoteUser)
	if err != nil {
		if httpStatus == http.StatusUnauthorized {
			httpStatus = http.StatusForbidden
		}
		return httpStatus, err
	}

	req.Data.UserID = d.user.ID
	req.Data.RemoteUser.Password = ""

	err = d.store.Agents.Save(req.Data)
	if err != nil {
		return http.StatusInternalServerError, err
	}

	w.Header().Set("Location", "/settings/agents/"+strconv.FormatUint(uint64(req.Data.ID), 10))
	return http.StatusCreated, nil
})

var agentVerifyUserCredentialsPostHandler = withAgentAdmin(func(w http.ResponseWriter, r *http.Request, d *data) (int, error) {
	req := &getUserRequest{}
	err := json.NewDecoder(r.Body).Decode(req)
	if err != nil {
		return http.StatusInternalServerError, err
	}

	u, err := d.store.Users.Get(d.server.Root, req.Name)
	if err != nil || !users.CheckPwd(req.Password, u.Password) {
		return http.StatusUnauthorized, os.ErrPermission
	}

	scope := u.Scope
	if scope == "." {
		scope = "/"
	}

	return renderJSON(w, r, getUserResponse{ID: u.ID, Root: d.server.Root + scope})
})

var agentGetVersionHandler = withUser(func(w http.ResponseWriter, r *http.Request, d *data) (int, error) {
	id, err := getAgentID(r)
	if err != nil {
		return http.StatusInternalServerError, err
	}

	agent, err := d.store.Agents.Get(id)
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
