package auth

import (
	"net/http"

	"github.com/marekful/webscp/settings"
	"github.com/marekful/webscp/users"
)

// MethodNoAuth is used to identify no auth.
const MethodNoAuth settings.AuthMethod = "noauth"

// NoAuth is no auth implementation of auther.
type NoAuth struct{}

// Auth uses authenticates user 1.
func (a NoAuth) Auth(r *http.Request, usr users.Store, stg *settings.Settings, srv *settings.Server) (*users.User, error) {
	return usr.Get(srv.Root, uint(1))
}

// LoginPage tells that no auth doesn't require a login page.
func (a NoAuth) LoginPage() bool {
	return false
}

// ConfigChanged tells if the provided config values are different compared to saved values.
func (a NoAuth) ConfigChanged(config map[string]string) bool {
	return false
}
