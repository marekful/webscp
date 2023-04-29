package bolt

import (
	"github.com/asdine/storm/v3"

	"github.com/marekful/webscp/auth"
	"github.com/marekful/webscp/errors"
	"github.com/marekful/webscp/settings"
)

type authBackend struct {
	db *storm.DB
}

func (s authBackend) Get(t settings.AuthMethod) (auth.Auther, error) {
	var auther auth.Auther

	switch t {
	case auth.MethodJSONAuth:
		auther = &auth.JSONAuth{}
	case auth.MethodOIDCAuth:
		auther = &auth.OIDCAuth{}
	case auth.MethodProxyAuth:
		auther = &auth.ProxyAuth{}
	case auth.MethodHookAuth:
		auther = &auth.HookAuth{}
	case auth.MethodNoAuth:
		auther = &auth.NoAuth{}
	default:
		return nil, errors.ErrInvalidAuthMethod
	}

	return auther, get(s.db, "auther", auther)
}

func (s authBackend) Save(a auth.Auther) error {
	return save(s.db, "auther", a)
}
