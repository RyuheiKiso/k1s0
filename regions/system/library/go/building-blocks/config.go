package buildingblocks

import (
	"fmt"
	"os"

	"gopkg.in/yaml.v3"
)

// ComponentConfig represents a single component's configuration.
type ComponentConfig struct {
	Name     string            `yaml:"name" validate:"required"`
	Type     string            `yaml:"type" validate:"required"`
	Version  string            `yaml:"version,omitempty"`
	Metadata map[string]string `yaml:"metadata,omitempty"`
}

// ComponentsConfig represents the full components.yaml configuration.
type ComponentsConfig struct {
	Components []ComponentConfig `yaml:"components" validate:"required,dive"`
}

// LoadComponentsConfig loads and parses a components.yaml file.
func LoadComponentsConfig(path string) (*ComponentsConfig, error) {
	data, err := os.ReadFile(path)
	if err != nil {
		return nil, fmt.Errorf("failed to read components config: %w", err)
	}

	return ParseComponentsConfig(data)
}

// ParseComponentsConfig parses components.yaml from bytes.
func ParseComponentsConfig(data []byte) (*ComponentsConfig, error) {
	var config ComponentsConfig
	if err := yaml.Unmarshal(data, &config); err != nil {
		return nil, fmt.Errorf("failed to parse components config: %w", err)
	}
	return &config, nil
}
