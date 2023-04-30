package auth

import (
	"net/http"

	"github.com/marekful/webscp/settings"
	"github.com/marekful/webscp/users"
)

// Auther is the authentication interface.
type Auther interface {
	// Auth is called to authenticate a request.
	Auth(r *http.Request, usr users.Store, stg *settings.Settings, srv *settings.Server) (*users.User, error)
	// LoginPage indicates if this auther needs a login page.
	LoginPage() bool
	// ConfigChanged indicates if the provided config map is different compared to saved values
	ConfigChanged(config map[string]string) bool
}
