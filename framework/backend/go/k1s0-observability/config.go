// Package k1s0observability provides unified observability (logging/tracing/metrics) for the k1s0 framework.
//
// This package enforces the following conventions:
//   - service.name and env are required fields
//   - JSON structured logging with mandatory fields
//   - OpenTelemetry integration for tracing and metrics
//
// # Required Log Fields
//
//	| Field         | Description                    |
//	|---------------|--------------------------------|
//	| timestamp     | ISO 8601 timestamp             |
//	| level         | Log level (DEBUG/INFO/WARN/ERROR) |
//	| message       | Log message                    |
//	| service.name  | Service name                   |
//	| service.env   | Environment (dev/stg/prod)     |
//	| trace.id      | Trace ID for request correlation |
//	| request.id    | Request ID                     |
//
// # Usage
//
//	config, err := k1s0observability.NewConfigBuilder().
//	    ServiceName("user-service").
//	    Env("dev").
//	    Build()
//
//	logger, err := k1s0observability.NewLogger(config)
//	logger.Info(ctx, "User created", zap.String("user_id", id))
package k1s0observability

import "errors"

// ConfigError represents a configuration error.
type ConfigError struct {
	Message string
}

func (e *ConfigError) Error() string {
	return e.Message
}

var (
	// ErrMissingServiceName is returned when service_name is not set.
	ErrMissingServiceName = &ConfigError{Message: "service_name is required"}
	// ErrMissingEnv is returned when env is not set.
	ErrMissingEnv = &ConfigError{Message: "env is required"}
	// ErrInvalidEnv is returned when env is not valid.
	ErrInvalidEnv = &ConfigError{Message: "env must be one of: dev, stg, prod"}
)

// ValidEnvs contains the allowed environment names.
var ValidEnvs = []string{"dev", "stg", "prod"}

// IsValidEnv checks if the given environment name is valid.
func IsValidEnv(env string) bool {
	for _, valid := range ValidEnvs {
		if env == valid {
			return true
		}
	}
	return false
}

// Config holds observability configuration.
type Config struct {
	// ServiceName is the name of the service (required).
	ServiceName string

	// Env is the environment name: dev, stg, prod (required).
	Env string

	// Version is the service version (optional).
	Version string

	// InstanceID is the service instance identifier (optional).
	InstanceID string

	// OTelEndpoint is the OpenTelemetry collector endpoint (optional).
	OTelEndpoint string

	// LogLevel is the minimum log level (default: INFO).
	LogLevel string

	// SamplingRate is the trace sampling rate 0.0-1.0 (default: 1.0).
	SamplingRate float64
}

// NewRequestContext creates a new RequestContext.
func (c *Config) NewRequestContext() *RequestContext {
	return NewRequestContext()
}

// NewRequestContextWithTrace creates a new RequestContext with a specific trace ID.
func (c *Config) NewRequestContextWithTrace(traceID string) *RequestContext {
	return NewRequestContextWithTraceID(traceID)
}

// IsProduction returns true if the environment is prod.
func (c *Config) IsProduction() bool {
	return c.Env == "prod"
}

// ConfigBuilder builds an observability Config.
type ConfigBuilder struct {
	serviceName  string
	env          string
	version      string
	instanceID   string
	otelEndpoint string
	logLevel     string
	samplingRate float64
}

// NewConfigBuilder creates a new ConfigBuilder.
func NewConfigBuilder() *ConfigBuilder {
	return &ConfigBuilder{
		logLevel:     "INFO",
		samplingRate: 1.0,
	}
}

// ServiceName sets the service name (required).
func (b *ConfigBuilder) ServiceName(name string) *ConfigBuilder {
	b.serviceName = name
	return b
}

// Env sets the environment name (required).
func (b *ConfigBuilder) Env(env string) *ConfigBuilder {
	b.env = env
	return b
}

// Version sets the service version.
func (b *ConfigBuilder) Version(version string) *ConfigBuilder {
	b.version = version
	return b
}

// InstanceID sets the service instance ID.
func (b *ConfigBuilder) InstanceID(id string) *ConfigBuilder {
	b.instanceID = id
	return b
}

// OTelEndpoint sets the OpenTelemetry collector endpoint.
func (b *ConfigBuilder) OTelEndpoint(endpoint string) *ConfigBuilder {
	b.otelEndpoint = endpoint
	return b
}

// LogLevel sets the minimum log level.
func (b *ConfigBuilder) LogLevel(level string) *ConfigBuilder {
	b.logLevel = level
	return b
}

// SamplingRate sets the trace sampling rate (0.0-1.0).
func (b *ConfigBuilder) SamplingRate(rate float64) *ConfigBuilder {
	if rate < 0 {
		rate = 0
	}
	if rate > 1 {
		rate = 1
	}
	b.samplingRate = rate
	return b
}

// Build creates the Config.
// Returns an error if required fields are missing.
func (b *ConfigBuilder) Build() (*Config, error) {
	if b.serviceName == "" {
		return nil, ErrMissingServiceName
	}
	if b.env == "" {
		return nil, ErrMissingEnv
	}
	if !IsValidEnv(b.env) {
		return nil, errors.New("env must be one of: dev, stg, prod, got: " + b.env)
	}

	return &Config{
		ServiceName:  b.serviceName,
		Env:          b.env,
		Version:      b.version,
		InstanceID:   b.instanceID,
		OTelEndpoint: b.otelEndpoint,
		LogLevel:     b.logLevel,
		SamplingRate: b.samplingRate,
	}, nil
}
