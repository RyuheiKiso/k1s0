package buildingblocks

import (
	"encoding/json"
	"testing"
)

func TestMetadataCreation(t *testing.T) {
	m := Metadata{
		Name:    "test-component",
		Version: "v1.0.0",
		Tags:    map[string]string{"env": "prod"},
	}
	if m.Name != "test-component" {
		t.Errorf("expected Name 'test-component', got %q", m.Name)
	}
	if m.Version != "v1.0.0" {
		t.Errorf("expected Version 'v1.0.0', got %q", m.Version)
	}
	if m.Tags["env"] != "prod" {
		t.Errorf("expected tag env=prod, got %q", m.Tags["env"])
	}
}

func TestMetadataJSONMarshal(t *testing.T) {
	m := Metadata{
		Name:    "my-component",
		Version: "v2.0.0",
		Tags:    map[string]string{"region": "us-east"},
	}
	data, err := json.Marshal(m)
	if err != nil {
		t.Fatalf("failed to marshal Metadata: %v", err)
	}

	var decoded Metadata
	if err := json.Unmarshal(data, &decoded); err != nil {
		t.Fatalf("failed to unmarshal Metadata: %v", err)
	}
	if decoded.Name != m.Name || decoded.Version != m.Version {
		t.Errorf("roundtrip mismatch: got %+v", decoded)
	}
	if decoded.Tags["region"] != "us-east" {
		t.Errorf("expected tag region=us-east, got %q", decoded.Tags["region"])
	}
}

func TestMetadataJSONOmitEmptyTags(t *testing.T) {
	m := Metadata{Name: "simple", Version: "v1"}
	data, err := json.Marshal(m)
	if err != nil {
		t.Fatalf("failed to marshal: %v", err)
	}
	var raw map[string]any
	if err := json.Unmarshal(data, &raw); err != nil {
		t.Fatalf("failed to unmarshal: %v", err)
	}
	if _, ok := raw["tags"]; ok {
		t.Error("expected 'tags' to be omitted when nil")
	}
}

func TestComponentStatusConstants(t *testing.T) {
	statuses := map[ComponentStatus]string{
		StatusUninitialized: "uninitialized",
		StatusReady:         "ready",
		StatusDegraded:      "degraded",
		StatusClosed:        "closed",
		StatusError:         "error",
	}
	for status, expected := range statuses {
		if string(status) != expected {
			t.Errorf("expected %q, got %q", expected, string(status))
		}
	}
}
