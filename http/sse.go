package http

import (
	"fmt"
	"log"
	"net/http"

	"github.com/gorilla/mux"
)

var channelList = make(map[string]chan string)

func addSSEHaders(w http.ResponseWriter) {
	w.Header().Set("Content-Type", "text/event-stream")
	w.Header().Set("Cache-Control", "no-cache")
	w.Header().Set("Connection", "keep-alive")
	w.Header().Set("Access-Control-Allow-Origin", "*")
}

func writeData(w http.ResponseWriter, chanID string) (int, error) {
	return fmt.Fprintf(w, "data: %s\n\n", <-channelList[chanID])
}

var sseTransferPollGetHandler = withUser(func(w http.ResponseWriter, r *http.Request, d *data) (int, error) {
	vars := mux.Vars(r)
	chanID := vars["id"]

	addSSEHaders(w)

	// initialize message chan
	channelList[chanID] = make(chan string)

	defer func() {
		close(channelList[chanID])
		channelList[chanID] = nil
	}()

	flusher, _ := w.(http.Flusher)
	for {
		write, err := writeData(w, chanID)
		if err != nil {
			log.Println(err)
		}
		log.Println(write)
		flusher.Flush()
	}
})

var sseTransferUpdateGetHandler = withAgent(func(w http.ResponseWriter, r *http.Request, d *data) (int, error) {
	vars := mux.Vars(r)
	chanID := vars["id"]
	message := vars["message"]

	if channelList[chanID] != nil {
		channelList[chanID] <- message

		return 0, nil
	}

	return http.StatusNotFound, nil
})
