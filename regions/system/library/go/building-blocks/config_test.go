package buildingblocks

import (
	"testing"
)

// ParseComponentsConfig が有効な YAML からコンポーネント設定を正しくパースすることを確認する。
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

// ParseComponentsConfig が無効な YAML に対してエラーを返すことを確認する。
func TestParseComponentsConfigInvalid(t *testing.T) {
	invalidYAML := []byte(`{{{invalid yaml`)
	_, err := ParseComponentsConfig(invalidYAML)
	if err == nil {
		t.Fatal("expected error for invalid YAML, got nil")
	}
}

// ParseComponentsConfig が空のコンポーネントリストを持つ YAML を正しくパースすることを確認する。
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

// LoadComponentsConfig が存在しないファイルパスを指定するとエラーを返すことを確認する。
func TestLoadComponentsConfigMissingFile(t *testing.T) {
	_, err := LoadComponentsConfig("/nonexistent/path/components.yaml")
	if err == nil {
		t.Fatal("expected error for missing file, got nil")
	}
}
