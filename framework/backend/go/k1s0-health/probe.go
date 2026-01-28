package k1s0health

import (
	"encoding/json"
	"net/http"
)

// ProbeHandler provides HTTP handlers for health probes.
type ProbeHandler struct {
	checker *Checker
}

// NewProbeHandler creates a new ProbeHandler.
func NewProbeHandler(checker *Checker) *ProbeHandler {
	return &ProbeHandler{
		checker: checker,
	}
}

// LivenessHandler returns an http.HandlerFunc for liveness probes.
func (p *ProbeHandler) LivenessHandler() http.HandlerFunc {
	return func(w http.ResponseWriter, r *http.Request) {
		response := p.checker.Liveness(r.Context())
		p.writeResponse(w, response)
	}
}

// ReadinessHandler returns an http.HandlerFunc for readiness probes.
func (p *ProbeHandler) ReadinessHandler() http.HandlerFunc {
	return func(w http.ResponseWriter, r *http.Request) {
		response := p.checker.Readiness(r.Context())
		p.writeResponse(w, response)
	}
}

// StartupHandler returns an http.HandlerFunc for startup probes.
func (p *ProbeHandler) StartupHandler() http.HandlerFunc {
	return func(w http.ResponseWriter, r *http.Request) {
		response := p.checker.Startup(r.Context())
		p.writeResponse(w, response)
	}
}

// HealthHandler returns an http.HandlerFunc for general health checks.
// This performs a full check of all components.
func (p *ProbeHandler) HealthHandler() http.HandlerFunc {
	return func(w http.ResponseWriter, r *http.Request) {
		response := p.checker.Check(r.Context())
		p.writeResponse(w, response)
	}
}

// ComponentHandler returns an http.HandlerFunc for checking a specific component.
func (p *ProbeHandler) ComponentHandler(componentName string) http.HandlerFunc {
	return func(w http.ResponseWriter, r *http.Request) {
		health := p.checker.CheckComponent(r.Context(), componentName)

		status := 200
		if !health.IsHealthy() {
			status = 503
		}

		w.Header().Set("Content-Type", "application/json")
		w.WriteHeader(status)
		json.NewEncoder(w).Encode(health)
	}
}

// writeResponse writes a HealthResponse as JSON.
func (p *ProbeHandler) writeResponse(w http.ResponseWriter, response *HealthResponse) {
	w.Header().Set("Content-Type", "application/json")
	w.WriteHeader(response.HTTPStatusCode())
	json.NewEncoder(w).Encode(response)
}

// RegisterHandlers registers all health handlers on an http.ServeMux.
func (p *ProbeHandler) RegisterHandlers(mux *http.ServeMux, basePath string) {
	if basePath == "" {
		basePath = "/healthz"
	}

	mux.HandleFunc(basePath+"/live", p.LivenessHandler())
	mux.HandleFunc(basePath+"/ready", p.ReadinessHandler())
	mux.HandleFunc(basePath+"/startup", p.StartupHandler())
	mux.HandleFunc(basePath, p.HealthHandler())
}

// Checker returns the underlying Checker.
func (p *ProbeHandler) Checker() *Checker {
	return p.checker
}
