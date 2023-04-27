package agents

import (
	"github.com/marekful/webscp/errors"
)

// ViewMode describes a view mode.
type ViewMode string

// Agent describes an agent.
type Agent struct {
	ID         uint       `storm:"id,increment" json:"id"`
	UserID     uint       `json:"userID"`
	Host       string     `json:"host"`
	Port       string     `json:"port"`
	Secret     string     `json:"secret,omitempty"`
	RemoteUser RemoteUser `json:"remote_user"`
}

type RemoteUser struct {
	ID       uint   `storm:"index" json:"id"`
	Name     string `json:"name"`
	Password string `json:"password,omitempty"`
	Token    string `json:"token"`
}

type TokenUser struct {
	ID   uint   `json:"id"`
	Name string `json:"name"`
}

var checkableFields = []string{
	"Host",
	"Port",
	"Secret",
}

// Clean cleans up an agent and verifies if all its fields
// are alright to be saved.
func (a *Agent) Clean(fields ...string) error {
	if len(fields) == 0 {
		fields = checkableFields
	}

	for _, field := range fields {
		switch field {
		case "Host":
			if a.Host == "" {
				return errors.ErrEmptyAgentHost
			}
		case "Port":
			if a.Port == "" {
				return errors.ErrEmptyAgentPort
			}
		case "Secret":
			a.Secret = ""
		}
	}

	return nil
}
