package k1s0cache

import (
	"encoding/json"

	"github.com/vmihailenco/msgpack/v5"
)

// Serializer serializes and deserializes values.
type Serializer interface {
	// Marshal serializes a value to bytes.
	Marshal(v any) ([]byte, error)

	// Unmarshal deserializes bytes to a value.
	Unmarshal(data []byte, v any) error

	// Name returns the serializer name.
	Name() string
}

// JSONSerializer serializes values as JSON.
type JSONSerializer struct{}

// Marshal serializes a value to JSON bytes.
func (s *JSONSerializer) Marshal(v any) ([]byte, error) {
	return json.Marshal(v)
}

// Unmarshal deserializes JSON bytes to a value.
func (s *JSONSerializer) Unmarshal(data []byte, v any) error {
	return json.Unmarshal(data, v)
}

// Name returns "json".
func (s *JSONSerializer) Name() string {
	return "json"
}

// MsgpackSerializer serializes values as Msgpack.
type MsgpackSerializer struct{}

// Marshal serializes a value to Msgpack bytes.
func (s *MsgpackSerializer) Marshal(v any) ([]byte, error) {
	return msgpack.Marshal(v)
}

// Unmarshal deserializes Msgpack bytes to a value.
func (s *MsgpackSerializer) Unmarshal(data []byte, v any) error {
	return msgpack.Unmarshal(data, v)
}

// Name returns "msgpack".
func (s *MsgpackSerializer) Name() string {
	return "msgpack"
}

// NewSerializer creates a serializer by name.
func NewSerializer(name string) Serializer {
	switch name {
	case "msgpack":
		return &MsgpackSerializer{}
	default:
		return &JSONSerializer{}
	}
}
