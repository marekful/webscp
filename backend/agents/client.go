package agents

import (
	"bytes"
	"encoding/json"
	"fmt"
	nethttps "net/http"
	neturl "net/url"
	"os"
	"strings"
)

type AgentBackend interface {
	GetTokenUser(userID uint, user *TokenUser, accessToken, token string) (status int, err error)
	ExchangeKeys(userID uint, host, port, secret, token string) (status int, err error)
	GetRemoteUser(userID uint, user *RemoteUser, token string) (status int, err error)
	GetResource(url, token string) (response *GetResourceResponse, status int, err error)
	RemoteCopy(archiveName, srcRoot, token string, items []ResourceItem, compress bool) (response *BeforeCopyResponse, status int, err error)
	CancelTransfer(transferID, token string) (status int, err error)
	GetVersion(token string) GetVersionResponse
}

type AgentClient struct {
	Agent *Agent
}

type GenerateAccessTokenResponse struct {
	Code       uint   `json:"code"`
	Token      string `json:"token"`
	ValidUntil int64  `json:"valid_until"`
	Error      string `json:"error"`
}

type ExchangeKeysResponse struct {
	Success bool   `json:"success"`
	Error   string `json:"error"`
}

type GetRemoteUserResponse struct {
	Code  int32  `json:"code"`
	ID    uint   `json:"id"`
	Token string `json:"token"`
	Error string `json:"error,omitempty"`
}

type GetTokenUserResponse struct {
	Code  int32  `json:"code"`
	ID    uint   `json:"id"`
	Name  string `json:"name"`
	Error string `json:"error,omitempty"`
}

type GetVersionResponse struct {
	Latency string  `json:"latency"`
	Version Version `json:"version"`
	Error   string  `json:"error"`
}

type Version struct {
	Agent string `json:"agent"`
	Files string `json:"files"`
}

type GetResourceResponse struct {
	Resource string `json:"resource"`
	Error    string `json:"error"`
}

type BeforeCopyResponse struct {
	Code    int32  `json:"code"`
	Message string `json:"message"`
}

type ResourceItem struct {
	Source      string `json:"source"`
	Destination string `json:"destination"`
	Overwrite   bool   `json:"overwrite"`
	Keep        bool   `json:"keep"`
}

type RemoteResourceAgentRequest struct {
	Items      []ResourceItem `json:"items"`
	Compress   bool           `json:"compress"`
	SourceRoot string         `json:"source_root"`
}

type CancelTransferRequest struct {
	TransferID string `json:"transfer_id"`
}

func GetTemporaryAccessToken(token string, userID uint) (response *GenerateAccessTokenResponse, status int, err error) {
	agentAddress := os.Getenv("AGENT_ADDRESS")
	requestURL := fmt.Sprintf("%s/api/users/%d/temporary-access-token", agentAddress, userID)

	r, err := nethttps.NewRequest("GET", requestURL, nethttps.NoBody)
	if err != nil {
		return nil, nethttps.StatusInternalServerError, fmt.Errorf("error initializing agent API reqeuest: %v", err)
	}

	cookie := nethttps.Cookie{Name: "rc_auth", Value: token}
	r.AddCookie(&cookie)

	client := &nethttps.Client{}
	agentResponse, err := client.Do(r)
	if err != nil {
		return nil, nethttps.StatusInternalServerError, fmt.Errorf("error sending agent API request: %v", err)
	}

	defer agentResponse.Body.Close()

	resp := &GenerateAccessTokenResponse{}
	dErr := json.NewDecoder(agentResponse.Body).Decode(resp)
	if dErr != nil {
		return nil, nethttps.StatusInternalServerError, dErr
	}

	if agentResponse.StatusCode != nethttps.StatusOK {
		return nil, agentResponse.StatusCode, fmt.Errorf("generate token error")
	}

	return resp, nethttps.StatusOK, nil
}

func (c *AgentClient) GetTokenUser(userID uint, user *TokenUser, accessToken, token string) (status int, err error) {
	agentAddress := os.Getenv("AGENT_ADDRESS")
	requestURL := fmt.Sprintf("%s/api/users/%d/connections/%s/%s/token-user", agentAddress, userID, c.Agent.Host, c.Agent.Port)
	body := []byte(`{
		"access_token": "` + accessToken + `"
	}`)

	r, err := nethttps.NewRequest("POST", requestURL, bytes.NewBuffer(body))
	if err != nil {
		message := fmt.Errorf("error initializing agent API request: %v", err)
		return nethttps.StatusInternalServerError, message
	}

	cookie := nethttps.Cookie{Name: "rc_auth", Value: token}
	r.AddCookie(&cookie)

	r.Header.Add("Content-Type", "application/json")

	client := &nethttps.Client{}
	agentResponse, err := client.Do(r)
	if err != nil {
		return nethttps.StatusServiceUnavailable, fmt.Errorf("error sending agent API request: %v", err)
	}

	defer agentResponse.Body.Close()

	resp := &GetTokenUserResponse{}
	dErr := json.NewDecoder(agentResponse.Body).Decode(resp)
	if dErr != nil {
		return nethttps.StatusInternalServerError, dErr
	}

	if agentResponse.StatusCode != nethttps.StatusOK {
		return agentResponse.StatusCode, fmt.Errorf("%s", resp.Error)
	}

	if len(resp.Error) > 0 {
		return nethttps.StatusServiceUnavailable, fmt.Errorf("%s", resp.Error)
	}

	user.ID = resp.ID
	user.Name = resp.Name

	return 0, nil
}

func (c *AgentClient) ExchangeKeys(userID uint, host, port, secret, token string) (status int, err error) {
	agentAddress := os.Getenv("AGENT_ADDRESS")
	requestURL := fmt.Sprintf("%s/api/users/%d/connections", agentAddress, userID)
	body := []byte(`{
		"host": "` + host + `", 
		"port": "` + port + `",
		"secret": "` + secret + `"
	}`)

	r, err := nethttps.NewRequest("POST", requestURL, bytes.NewBuffer(body))
	if err != nil {
		return nethttps.StatusInternalServerError, fmt.Errorf("error initializing agent API request: %v", err)
	}

	cookie := nethttps.Cookie{Name: "rc_auth", Value: token}
	r.AddCookie(&cookie)

	r.Header.Add("Content-Type", "application/json")

	client := &nethttps.Client{}
	agentResponse, err := client.Do(r)
	if err != nil {
		return nethttps.StatusInternalServerError, fmt.Errorf("error sending agent API request: %v", err)
	}

	defer agentResponse.Body.Close()

	resp := &ExchangeKeysResponse{}
	dErr := json.NewDecoder(agentResponse.Body).Decode(resp)
	if dErr != nil {
		return nethttps.StatusInternalServerError, dErr
	}

	if agentResponse.StatusCode != nethttps.StatusOK {
		return agentResponse.StatusCode, fmt.Errorf("token error: %s", resp.Error)
	}

	if !resp.Success {
		return nethttps.StatusInternalServerError, fmt.Errorf("unexpected error while sending agent API request")
	}

	return nethttps.StatusOK, nil
}

func (c *AgentClient) GetRemoteUser(userID uint, user *RemoteUser, token string) (status int, err error) {
	agentAddress := os.Getenv("AGENT_ADDRESS")
	requestURL := fmt.Sprintf("%s/api/users/%d/connections/%s/%s/login", agentAddress, userID, c.Agent.Host, c.Agent.Port)
	body := []byte(`{
		"name": "` + user.Name + `",
		"password": "` + user.Password + `"
	}`)

	r, err := nethttps.NewRequest("POST", requestURL, bytes.NewBuffer(body))
	if err != nil {
		message := fmt.Errorf("error initializing agent API request: %v", err)
		return nethttps.StatusInternalServerError, message
	}

	cookie := nethttps.Cookie{Name: "rc_auth", Value: token}
	r.AddCookie(&cookie)

	r.Header.Add("Content-Type", "application/json")

	client := &nethttps.Client{}
	agentResponse, err := client.Do(r)
	if err != nil {
		return nethttps.StatusServiceUnavailable, fmt.Errorf("error sending agent API request: %v", err)
	}

	defer agentResponse.Body.Close()

	resp := &GetRemoteUserResponse{}
	dErr := json.NewDecoder(agentResponse.Body).Decode(resp)
	if dErr != nil {
		return nethttps.StatusInternalServerError, dErr
	}

	if agentResponse.StatusCode != nethttps.StatusOK {
		return agentResponse.StatusCode, fmt.Errorf("%s", resp.Error)
	}

	if len(resp.Error) > 0 {
		return nethttps.StatusServiceUnavailable, fmt.Errorf("%s", resp.Error)
	}

	user.ID = resp.ID
	user.Token = resp.Token

	return 0, nil
}

func (c *AgentClient) GetResource(url, token string) (response *GetResourceResponse, status int, err error) {
	url = neturl.PathEscape(url)
	agentAddress := os.Getenv("AGENT_ADDRESS")
	requestURL := fmt.Sprintf("%s/api/agents/%d/resources/%s", agentAddress, c.Agent.ID, url)

	r, err := nethttps.NewRequest("GET", requestURL, nethttps.NoBody)
	if err != nil {
		return nil, nethttps.StatusInternalServerError, fmt.Errorf("error initializing agent API reqeuest: %v", err)
	}

	cookie := nethttps.Cookie{Name: "rc_auth", Value: token}
	r.AddCookie(&cookie)

	client := &nethttps.Client{}
	res, err := client.Do(r)
	if err != nil {
		return nil, nethttps.StatusInternalServerError, fmt.Errorf("error sending agent API request: %v", err)
	}

	defer res.Body.Close()

	resp := &GetResourceResponse{}
	dErr := json.NewDecoder(res.Body).Decode(resp)
	if dErr != nil {
		return nil, nethttps.StatusInternalServerError, dErr
	}

	return resp, res.StatusCode, nil
}

func (c *AgentClient) RemoteCopy(
	archiveName,
	srcRoot,
	token string,
	items []ResourceItem,
	compress bool,
) (response *BeforeCopyResponse, status int, err error) {
	agentAddress := os.Getenv("AGENT_ADDRESS")
	requestURL := fmt.Sprintf("%s/api/agents/%d/resources/%s", agentAddress, c.Agent.ID, strings.Trim(archiveName, "\n"))

	request := RemoteResourceAgentRequest{
		Items:      items,
		Compress:   compress,
		SourceRoot: srcRoot,
	}
	requestBody, err := json.Marshal(request)
	if err != nil {
		return nil, nethttps.StatusInternalServerError, fmt.Errorf("error decoding items: %v", err)
	}

	r, err := nethttps.NewRequest("PATCH", requestURL, bytes.NewReader(requestBody))
	if err != nil {
		return nil, nethttps.StatusInternalServerError, fmt.Errorf("error initializing agent API request: %v", err)
	}
	r.Header.Add("Content-Type", "application/json")

	cookie := nethttps.Cookie{Name: "rc_auth", Value: token}
	r.AddCookie(&cookie)

	client := &nethttps.Client{}
	agentResponse, err := client.Do(r)
	if err != nil {
		return nil, nethttps.StatusInternalServerError, fmt.Errorf("error sending agent API request: %v", err)
	}

	defer agentResponse.Body.Close()

	resp := &BeforeCopyResponse{}
	dErr := json.NewDecoder(agentResponse.Body).Decode(resp)
	if dErr != nil {
		return nil, nethttps.StatusInternalServerError, dErr
	}

	if agentResponse.StatusCode != nethttps.StatusOK {
		return nil, agentResponse.StatusCode, fmt.Errorf("%s", resp.Message)
	}

	return resp, nethttps.StatusOK, nil
}

func (c *AgentClient) CancelTransfer(transferID, token string) (status int, err error) {
	agentAddress := os.Getenv("AGENT_ADDRESS")
	requestURL := fmt.Sprintf("%s/api/agents/%d/transfers/%s", agentAddress, c.Agent.ID, transferID)

	r, err := nethttps.NewRequest("DELETE", requestURL, nethttps.NoBody)
	if err != nil {
		return nethttps.StatusInternalServerError, fmt.Errorf("error initializing agent API request: %v", err)
	}

	cookie := nethttps.Cookie{Name: "rc_auth", Value: token}
	r.AddCookie(&cookie)

	client := &nethttps.Client{}
	agentResponse, err := client.Do(r)
	if err != nil {
		return nethttps.StatusInternalServerError, fmt.Errorf("error sending agent API request: %v", err)
	}

	defer agentResponse.Body.Close()

	if agentResponse.StatusCode != nethttps.StatusOK {
		return agentResponse.StatusCode, fmt.Errorf("cancel transfer error: %s", agentResponse.Status)
	}

	return nethttps.StatusOK, nil
}

func (c *AgentClient) GetVersion(token string) GetVersionResponse {
	agentAddress := os.Getenv("AGENT_ADDRESS")
	requestURL := fmt.Sprintf("%s/api/agents/%d/version", agentAddress, c.Agent.ID)

	returnVersion := Version{
		Agent: "unknown",
		Files: "unknown",
	}
	returnError := ""

	r, err := nethttps.NewRequest("GET", requestURL, nethttps.NoBody)
	if err != nil {
		returnError = fmt.Sprintf("error initializing agent API request: %v", err)
	}

	cookie := nethttps.Cookie{Name: "rc_auth", Value: token}
	r.AddCookie(&cookie)

	client := &nethttps.Client{}
	res, err := client.Do(r)
	if err != nil {
		returnError = fmt.Sprintf("error sending agent API request %v", err)
	}

	defer res.Body.Close()

	resp := &GetVersionResponse{}
	dErr := json.NewDecoder(res.Body).Decode(resp)
	if dErr != nil {
		returnError = "decode error: " + dErr.Error()
	}

	if resp.Error != "" {
		returnError = resp.Error
	} else {
		returnVersion = resp.Version
	}

	return GetVersionResponse{
		Version: returnVersion,
		Error:   returnError,
		Latency: resp.Latency,
	}
}
