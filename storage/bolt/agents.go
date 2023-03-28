package bolt

import (
	"fmt"
	"github.com/filebrowser/filebrowser/v2/agents"
	"reflect"

	"github.com/asdine/storm/v3"

	"github.com/filebrowser/filebrowser/v2/errors"
)

type agentsBackend struct {
	db *storm.DB
}

func (st agentsBackend) GetBy(i interface{}) (agent *agents.Agent, err error) {
	agent = &agents.Agent{}

	var arg string
	switch i.(type) {
	case uint:
		arg = "ID"
	case string:
		arg = "Host"
	default:
		return nil, errors.ErrInvalidDataType
	}

	err = st.db.One(arg, i, agent)

	if err != nil {
		if err == storm.ErrNotFound {
			return nil, errors.ErrNotExist
		}
		return nil, err
	}

	return
}

func (st agentsBackend) Gets() ([]*agents.Agent, error) {
	var allAgents []*agents.Agent
	err := st.db.All(&allAgents)
	if err == storm.ErrNotFound {
		return nil, errors.ErrNotExist
	}

	if err != nil {
		return allAgents, err
	}

	return allAgents, err
}

func (st agentsBackend) Update(agent *agents.Agent, fields ...string) error {
	if len(fields) == 0 {
		return st.Save(agent)
	}

	for _, field := range fields {
		agentField := reflect.ValueOf(agent).Elem().FieldByName(field)
		if !agentField.IsValid() {
			return fmt.Errorf("invalid field: %s", field)
		}
		val := agentField.Interface()
		if err := st.db.UpdateField(agent, field, val); err != nil {
			return err
		}
	}

	return nil
}

func (st agentsBackend) Save(agent *agents.Agent) error {
	err := st.db.Save(agent)
	if err == storm.ErrAlreadyExists {
		return errors.ErrExist
	}
	return err
}

func (st agentsBackend) DeleteByID(id uint) error {
	return st.db.DeleteStruct(&agents.Agent{ID: id})
}