package agents

import (
	"bytes"
	"fmt"
	"os"

	"encoding/json"
	nethttps "net/http"
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

type GetVersionResponse struct {
	Latency string `json:"latency"`
	Version string `json:"version"`
	Error   string `json:"error"`
}

func (c *AgentClient) ExchangeKeys(host string, port string, secret string) error {

	agentAddress := os.Getenv("AGENT_ADDRESS")
	requestURL := agentAddress + "/api/register-public-key"
	body := []byte(`{
		"host": "` + host + `", 
		"port": "` + port + `",
		"secret": "` + secret + `"
	}`)

	r, err := nethttps.NewRequest("POST", requestURL, bytes.NewBuffer(body))
	if err != nil {
		return fmt.Errorf("unexpected error: %v", err)
	}

	r.Header.Add("Content-Type", "application/json")

	client := &nethttps.Client{}
	res, err := client.Do(r)
	if err != nil {
		return fmt.Errorf("unexpected error: %v", err)
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

	if resp.Success != true {
		return fmt.Errorf("unexpected error")
	}

	return nil
}

func (c *AgentClient) GetVersion() GetVersionResponse {
	agentAddress := os.Getenv("AGENT_ADDRESS")
	requestURL := fmt.Sprintf("%s/api/version/%s/%s", agentAddress, c.Agent.Host, c.Agent.Port)

	returnVersion := ""
	returnError := ""

	r, err := nethttps.NewRequest("GET", requestURL, nil)
	if err != nil {
		returnError = fmt.Sprintf("unexpected error: %v", err)
	}

	client := &nethttps.Client{}
	res, err := client.Do(r)
	if err != nil {
		returnError = fmt.Sprintf("unexpected error: %v", err)
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
