// Package k1s0config provides YAML configuration loading for the k1s0 framework.
//
// This package enforces the following k1s0 conventions:
//   - No environment variable usage (os.Getenv is prohibited)
//   - Secrets must be referenced via *_file suffix keys
//   - Configuration files are YAML format only
//   - Environment names are restricted to: default, dev, stg, prod
//
// # Usage
//
//	options := k1s0config.NewConfigOptions("dev").
//	    WithConfigPath("config/dev.yaml").
//	    WithSecretsDir("/var/run/secrets/k1s0")
//
//	loader, err := k1s0config.NewConfigLoader(options)
//	if err != nil {
//	    log.Fatal(err)
//	}
//
//	var config AppConfig
//	if err := loader.Load(&config); err != nil {
//	    log.Fatal(err)
//	}
//
//	// Resolve secrets referenced by *_file keys
//	password, err := loader.ResolveSecretFile(config.DB.PasswordFile, "db.password_file")
package k1s0config

import (
	"errors"
	"fmt"
)

// ConfigError represents a configuration error.
type ConfigError struct {
	Code    string
	Message string
	Path    string
	Key     string
	Hint    string
	Wrapped error
}

// Error implements the error interface.
func (e *ConfigError) Error() string {
	if e.Path != "" {
		return fmt.Sprintf("%s: %s (path: %s)", e.Code, e.Message, e.Path)
	}
	if e.Key != "" {
		return fmt.Sprintf("%s: %s (key: %s)", e.Code, e.Message, e.Key)
	}
	return fmt.Sprintf("%s: %s", e.Code, e.Message)
}

// Unwrap returns the wrapped error.
func (e *ConfigError) Unwrap() error {
	return e.Wrapped
}

// GetHint returns the hint for resolving the error.
func (e *ConfigError) GetHint() string {
	return e.Hint
}

// Error codes
const (
	ErrCodeInvalidEnv         = "CONFIG_INVALID_ENV"
	ErrCodeConfigNotFound     = "CONFIG_NOT_FOUND"
	ErrCodeConfigReadError    = "CONFIG_READ_ERROR"
	ErrCodeConfigParseError   = "CONFIG_PARSE_ERROR"
	ErrCodeSecretsDirNotFound = "SECRETS_DIR_NOT_FOUND"
	ErrCodeSecretNotFound     = "SECRET_NOT_FOUND"
	ErrCodeSecretReadError    = "SECRET_READ_ERROR"
)

// NewInvalidEnvError creates an error for invalid environment name.
func NewInvalidEnvError(env string) *ConfigError {
	return &ConfigError{
		Code:    ErrCodeInvalidEnv,
		Message: fmt.Sprintf("invalid environment name: %s", env),
		Key:     env,
		Hint:    "Environment must be one of: default, dev, stg, prod",
	}
}

// NewConfigNotFoundError creates an error for missing config file.
func NewConfigNotFoundError(path string) *ConfigError {
	return &ConfigError{
		Code:    ErrCodeConfigNotFound,
		Message: "configuration file not found",
		Path:    path,
		Hint:    "Ensure the configuration file exists at the specified path",
	}
}

// NewConfigReadError creates an error for config file read failure.
func NewConfigReadError(path string, err error) *ConfigError {
	return &ConfigError{
		Code:    ErrCodeConfigReadError,
		Message: "failed to read configuration file",
		Path:    path,
		Wrapped: err,
	}
}

// NewConfigParseError creates an error for YAML parse failure.
func NewConfigParseError(path string, err error) *ConfigError {
	return &ConfigError{
		Code:    ErrCodeConfigParseError,
		Message: "failed to parse configuration file",
		Path:    path,
		Wrapped: err,
		Hint:    "Check the YAML syntax of the configuration file",
	}
}

// NewSecretsDirNotFoundError creates an error for missing secrets directory.
func NewSecretsDirNotFoundError(path string) *ConfigError {
	return &ConfigError{
		Code:    ErrCodeSecretsDirNotFound,
		Message: "secrets directory not found",
		Path:    path,
		Hint:    "Ensure the secrets directory exists. In Kubernetes, mount a Secret volume to this path.",
	}
}

// NewSecretNotFoundError creates an error for missing secret file.
func NewSecretNotFoundError(path, key string) *ConfigError {
	return &ConfigError{
		Code:    ErrCodeSecretNotFound,
		Message: fmt.Sprintf("secret file not found (referenced by %s)", key),
		Path:    path,
		Key:     key,
		Hint:    fmt.Sprintf("Ensure the secret file exists. In Kubernetes, add the key to the Secret and mount it to %s.", path),
	}
}

// NewSecretReadError creates an error for secret file read failure.
func NewSecretReadError(path, key string, err error) *ConfigError {
	return &ConfigError{
		Code:    ErrCodeSecretReadError,
		Message: fmt.Sprintf("failed to read secret file (referenced by %s)", key),
		Path:    path,
		Key:     key,
		Wrapped: err,
	}
}

// IsConfigNotFound returns true if the error is a config not found error.
func IsConfigNotFound(err error) bool {
	var configErr *ConfigError
	if errors.As(err, &configErr) {
		return configErr.Code == ErrCodeConfigNotFound
	}
	return false
}

// IsSecretNotFound returns true if the error is a secret not found error.
func IsSecretNotFound(err error) bool {
	var configErr *ConfigError
	if errors.As(err, &configErr) {
		return configErr.Code == ErrCodeSecretNotFound
	}
	return false
}
