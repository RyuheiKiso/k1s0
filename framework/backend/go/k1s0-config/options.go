package k1s0config

import (
	"path/filepath"
)

// ValidEnvs contains the allowed environment names.
var ValidEnvs = []string{"default", "dev", "stg", "prod"}

// IsValidEnv checks if the given environment name is valid.
func IsValidEnv(env string) bool {
	for _, valid := range ValidEnvs {
		if env == valid {
			return true
		}
	}
	return false
}

// ConfigOptions holds configuration loading options.
type ConfigOptions struct {
	// Env is the environment name (default, dev, stg, prod).
	Env string

	// ConfigPath is the explicit path to the configuration file.
	// If empty, defaults to config/{env}.yaml.
	ConfigPath string

	// ConfigDir is the directory containing configuration files.
	// Used when ConfigPath is not set.
	ConfigDir string

	// SecretsDir is the directory containing secret files.
	SecretsDir string

	// RequireConfigFile controls whether the config file must exist.
	RequireConfigFile bool

	// RequireSecretsDir controls whether the secrets directory must exist.
	RequireSecretsDir bool
}

// NewConfigOptions creates new ConfigOptions with the given environment.
func NewConfigOptions(env string) *ConfigOptions {
	return &ConfigOptions{
		Env:               env,
		ConfigDir:         "config",
		SecretsDir:        "/var/run/secrets/k1s0",
		RequireConfigFile: true,
		RequireSecretsDir: false,
	}
}

// WithConfigPath sets the explicit configuration file path.
func (o *ConfigOptions) WithConfigPath(path string) *ConfigOptions {
	o.ConfigPath = path
	return o
}

// WithConfigDir sets the configuration directory.
func (o *ConfigOptions) WithConfigDir(dir string) *ConfigOptions {
	o.ConfigDir = dir
	return o
}

// WithSecretsDir sets the secrets directory.
func (o *ConfigOptions) WithSecretsDir(dir string) *ConfigOptions {
	o.SecretsDir = dir
	return o
}

// WithRequireConfigFile sets whether the config file must exist.
func (o *ConfigOptions) WithRequireConfigFile(require bool) *ConfigOptions {
	o.RequireConfigFile = require
	return o
}

// WithRequireSecretsDir sets whether the secrets directory must exist.
func (o *ConfigOptions) WithRequireSecretsDir(require bool) *ConfigOptions {
	o.RequireSecretsDir = require
	return o
}

// EffectiveConfigPath returns the effective configuration file path.
func (o *ConfigOptions) EffectiveConfigPath() string {
	if o.ConfigPath != "" {
		return o.ConfigPath
	}
	return filepath.Join(o.ConfigDir, o.Env+".yaml")
}

// IsValidEnv returns true if the configured environment is valid.
func (o *ConfigOptions) IsValidEnv() bool {
	return IsValidEnv(o.Env)
}
