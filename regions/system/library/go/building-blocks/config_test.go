package buildingblocks

import (
	"testing"
)

func TestParseComponentsConfigValid(t *testing.T) {
	yamlData := []byte(`
components:
  - name: redis-state
    type: state.redis
    version: v1
    metadata:
      host: localhost
      port: "6379"
  - name: kafka-pubsub
    type: pubsub.kafka
`)
	config, err := ParseComponentsConfig(yamlData)
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	if len(config.Components) != 2 {
		t.Fatalf("expected 2 components, got %d", len(config.Components))
	}
	c := config.Components[0]
	if c.Name != "redis-state" {
		t.Errorf("expected Name 'redis-state', got %q", c.Name)
	}
	if c.Type != "state.redis" {
		t.Errorf("expected Type 'state.redis', got %q", c.Type)
	}
	if c.Version != "v1" {
		t.Errorf("expected Version 'v1', got %q", c.Version)
	}
	if c.Metadata["host"] != "localhost" {
		t.Errorf("expected metadata host=localhost, got %q", c.Metadata["host"])
	}

	c2 := config.Components[1]
	if c2.Name != "kafka-pubsub" {
		t.Errorf("expected Name 'kafka-pubsub', got %q", c2.Name)
	}
}

func TestParseComponentsConfigInvalid(t *testing.T) {
	invalidYAML := []byte(`{{{invalid yaml`)
	_, err := ParseComponentsConfig(invalidYAML)
	if err == nil {
		t.Fatal("expected error for invalid YAML, got nil")
	}
}

func TestParseComponentsConfigEmpty(t *testing.T) {
	yamlData := []byte(`components: []`)
	config, err := ParseComponentsConfig(yamlData)
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	if len(config.Components) != 0 {
		t.Errorf("expected 0 components, got %d", len(config.Components))
	}
}

func TestLoadComponentsConfigMissingFile(t *testing.T) {
	_, err := LoadComponentsConfig("/nonexistent/path/components.yaml")
	if err == nil {
		t.Fatal("expected error for missing file, got nil")
	}
}
