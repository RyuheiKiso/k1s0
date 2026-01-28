// Package k1s0health provides Kubernetes health check support for the k1s0 framework.
//
// This package implements three types of health probes:
//   - Liveness: Indicates if the application is running
//   - Readiness: Indicates if the application is ready to receive traffic
//   - Startup: Indicates if the application has started successfully
//
// # Kubernetes Integration
//
// Configure your Pod with these probes:
//
//	livenessProbe:
//	  httpGet:
//	    path: /healthz/live
//	    port: 8080
//	  initialDelaySeconds: 3
//	  periodSeconds: 3
//
//	readinessProbe:
//	  httpGet:
//	    path: /healthz/ready
//	    port: 8080
//	  initialDelaySeconds: 5
//	  periodSeconds: 5
//
//	startupProbe:
//	  httpGet:
//	    path: /healthz/startup
//	    port: 8080
//	  failureThreshold: 30
//	  periodSeconds: 10
//
// # Usage
//
//	checker := k1s0health.NewChecker().
//	    AddComponent("database", dbHealthCheck).
//	    AddComponent("cache", cacheHealthCheck)
//
//	http.HandleFunc("/healthz/live", checker.LivenessHandler())
//	http.HandleFunc("/healthz/ready", checker.ReadinessHandler())
//	http.HandleFunc("/healthz/startup", checker.StartupHandler())
package k1s0health

import "time"

// Status represents the health status of a component.
type Status string

const (
	// StatusHealthy indicates the component is healthy.
	StatusHealthy Status = "healthy"
	// StatusUnhealthy indicates the component is unhealthy.
	StatusUnhealthy Status = "unhealthy"
	// StatusDegraded indicates the component is degraded but functional.
	StatusDegraded Status = "degraded"
	// StatusUnknown indicates the health status is unknown.
	StatusUnknown Status = "unknown"
)

// ComponentHealth represents the health status of a single component.
type ComponentHealth struct {
	// Name is the component name.
	Name string `json:"name"`

	// Status is the health status.
	Status Status `json:"status"`

	// Message is an optional description of the health status.
	Message string `json:"message,omitempty"`

	// Duration is how long the health check took.
	Duration time.Duration `json:"duration_ms,omitempty"`

	// LastChecked is when the health was last checked.
	LastChecked time.Time `json:"last_checked,omitempty"`

	// Extra holds additional component-specific information.
	Extra map[string]interface{} `json:"extra,omitempty"`
}

// NewComponentHealth creates a new ComponentHealth.
func NewComponentHealth(name string, status Status) *ComponentHealth {
	return &ComponentHealth{
		Name:        name,
		Status:      status,
		LastChecked: time.Now(),
	}
}

// Healthy creates a healthy ComponentHealth.
func Healthy(name string) *ComponentHealth {
	return NewComponentHealth(name, StatusHealthy)
}

// Unhealthy creates an unhealthy ComponentHealth.
func Unhealthy(name, message string) *ComponentHealth {
	h := NewComponentHealth(name, StatusUnhealthy)
	h.Message = message
	return h
}

// Degraded creates a degraded ComponentHealth.
func Degraded(name, message string) *ComponentHealth {
	h := NewComponentHealth(name, StatusDegraded)
	h.Message = message
	return h
}

// WithMessage sets the message.
func (h *ComponentHealth) WithMessage(message string) *ComponentHealth {
	h.Message = message
	return h
}

// WithDuration sets the duration.
func (h *ComponentHealth) WithDuration(d time.Duration) *ComponentHealth {
	h.Duration = d
	return h
}

// WithExtra adds extra information.
func (h *ComponentHealth) WithExtra(key string, value interface{}) *ComponentHealth {
	if h.Extra == nil {
		h.Extra = make(map[string]interface{})
	}
	h.Extra[key] = value
	return h
}

// IsHealthy returns true if the status is healthy.
func (h *ComponentHealth) IsHealthy() bool {
	return h.Status == StatusHealthy
}

// HealthResponse represents the overall health response.
type HealthResponse struct {
	// Status is the overall health status.
	Status Status `json:"status"`

	// Timestamp is when the health check was performed.
	Timestamp time.Time `json:"timestamp"`

	// Version is the service version (optional).
	Version string `json:"version,omitempty"`

	// ServiceName is the name of the service.
	ServiceName string `json:"service_name,omitempty"`

	// Components contains health status of individual components.
	Components []*ComponentHealth `json:"components,omitempty"`

	// Message is an optional overall message.
	Message string `json:"message,omitempty"`
}

// NewHealthResponse creates a new HealthResponse.
func NewHealthResponse(status Status) *HealthResponse {
	return &HealthResponse{
		Status:    status,
		Timestamp: time.Now(),
	}
}

// HealthyResponse creates a healthy HealthResponse.
func HealthyResponse() *HealthResponse {
	return NewHealthResponse(StatusHealthy)
}

// UnhealthyResponse creates an unhealthy HealthResponse.
func UnhealthyResponse(message string) *HealthResponse {
	r := NewHealthResponse(StatusUnhealthy)
	r.Message = message
	return r
}

// WithVersion sets the service version.
func (r *HealthResponse) WithVersion(version string) *HealthResponse {
	r.Version = version
	return r
}

// WithServiceName sets the service name.
func (r *HealthResponse) WithServiceName(name string) *HealthResponse {
	r.ServiceName = name
	return r
}

// WithComponents sets the component health statuses.
func (r *HealthResponse) WithComponents(components []*ComponentHealth) *HealthResponse {
	r.Components = components

	// Update overall status based on components
	hasUnhealthy := false
	hasDegraded := false

	for _, c := range components {
		if c.Status == StatusUnhealthy {
			hasUnhealthy = true
		} else if c.Status == StatusDegraded {
			hasDegraded = true
		}
	}

	if hasUnhealthy {
		r.Status = StatusUnhealthy
	} else if hasDegraded {
		r.Status = StatusDegraded
	}

	return r
}

// AddComponent adds a component health status.
func (r *HealthResponse) AddComponent(component *ComponentHealth) *HealthResponse {
	r.Components = append(r.Components, component)

	// Update overall status
	if component.Status == StatusUnhealthy {
		r.Status = StatusUnhealthy
	} else if component.Status == StatusDegraded && r.Status == StatusHealthy {
		r.Status = StatusDegraded
	}

	return r
}

// IsHealthy returns true if the overall status is healthy.
func (r *HealthResponse) IsHealthy() bool {
	return r.Status == StatusHealthy
}

// HTTPStatusCode returns the appropriate HTTP status code.
func (r *HealthResponse) HTTPStatusCode() int {
	switch r.Status {
	case StatusHealthy:
		return 200
	case StatusDegraded:
		return 200 // Degraded is still considered passing for health checks
	default:
		return 503
	}
}
