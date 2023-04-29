package http

import (
	"net/http"

	"github.com/marekful/webscp/version"
)

var versionHandler = func(w http.ResponseWriter, r *http.Request, d *data) (int, error) {
	w.Header().Set("Content-Type", "text/plain")
	_, err := w.Write([]byte(version.Version))
	if err != nil {
		return http.StatusInternalServerError, err
	}
	return 0, nil
}
