package agents

import (
	"github.com/filebrowser/filebrowser/v2/errors"
	"sync"
)

// StorageBackend is the interface to implement for remote agents storage.
type StorageBackend interface {
	GetBy(interface{}) (*Agent, error)
	Gets() ([]*Agent, error)
	Save(a *Agent) error
	Update(a *Agent, fields ...string) error
	DeleteByID(uint) error
}

type Store interface {
	Get(id interface{}) (user *Agent, err error)
	Gets() ([]*Agent, error)
	Save(agent *Agent) error
	Delete(id interface{}) error
	LastUpdate(id uint) int64
}

// Storage is a agents storage.
type Storage struct {
	back    StorageBackend
	updated map[uint]int64
	mux     sync.RWMutex
}

// NewStorage creates a users storage from a backend.
func NewStorage(back StorageBackend) *Storage {
	return &Storage{
		back:    back,
		updated: map[uint]int64{},
	}
}

func (s *Storage) Get(id interface{}) (agent *Agent, err error) {
	agent, err = s.back.GetBy(id)
	if err != nil {
		return
	}
	if err := agent.Clean(); err != nil {
		return nil, err
	}
	return
}

// Gets gets a list of all users.
func (s *Storage) Gets() ([]*Agent, error) {
	agents, err := s.back.Gets()
	if err != nil {
		return nil, err
	}

	for _, agent := range agents {
		if err := agent.Clean(); err != nil { //nolint:govet
			return nil, err
		}
	}

	return agents, err
}

// Save saves the agent in a storage.
func (s *Storage) Save(agent *Agent) error {
	if err := agent.Clean(""); err != nil {
		return err
	}

	return s.back.Save(agent)
}

// Delete allows you to delete an agent by its name or username. The provided
// id must be a string for username lookup or a uint for id lookup. If id
// is neither, a ErrInvalidDataType will be returned.
func (s *Storage) Delete(id interface{}) error {
	switch id := id.(type) {
	case uint:
		return s.back.DeleteByID(id)
	default:
		return errors.ErrInvalidDataType
	}
}

// LastUpdate gets the timestamp for the last update of an user.
func (s *Storage) LastUpdate(id uint) int64 {
	s.mux.RLock()
	defer s.mux.RUnlock()
	if val, ok := s.updated[id]; ok {
		return val
	}
	return 0
}
