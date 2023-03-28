package agents

import (
	"github.com/filebrowser/filebrowser/v2/errors"
)

// ViewMode describes a view mode.
type ViewMode string

// Agent describes an agent.
type Agent struct {
	ID     uint   `storm:"id,increment" json:"id"`
	Host   string `json:"host"`
	Port   string `json:"port"`
	Secret string `json:"secret,omitempty"`
}

var checkableFields = []string{
	"Host",
	"Port",
	"Secret",
}

// Clean cleans up a agent and verifies if all its fields
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
