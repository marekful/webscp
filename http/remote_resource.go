package http

import (
	"encoding/json"
	"fmt"
	"net/http"
	"net/url"
	"os/exec"
	"strconv"

	"github.com/gorilla/mux"

	"github.com/filebrowser/filebrowser/v2/agents"
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

	resp, err := client.GetResource(agent.RemoteUser.ID, agent.Host, agent.Port, vars["url"])
	if err != nil {
		return http.StatusBadRequest, err
	}

	return renderJSON(w, r, resp)
})

func remoteSourceResourcePostHandler() handleFunc {
	return withUser(func(w http.ResponseWriter, r *http.Request, d *data) (int, error) {
		var req []agents.ResourceItem
		err := json.NewDecoder(r.Body).Decode(&req)
		if err != nil {
			return http.StatusBadRequest, err
		}

		vars := mux.Vars(r)
		id64, err := strconv.ParseUint(vars["agent_id"], 10, 64)
		if err != nil {
			return http.StatusBadRequest, err
		}
		agentID := uint(id64)

		action := r.URL.Query().Get("action")
		for idx, item := range req {
			src := item.Source
			dst := item.Destination

			dst, dErr := url.QueryUnescape(dst)
			if dErr != nil {
				return errToStatus(err), err
			}
			req[idx].Destination = dst

			if !d.Check(src) {
				return http.StatusForbidden, nil
			}

			if src == "/" {
				return http.StatusForbidden, nil
			}
		}

		//TODO: consider running hooks
		status, response, err := remoteResourcePostAction(agentID, action, req, d)
		if status == http.StatusOK {
			return renderJSON(w, r, response)
		}

		return status, err
	})
}

func remoteDestinationResourcePostHandler() handleFunc {
	return withAgentUser(func(w http.ResponseWriter, r *http.Request, d *data) (int, error) {
		var req []agents.ResourceItem
		err := json.NewDecoder(r.Body).Decode(&req)
		if err != nil {
			return http.StatusBadRequest, err
		}

		for _, item := range req {
			dst := item.Destination
			dst, err := url.QueryUnescape(dst)
			if err != nil {
				return errToStatus(err), err
			}

			if !d.Check(dst) {
				return http.StatusForbidden, nil
			}

			if dst == "/" {
				return http.StatusForbidden, nil
			}

			override := item.Overwrite
			rename := item.Rename

			if !override && !rename {
				if _, err = d.user.Fs.Stat(dst); err == nil {
					return http.StatusConflict, nil
				}
			}

			// Permission for overwriting the file
			if override && !d.user.Perm.Modify {
				return http.StatusForbidden, nil
			}
		}

		return errToStatus(nil), nil
	})
}

func remoteResourcePostAction(
	agentID uint,
	action string,
	items []agents.ResourceItem,
	d *data,
) (int, *agents.BeforeCopyResponse, error) {
	switch action {
	// TODO: use enum
	case "remote-copy":
		// random uuid for archive file name
		uuid, err := exec.Command("uuidgen", "-r").Output()
		if err != nil {
			return http.StatusInternalServerError, nil, err
		}

		// execute remote copy operation
		agent, err := d.store.Agents.Get(agentID)
		if err != nil {
			return http.StatusInternalServerError, nil, err
		}

		client := agents.AgentClient{
			Agent: agent,
		}

		resp, status, err := client.RemoteCopy(agent.RemoteUser.ID, agent.Host, agent.Port, string(uuid), items)
		if err != nil {
			return status, nil, err
		}

		if resp.Code != 0 {
			return http.StatusInternalServerError, nil, fmt.Errorf("unexpected error12 %s", resp.Message)
		}

		// TODO: execute after-remote-copy on target agent or error

		// TODO:
		return http.StatusOK, resp, nil
	case "remote-rename":
		return http.StatusNotImplemented, nil, nil
	default:
		return http.StatusNotImplemented, nil, nil
	}
}
