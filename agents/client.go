package agents

import (
	"bytes"
	"fmt"
	"os"
	"strings"

	"encoding/json"
	nethttps "net/http"
	neturl "net/url"
)

type AgentBackend interface {
	ExchangeKeys(host string, port string, secret string) error
}

type AgentClient struct {
	Agent *Agent
}

type ExchangeKeysResponse struct {
	Success bool   `json:"success"`
	Error   string `json:"error"`
}

type GetRemoteUserResponse struct {
	Code  int32  `json:"code"`
	ID    uint   `json:"id"`
	Root  string `json:"root"`
	Error string `json:"error,omitempty"`
}

type GetVersionResponse struct {
	Latency string `json:"latency"`
	Version string `json:"version"`
	Error   string `json:"error"`
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
	Rename      bool   `json:"rename"`
}

type RemoteResourceAgentRequest struct {
	Items           []ResourceItem `json:"items"`
	SourceRoot      string         `json:"source_root"`
	DestinationRoot string         `json:"destination_root"`
}

type CancelTransferRequest struct {
	TransferID string `json:"transfer_id"`
}

func (c *AgentClient) ExchangeKeys(host, port, secret string) error {
	agentAddress := os.Getenv("AGENT_ADDRESS")
	requestURL := agentAddress + "/api/register-public-key"
	body := []byte(`{
		"host": "` + host + `", 
		"port": "` + port + `",
		"secret": "` + secret + `"
	}`)

	r, err := nethttps.NewRequest("POST", requestURL, bytes.NewBuffer(body))
	if err != nil {
		return fmt.Errorf("error initializing agent API request: %v", err)
	}

	r.Header.Add("Content-Type", "application/json")

	client := &nethttps.Client{}
	res, err := client.Do(r)
	if err != nil {
		return fmt.Errorf("error sending agent API request: %v", err)
	}

	defer res.Body.Close()

	resp := &ExchangeKeysResponse{}
	dErr := json.NewDecoder(res.Body).Decode(resp)
	if dErr != nil {
		return dErr
	}

	if len(resp.Error) > 0 {
		return fmt.Errorf("error connecting to host: %s", resp.Error)
	}

	if !resp.Success {
		return fmt.Errorf("unexpected error while sending agent API request")
	}

	return nil
}

func (c *AgentClient) GetRemoteUserID(user *RemoteUser) (status int, err error) {
	agentAddress := os.Getenv("AGENT_ADDRESS")
	requestURL := agentAddress + "/api/get-remote-user/" + c.Agent.Host + "/" + c.Agent.Port
	body := []byte(`{
		"name": "` + user.Name + `",
		"password": "` + user.Password + `"
	}`)

	r, err := nethttps.NewRequest("POST", requestURL, bytes.NewBuffer(body))
	if err != nil {
		message := fmt.Errorf("error initializing agent API request: %v", err)
		return nethttps.StatusInternalServerError, message
	}

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
	user.Root = resp.Root

	return 0, nil
}

func (c *AgentClient) GetVersion() GetVersionResponse {
	agentAddress := os.Getenv("AGENT_ADDRESS")
	requestURL := fmt.Sprintf("%s/api/version/%s/%s", agentAddress, c.Agent.Host, c.Agent.Port)

	returnVersion := ""
	returnError := ""

	r, err := nethttps.NewRequest("GET", requestURL, nethttps.NoBody)
	if err != nil {
		returnError = fmt.Sprintf("error initializing agent API request: %v", err)
	}

	client := &nethttps.Client{}
	res, err := client.Do(r)
	if err != nil {
		returnError = fmt.Sprintf("error sending agent API request %v", err)
	}

	defer res.Body.Close()

	resp := &GetVersionResponse{}
	dErr := json.NewDecoder(res.Body).Decode(resp)
	if dErr != nil {
		returnError = dErr.Error()
	}

	returnVersion = resp.Version
	if resp.Version == "" {
		returnVersion = "unknown"
	}

	if resp.Error != "" {
		returnError = resp.Error
	}

	return GetVersionResponse{
		Version: returnVersion,
		Error:   returnError,
		Latency: resp.Latency,
	}
}

func (c *AgentClient) GetResource(userID uint, host, port, url string) (response *GetResourceResponse, err error) {
	url = neturl.QueryEscape(url)
	agentAddress := os.Getenv("AGENT_ADDRESS")
	requestURL := fmt.Sprintf("%s/api/resources/%s/%s/%d/%s", agentAddress, host, port, userID, url)

	r, err := nethttps.NewRequest("GET", requestURL, nethttps.NoBody)
	if err != nil {
		return nil, fmt.Errorf("error initializing agent API reqeuest: %v", err)
	}

	client := &nethttps.Client{}
	res, err := client.Do(r)
	if err != nil {
		return nil, fmt.Errorf("error sending agent API request: %v", err)
	}

	defer res.Body.Close()

	resp := &GetResourceResponse{}
	dErr := json.NewDecoder(res.Body).Decode(resp)
	if dErr != nil {
		return nil, dErr
	}

	return resp, nil
}

func (c *AgentClient) RemoteCopy(
	userID uint,
	host,
	port,
	archiveName,
	srcRoot,
	dstRoot string,
	items []ResourceItem,
) (response *BeforeCopyResponse, status int, err error) {
	agentAddress := os.Getenv("AGENT_ADDRESS")
	requestURL := fmt.Sprintf("%s/api/copy/%s/%s/%d/%s", agentAddress, host, port, userID, strings.Trim(archiveName, "\n"))
	request := RemoteResourceAgentRequest{Items: items, SourceRoot: srcRoot, DestinationRoot: dstRoot}
	requestBody, err := json.Marshal(request)
	if err != nil {
		return nil, nethttps.StatusInternalServerError, fmt.Errorf("error decoding items: %v", err)
	}

	r, err := nethttps.NewRequest("POST", requestURL, bytes.NewReader(requestBody))
	if err != nil {
		return nil, nethttps.StatusInternalServerError, fmt.Errorf("error initializing agent API request: %v", err)
	}
	r.Header.Add("Content-Type", "application/json")
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
		return nil, agentResponse.StatusCode, fmt.Errorf("unexpected error: %s", resp.Message)
	}

	return resp, nethttps.StatusOK, nil
}

func (c *AgentClient) CancelTransfer(
	transferID string,
) (status int, err error) {
	agentAddress := os.Getenv("AGENT_ADDRESS")
	requestURL := fmt.Sprintf("%s/api/transfers/%s", agentAddress, transferID)

	r, err := nethttps.NewRequest("DELETE", requestURL, nethttps.NoBody)
	if err != nil {
		return nethttps.StatusInternalServerError, fmt.Errorf("error initializing agent API request: %v", err)
	}
	client := &nethttps.Client{}
	agentResponse, err := client.Do(r)
	if err != nil {
		return nethttps.StatusInternalServerError, fmt.Errorf("error sending agent API request: %v", err)
	}

	defer agentResponse.Body.Close()

	if agentResponse.StatusCode != nethttps.StatusOK {
		return agentResponse.StatusCode, fmt.Errorf("unexpected error: %s", agentResponse.Status)
	}

	return nethttps.StatusOK, nil
}
