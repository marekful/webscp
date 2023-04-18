package storage

import (
	"github.com/marekful/webscp/agents"
	"github.com/marekful/webscp/auth"
	"github.com/marekful/webscp/settings"
	"github.com/marekful/webscp/share"
	"github.com/marekful/webscp/users"
)

// Storage is a storage powered by a Backend which makes the necessary
// verifications when fetching and saving data to ensure consistency.
type Storage struct {
	Users    users.Store
	Share    *share.Storage
	Auth     *auth.Storage
	Agents   *agents.Storage
	Settings *settings.Storage
}

const IDFieldName = "ID"
