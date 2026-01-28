package k1s0config

// ServiceArgs represents common CLI arguments for k1s0 services.
//
// These arguments follow k1s0 conventions:
//   - Configuration is loaded from YAML files only
//   - Environment must be explicitly specified
//   - Secrets are loaded from file references
type ServiceArgs struct {
	// Env is the environment name (required: dev, stg, prod).
	Env string

	// ConfigPath is the explicit path to the configuration file.
	// If empty, defaults to config/{env}.yaml.
	ConfigPath string

	// SecretsDir is the directory containing secret files.
	// Defaults to /var/run/secrets/k1s0.
	SecretsDir string
}

// NewServiceArgs creates ServiceArgs with default values.
func NewServiceArgs() *ServiceArgs {
	return &ServiceArgs{
		SecretsDir: "/var/run/secrets/k1s0",
	}
}

// ToConfigOptions converts ServiceArgs to ConfigOptions.
func (a *ServiceArgs) ToConfigOptions() *ConfigOptions {
	opts := NewConfigOptions(a.Env)
	if a.ConfigPath != "" {
		opts.WithConfigPath(a.ConfigPath)
	}
	if a.SecretsDir != "" {
		opts.WithSecretsDir(a.SecretsDir)
	}
	return opts
}

// Validate validates the service arguments.
func (a *ServiceArgs) Validate() error {
	if a.Env == "" {
		return &ConfigError{
			Code:    "ARGS_MISSING_ENV",
			Message: "environment name is required",
			Key:     "env",
			Hint:    "Specify the environment using --env flag (dev, stg, or prod)",
		}
	}
	if !IsValidEnv(a.Env) {
		return NewInvalidEnvError(a.Env)
	}
	return nil
}
