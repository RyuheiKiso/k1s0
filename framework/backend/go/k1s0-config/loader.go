package k1s0config

import (
	"os"

	"gopkg.in/yaml.v3"
)

// ConfigLoader loads configuration from YAML files.
type ConfigLoader struct {
	options        *ConfigOptions
	secretResolver *SecretResolver
	rawContent     []byte
}

// NewConfigLoader creates a new ConfigLoader.
//
// Returns an error if:
//   - The environment name is invalid
//   - The config file is required but doesn't exist
//   - The secrets directory is required but doesn't exist
func NewConfigLoader(options *ConfigOptions) (*ConfigLoader, error) {
	// Validate environment name
	if !options.IsValidEnv() {
		return nil, NewInvalidEnvError(options.Env)
	}

	configPath := options.EffectiveConfigPath()

	// Check config file existence
	if options.RequireConfigFile {
		if _, err := os.Stat(configPath); os.IsNotExist(err) {
			return nil, NewConfigNotFoundError(configPath)
		}
	}

	// Check secrets directory existence
	if options.RequireSecretsDir {
		if _, err := os.Stat(options.SecretsDir); os.IsNotExist(err) {
			return nil, NewSecretsDirNotFoundError(options.SecretsDir)
		}
	}

	return &ConfigLoader{
		options:        options,
		secretResolver: NewSecretResolver(options.SecretsDir),
	}, nil
}

// Load reads the configuration file and unmarshals it into the target.
func (l *ConfigLoader) Load(target interface{}) error {
	configPath := l.options.EffectiveConfigPath()

	// Check if file exists
	if _, err := os.Stat(configPath); os.IsNotExist(err) {
		if l.options.RequireConfigFile {
			return NewConfigNotFoundError(configPath)
		}
		// Return empty config
		return yaml.Unmarshal([]byte("{}"), target)
	}

	content, err := os.ReadFile(configPath)
	if err != nil {
		return NewConfigReadError(configPath, err)
	}

	l.rawContent = content

	if err := yaml.Unmarshal(content, target); err != nil {
		return NewConfigParseError(configPath, err)
	}

	return nil
}

// LoadRaw reads the configuration file and returns the raw YAML content.
func (l *ConfigLoader) LoadRaw() ([]byte, error) {
	if l.rawContent != nil {
		return l.rawContent, nil
	}

	configPath := l.options.EffectiveConfigPath()

	if _, err := os.Stat(configPath); os.IsNotExist(err) {
		if l.options.RequireConfigFile {
			return nil, NewConfigNotFoundError(configPath)
		}
		return []byte{}, nil
	}

	content, err := os.ReadFile(configPath)
	if err != nil {
		return nil, NewConfigReadError(configPath, err)
	}

	l.rawContent = content
	return content, nil
}

// ResolveSecretFile resolves a secret value from a file reference.
//
// The fileValue is the value of a *_file key in the configuration.
// The key is the configuration key name (e.g., "db.password_file").
func (l *ConfigLoader) ResolveSecretFile(fileValue, key string) (string, error) {
	return l.secretResolver.Resolve(fileValue, key)
}

// SecretResolver returns the secret resolver.
func (l *ConfigLoader) SecretResolver() *SecretResolver {
	return l.secretResolver
}

// Options returns the configuration options.
func (l *ConfigLoader) Options() *ConfigOptions {
	return l.options
}

// Env returns the environment name.
func (l *ConfigLoader) Env() string {
	return l.options.Env
}

// ConfigPath returns the effective configuration file path.
func (l *ConfigLoader) ConfigPath() string {
	return l.options.EffectiveConfigPath()
}

// LoadFromFile is a helper function to load configuration from a specific file.
func LoadFromFile(path string, target interface{}) error {
	if _, err := os.Stat(path); os.IsNotExist(err) {
		return NewConfigNotFoundError(path)
	}

	content, err := os.ReadFile(path)
	if err != nil {
		return NewConfigReadError(path, err)
	}

	if err := yaml.Unmarshal(content, target); err != nil {
		return NewConfigParseError(path, err)
	}

	return nil
}
