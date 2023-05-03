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

	"github.com/marekful/webscp/agents"
	"github.com/marekful/webscp/errors"
	"github.com/marekful/webscp/users"
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
	ID    uint   `json:"id"`
	Token string `json:"token"`
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
}

func injectPath(r *http.Request) {
	vars := mux.Vars(r)
	if len(vars["url"]) > 0 {
		decodedURL, err := url.QueryUnescape(vars["url"])
		if err == nil {
			r.URL.Path = decodedURL
		}
	}
}

func checkHost(r *http.Request) error {
	internalHost := os.Getenv("INTERNAL_ADDRESS")
	requestHost := "http://" + r.Host
	if requestHost != internalHost {
		return fmt.Errorf("error: %s does not match %s", internalHost, requestHost)
	}

	return nil
}

// The withAgent middleware enforces
func withAgent(fn handleFunc) handleFunc {
	return func(w http.ResponseWriter, r *http.Request, d *data) (int, error) {
		// Deny access to requests whose 'Host' header doesn't match the internal docker address (e.g. http://filebrowser:80).
		err := checkHost(r)
		if err != nil {
			return http.StatusUnauthorized, err
		}

		injectAgent(r, d)
		injectPath(r)

		return fn(w, r, d)
	}
}

// The withAgentUser middleware is used for remote operations
// on the remote side to enforce valid user session.
func withAgentUser(fn handleFunc) handleFunc {
	return withUser(func(w http.ResponseWriter, r *http.Request, d *data) (int, error) {
		// Deny access to requests whose 'Host' header doesn't match the internal docker address (e.g. http://filebrowser:80).
		err := checkHost(r)
		if err != nil {
			return http.StatusUnauthorized, err
		}

		// Fetch the user referred to in query params
		vars := mux.Vars(r)
		userID := vars["user_id"]
		id64, err := strconv.ParseUint(userID, 10, 64)
		if err != nil {
			return http.StatusUnauthorized, nil
		}

		user, dErr := d.store.Users.Get(d.server.Root, id64)
		if dErr != nil {
			return http.StatusUnauthorized, nil
		}

		// Enforce that the referred user is the same as the one extracted from the token
		if user.ID != d.user.ID {
			return http.StatusUnauthorized, nil
		}

		injectPath(r)

		return fn(w, r, d)
	})
}

// The injectAgentWithUser middleware is used for remote operations
// on the local side to enforce valid user session.
func injectAgentWithUser(fn handleFunc) handleFunc {
	return withUser(func(w http.ResponseWriter, r *http.Request, d *data) (int, error) {
		injectAgent(r, d)
		injectPath(r)

		if d.agent.UserID != d.user.ID {
			return http.StatusForbidden, nil
		}

		return fn(w, r, d)
	})
}

var agentsGetHandler = withUser(func(w http.ResponseWriter, r *http.Request, d *data) (int, error) {
	agents, err := d.store.Agents.FindByUserID(d.user.ID)
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

	authCookie, _ := r.Cookie("auth")

	user := agents.TokenUser{}

	userStatus, err := client.GetTokenUser(d.user.ID, &user, req.Data.Secret, authCookie.Value)
	if err != nil {
		if userStatus == http.StatusUnauthorized {
			userStatus = http.StatusForbidden
		}
		return userStatus, err
	}

	kexStatus, err := client.ExchangeKeys(d.user.ID, req.Data.Host, req.Data.Port, req.Data.Secret, authCookie.Value)
	if err != nil {
		if kexStatus == http.StatusUnauthorized {
			kexStatus = http.StatusForbidden
		}
		return kexStatus, err
	}

	req.Data.Secret = ""
	// req.Data.RemoteUser.Password = ""
	req.Data.UserID = d.user.ID
	req.Data.RemoteUser.ID = user.ID
	req.Data.RemoteUser.Name = user.Name
	req.Data.RemoteUser.Token = "x.0"

	err = d.store.Agents.Save(req.Data)
	if err != nil {
		return http.StatusInternalServerError, err
	}

	w.Header().Set("Location", "/settings/agents")
	return http.StatusCreated, nil
})

var agentTemporaryAccessTokenGetHandler = withUser(func(w http.ResponseWriter, r *http.Request, d *data) (int, error) {
	authCookie, _ := r.Cookie("auth")

	accessTokenResponse, httpStatus, err := agents.GetTemporaryAccessToken(authCookie.Value, d.user.ID)
	if err != nil {
		if httpStatus == http.StatusUnauthorized {
			httpStatus = http.StatusForbidden
		}
		return httpStatus, err
	}

	return renderJSON(w, r, accessTokenResponse)
})

var remoteVerifyUserCredentialsPostHandler = injectAgentWithUser(func(w http.ResponseWriter, r *http.Request, d *data) (int, error) {
	req := &getUserRequest{}
	err := json.NewDecoder(r.Body).Decode(req)
	if err != nil {
		return http.StatusInternalServerError, err
	}

	user := agents.RemoteUser{Name: req.Name, Password: req.Password}

	client := agents.AgentClient{
		Agent: d.agent,
	}

	authCookie, _ := r.Cookie("auth")

	userStatus, err := client.GetRemoteUser(d.user.ID, &user, authCookie.Value)
	if err != nil {
		if userStatus == http.StatusUnauthorized {
			userStatus = http.StatusForbidden
		}
		return userStatus, err
	}

	user.Password = ""
	d.agent.RemoteUser = user

	err = d.store.Agents.Save(d.agent)
	if err != nil {
		return http.StatusInternalServerError, err
	}

	return 0, nil
})

var agentVerifyUserCredentialsPostHandler = withAgent(func(w http.ResponseWriter, r *http.Request, d *data) (int, error) {
	req := &getUserRequest{}
	err := json.NewDecoder(r.Body).Decode(req)
	if err != nil {
		return http.StatusInternalServerError, err
	}

	u, err := d.store.Users.Get(d.server.Root, req.Name)
	if err != nil || !users.CheckPwd(req.Password, u.Password) {
		return http.StatusUnauthorized, os.ErrPermission
	}

	token, err := getToken(r, d, u)
	if err != nil {
		return http.StatusInternalServerError, err
	}

	return renderJSON(w, r, getUserResponse{ID: u.ID, Token: token})
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

	authCookie, _ := r.Cookie("auth")

	client := agents.AgentClient{Agent: agent}
	version := client.GetVersion(authCookie.Value)

	versionResponse := agents.GetVersionResponse{
		Latency: version.Latency,
		Version: version.Version,
		Error:   version.Error,
	}

	return renderJSON(w, r, versionResponse)
})
