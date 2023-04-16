package http

import (
	"encoding/json"
	"fmt"
	"net/http"
	"net/url"
	"os/exec"
	"path/filepath"
	"strings"

	"github.com/filebrowser/filebrowser/v2/agents"

	"golang.org/x/sys/unix"
)

var remoteResourceGetHandler = injectAgentWithUser(func(w http.ResponseWriter, r *http.Request, d *data) (int, error) {
	client := agents.AgentClient{
		Agent: d.agent,
	}

	authCookie, _ := r.Cookie("auth")

	resp, status, err := client.GetResource(d.agent, r.URL.Path, authCookie.Value)
	if err != nil {
		return http.StatusBadRequest, err
	}

	if status == http.StatusOK {
		return renderJSON(w, r, resp.Resource)
	}

	return status, fmt.Errorf("%s", resp.Error)
})

func remoteSourceResourcePostHandler() handleFunc {
	return injectAgentWithUser(func(w http.ResponseWriter, r *http.Request, d *data) (int, error) {
		var req []agents.ResourceItem
		err := json.NewDecoder(r.Body).Decode(&req)
		if err != nil {
			return http.StatusBadRequest, err
		}

		action := r.URL.Query().Get("action")
		for idx, item := range req {
			src := item.Source
			dst := item.Destination

			src, sErr := url.QueryUnescape(src)
			if sErr != nil {
				return errToStatus(err), err
			}

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

			if !checkReadable(src, d) {
				return http.StatusForbidden, fmt.Errorf("cannot read %s", src)
			}
		}

		//TODO: consider running hooks
		status, response, err := remoteResourcePostAction(r, action, req, d)
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

			if writeable, dir := checkWriteable(dst, d); !writeable {
				return http.StatusForbidden, fmt.Errorf("cannot write into %s", dir)
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

func checkWriteable(dst string, d *data) (writeable bool, base string) {
	dir := filepath.Dir(dst)
	scope := d.user.Scope
	if scope == "." {
		scope = ""
	}
	path := d.server.Root + scope + dir
	return unix.Access(path, unix.W_OK) == nil, dir
}

func checkReadable(src string, d *data) bool {
	scope := d.user.Scope
	if scope == "." {
		scope = ""
	}
	src = strings.Replace(src, "/files", "", 1)
	path := d.server.Root + scope + src
	return unix.Access(path, unix.R_OK) == nil
}

func remoteResourcePostAction(
	r *http.Request,
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
		client := agents.AgentClient{
			Agent: d.agent,
		}

		srcScope := d.user.Scope
		if srcScope == "." {
			srcScope = ""
		}

		authCookie, _ := r.Cookie("auth")

		resp, status, err := client.RemoteCopy(
			string(uuid),
			d.server.Root+srcScope,
			authCookie.Value,
			items,
		)
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
