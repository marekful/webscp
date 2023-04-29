package bolt

import (
	"github.com/asdine/storm/v3"

	"github.com/marekful/webscp/agents"
	"github.com/marekful/webscp/auth"
	"github.com/marekful/webscp/settings"
	"github.com/marekful/webscp/share"
	"github.com/marekful/webscp/storage"
	"github.com/marekful/webscp/users"
)

// NewStorage creates a storage.Storage based on Bolt DB.
func NewStorage(db *storm.DB) (*storage.Storage, error) {
	userStore := users.NewStorage(usersBackend{db: db})
	agentStore := agents.NewStorage(agentsBackend{db: db})
	shareStore := share.NewStorage(shareBackend{db: db})
	settingsStore := settings.NewStorage(settingsBackend{db: db})
	authStore := auth.NewStorage(authBackend{db: db}, userStore)

	err := save(db, "version", 2) //nolint:gomnd
	if err != nil {
		return nil, err
	}

	return &storage.Storage{
		Auth:     authStore,
		Users:    userStore,
		Agents:   agentStore,
		Share:    shareStore,
		Settings: settingsStore,
	}, nil
}
