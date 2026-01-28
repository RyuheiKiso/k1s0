package k1s0health

import (
	"context"
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"testing"
	"time"
)

func TestComponentHealthy(t *testing.T) {
	h := Healthy("database")

	if h.Name != "database" {
		t.Errorf("expected 'database', got '%s'", h.Name)
	}
	if h.Status != StatusHealthy {
		t.Errorf("expected healthy, got %s", h.Status)
	}
	if !h.IsHealthy() {
		t.Error("expected IsHealthy to be true")
	}
}

func TestComponentUnhealthy(t *testing.T) {
	h := Unhealthy("database", "connection failed")

	if h.Status != StatusUnhealthy {
		t.Errorf("expected unhealthy, got %s", h.Status)
	}
	if h.Message != "connection failed" {
		t.Errorf("expected 'connection failed', got '%s'", h.Message)
	}
	if h.IsHealthy() {
		t.Error("expected IsHealthy to be false")
	}
}

func TestComponentDegraded(t *testing.T) {
	h := Degraded("cache", "high latency")

	if h.Status != StatusDegraded {
		t.Errorf("expected degraded, got %s", h.Status)
	}
}

func TestComponentHealthWithExtra(t *testing.T) {
	h := Healthy("database").
		WithMessage("connected").
		WithDuration(100 * time.Millisecond).
		WithExtra("connections", 10)

	if h.Message != "connected" {
		t.Errorf("expected 'connected', got '%s'", h.Message)
	}
	if h.Duration != 100*time.Millisecond {
		t.Errorf("expected 100ms, got %v", h.Duration)
	}
	if h.Extra["connections"] != 10 {
		t.Errorf("expected 10, got %v", h.Extra["connections"])
	}
}

func TestHealthResponse(t *testing.T) {
	r := HealthyResponse().
		WithServiceName("user-service").
		WithVersion("1.0.0")

	if r.Status != StatusHealthy {
		t.Errorf("expected healthy, got %s", r.Status)
	}
	if r.ServiceName != "user-service" {
		t.Errorf("expected 'user-service', got '%s'", r.ServiceName)
	}
	if r.Version != "1.0.0" {
		t.Errorf("expected '1.0.0', got '%s'", r.Version)
	}
}

func TestHealthResponseWithComponents(t *testing.T) {
	r := HealthyResponse().
		AddComponent(Healthy("database")).
		AddComponent(Healthy("cache"))

	if len(r.Components) != 2 {
		t.Errorf("expected 2 components, got %d", len(r.Components))
	}
	if r.Status != StatusHealthy {
		t.Errorf("expected healthy, got %s", r.Status)
	}
}

func TestHealthResponseUnhealthyComponent(t *testing.T) {
	r := HealthyResponse().
		AddComponent(Healthy("cache")).
		AddComponent(Unhealthy("database", "connection failed"))

	if r.Status != StatusUnhealthy {
		t.Errorf("expected unhealthy, got %s", r.Status)
	}
}

func TestHealthResponseDegradedComponent(t *testing.T) {
	r := HealthyResponse().
		AddComponent(Healthy("cache")).
		AddComponent(Degraded("database", "slow"))

	if r.Status != StatusDegraded {
		t.Errorf("expected degraded, got %s", r.Status)
	}
}

func TestHealthResponseHTTPStatusCode(t *testing.T) {
	tests := []struct {
		status   Status
		expected int
	}{
		{StatusHealthy, 200},
		{StatusDegraded, 200},
		{StatusUnhealthy, 503},
		{StatusUnknown, 503},
	}

	for _, tt := range tests {
		t.Run(string(tt.status), func(t *testing.T) {
			r := NewHealthResponse(tt.status)
			if r.HTTPStatusCode() != tt.expected {
				t.Errorf("expected %d, got %d", tt.expected, r.HTTPStatusCode())
			}
		})
	}
}

func TestChecker(t *testing.T) {
	checker := NewChecker().
		WithServiceName("test-service").
		WithVersion("1.0.0").
		AddComponent("database", func(ctx context.Context) *ComponentHealth {
			return Healthy("database")
		}).
		AddComponent("cache", func(ctx context.Context) *ComponentHealth {
			return Healthy("cache")
		})

	response := checker.Check(context.Background())

	if response.Status != StatusHealthy {
		t.Errorf("expected healthy, got %s", response.Status)
	}
	if len(response.Components) != 2 {
		t.Errorf("expected 2 components, got %d", len(response.Components))
	}
}

func TestCheckerWithUnhealthyComponent(t *testing.T) {
	checker := NewChecker().
		AddComponent("database", func(ctx context.Context) *ComponentHealth {
			return Unhealthy("database", "connection failed")
		})

	response := checker.Check(context.Background())

	if response.Status != StatusUnhealthy {
		t.Errorf("expected unhealthy, got %s", response.Status)
	}
}

func TestCheckerLiveness(t *testing.T) {
	checker := NewChecker().
		AddComponent("database", func(ctx context.Context) *ComponentHealth {
			return Unhealthy("database", "connection failed")
		})

	// Liveness should always return healthy (the app is running)
	response := checker.Liveness(context.Background())

	if response.Status != StatusHealthy {
		t.Errorf("expected healthy for liveness, got %s", response.Status)
	}
}

func TestCheckerReadiness(t *testing.T) {
	checker := NewChecker().
		AddComponent("database", func(ctx context.Context) *ComponentHealth {
			return Healthy("database")
		})

	response := checker.Readiness(context.Background())

	if response.Status != StatusHealthy {
		t.Errorf("expected healthy, got %s", response.Status)
	}
}

func TestCheckerStartup(t *testing.T) {
	checker := NewChecker()

	// Before SetStartupReady
	response := checker.Startup(context.Background())
	if response.Status != StatusUnhealthy {
		t.Errorf("expected unhealthy before startup ready, got %s", response.Status)
	}

	// After SetStartupReady
	checker.SetStartupReady()
	response = checker.Startup(context.Background())
	if response.Status != StatusHealthy {
		t.Errorf("expected healthy after startup ready, got %s", response.Status)
	}
}

func TestCheckerCheckComponent(t *testing.T) {
	checker := NewChecker().
		AddComponent("database", func(ctx context.Context) *ComponentHealth {
			return Healthy("database")
		})

	// Check existing component
	health := checker.CheckComponent(context.Background(), "database")
	if health.Status != StatusHealthy {
		t.Errorf("expected healthy, got %s", health.Status)
	}

	// Check non-existing component
	health = checker.CheckComponent(context.Background(), "nonexistent")
	if health.Status != StatusUnhealthy {
		t.Errorf("expected unhealthy for nonexistent, got %s", health.Status)
	}
}

func TestCheckerRemoveComponent(t *testing.T) {
	checker := NewChecker().
		AddComponent("database", func(ctx context.Context) *ComponentHealth {
			return Healthy("database")
		})

	checker.RemoveComponent("database")

	names := checker.ComponentNames()
	if len(names) != 0 {
		t.Errorf("expected 0 components, got %d", len(names))
	}
}

func TestProbeHandlerLiveness(t *testing.T) {
	checker := NewChecker()
	handler := NewProbeHandler(checker)

	req := httptest.NewRequest(http.MethodGet, "/healthz/live", nil)
	w := httptest.NewRecorder()

	handler.LivenessHandler()(w, req)

	if w.Code != 200 {
		t.Errorf("expected 200, got %d", w.Code)
	}

	var response HealthResponse
	if err := json.NewDecoder(w.Body).Decode(&response); err != nil {
		t.Fatalf("failed to decode response: %v", err)
	}

	if response.Status != StatusHealthy {
		t.Errorf("expected healthy, got %s", response.Status)
	}
}

func TestProbeHandlerReadiness(t *testing.T) {
	checker := NewChecker().
		AddComponent("database", func(ctx context.Context) *ComponentHealth {
			return Healthy("database")
		})
	handler := NewProbeHandler(checker)

	req := httptest.NewRequest(http.MethodGet, "/healthz/ready", nil)
	w := httptest.NewRecorder()

	handler.ReadinessHandler()(w, req)

	if w.Code != 200 {
		t.Errorf("expected 200, got %d", w.Code)
	}
}

func TestProbeHandlerReadinessUnhealthy(t *testing.T) {
	checker := NewChecker().
		AddComponent("database", func(ctx context.Context) *ComponentHealth {
			return Unhealthy("database", "connection failed")
		})
	handler := NewProbeHandler(checker)

	req := httptest.NewRequest(http.MethodGet, "/healthz/ready", nil)
	w := httptest.NewRecorder()

	handler.ReadinessHandler()(w, req)

	if w.Code != 503 {
		t.Errorf("expected 503, got %d", w.Code)
	}
}

func TestProbeHandlerStartup(t *testing.T) {
	checker := NewChecker()
	handler := NewProbeHandler(checker)

	// Before startup ready
	req := httptest.NewRequest(http.MethodGet, "/healthz/startup", nil)
	w := httptest.NewRecorder()
	handler.StartupHandler()(w, req)

	if w.Code != 503 {
		t.Errorf("expected 503 before startup ready, got %d", w.Code)
	}

	// After startup ready
	checker.SetStartupReady()
	w = httptest.NewRecorder()
	handler.StartupHandler()(w, req)

	if w.Code != 200 {
		t.Errorf("expected 200 after startup ready, got %d", w.Code)
	}
}

func TestProbeHandlerHealth(t *testing.T) {
	checker := NewChecker().
		WithServiceName("test-service").
		AddComponent("database", func(ctx context.Context) *ComponentHealth {
			return Healthy("database")
		})
	handler := NewProbeHandler(checker)

	req := httptest.NewRequest(http.MethodGet, "/healthz", nil)
	w := httptest.NewRecorder()

	handler.HealthHandler()(w, req)

	if w.Code != 200 {
		t.Errorf("expected 200, got %d", w.Code)
	}

	var response HealthResponse
	if err := json.NewDecoder(w.Body).Decode(&response); err != nil {
		t.Fatalf("failed to decode response: %v", err)
	}

	if response.ServiceName != "test-service" {
		t.Errorf("expected 'test-service', got '%s'", response.ServiceName)
	}
}

func TestProbeHandlerComponent(t *testing.T) {
	checker := NewChecker().
		AddComponent("database", func(ctx context.Context) *ComponentHealth {
			return Healthy("database")
		})
	handler := NewProbeHandler(checker)

	req := httptest.NewRequest(http.MethodGet, "/healthz/component/database", nil)
	w := httptest.NewRecorder()

	handler.ComponentHandler("database")(w, req)

	if w.Code != 200 {
		t.Errorf("expected 200, got %d", w.Code)
	}
}

func TestProbeHandlerRegisterHandlers(t *testing.T) {
	checker := NewChecker()
	handler := NewProbeHandler(checker)

	mux := http.NewServeMux()
	handler.RegisterHandlers(mux, "/healthz")

	// Test each registered endpoint
	endpoints := []string{"/healthz", "/healthz/live", "/healthz/ready", "/healthz/startup"}
	for _, endpoint := range endpoints {
		req := httptest.NewRequest(http.MethodGet, endpoint, nil)
		w := httptest.NewRecorder()
		mux.ServeHTTP(w, req)

		// Should not return 404
		if w.Code == 404 {
			t.Errorf("endpoint %s not registered", endpoint)
		}
	}
}

func TestCheckerTimeout(t *testing.T) {
	checker := NewChecker().
		WithTimeout(10 * time.Millisecond).
		AddComponent("slow", func(ctx context.Context) *ComponentHealth {
			select {
			case <-ctx.Done():
				return Unhealthy("slow", "timeout")
			case <-time.After(100 * time.Millisecond):
				return Healthy("slow")
			}
		})

	response := checker.Check(context.Background())

	// Should complete quickly due to timeout
	found := false
	for _, c := range response.Components {
		if c.Name == "slow" {
			found = true
			if c.Status != StatusUnhealthy {
				t.Errorf("expected unhealthy due to timeout, got %s", c.Status)
			}
		}
	}

	if !found {
		t.Error("expected slow component in response")
	}
}

func TestCheckerEmptyComponents(t *testing.T) {
	checker := NewChecker().
		WithServiceName("test-service")

	response := checker.Check(context.Background())

	if response.Status != StatusHealthy {
		t.Errorf("expected healthy with no components, got %s", response.Status)
	}
}

func TestHealthResponseWithComponentsSlice(t *testing.T) {
	components := []*ComponentHealth{
		Healthy("a"),
		Degraded("b", "slow"),
	}

	r := HealthyResponse().WithComponents(components)

	if r.Status != StatusDegraded {
		t.Errorf("expected degraded, got %s", r.Status)
	}
}

func TestHealthResponseWithUnhealthyComponentsSlice(t *testing.T) {
	components := []*ComponentHealth{
		Healthy("a"),
		Unhealthy("b", "failed"),
	}

	r := HealthyResponse().WithComponents(components)

	if r.Status != StatusUnhealthy {
		t.Errorf("expected unhealthy, got %s", r.Status)
	}
}
