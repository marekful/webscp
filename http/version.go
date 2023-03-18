package http

import (
	"net/http"

	"github.com/filebrowser/filebrowser/v2/version"
)

var versionHandler = func(w http.ResponseWriter, r *http.Request, d *data) (int, error) {
	w.Header().Set("Content-Type", "text/plain")
	w.Write([]byte(version.Version))
	return 0, nil
}
