package k1s0health

import (
	"context"
	"sync"
	"time"
)

// CheckFunc is a function that performs a health check.
// It should return a ComponentHealth with the result.
type CheckFunc func(ctx context.Context) *ComponentHealth

// Checker performs health checks on registered components.
type Checker struct {
	mu           sync.RWMutex
	components   map[string]CheckFunc
	serviceName  string
	version      string
	timeout      time.Duration
	startupReady bool
}

// NewChecker creates a new Checker.
func NewChecker() *Checker {
	return &Checker{
		components: make(map[string]CheckFunc),
		timeout:    5 * time.Second,
	}
}

// WithServiceName sets the service name for health responses.
func (c *Checker) WithServiceName(name string) *Checker {
	c.serviceName = name
	return c
}

// WithVersion sets the version for health responses.
func (c *Checker) WithVersion(version string) *Checker {
	c.version = version
	return c
}

// WithTimeout sets the timeout for health checks.
func (c *Checker) WithTimeout(timeout time.Duration) *Checker {
	c.timeout = timeout
	return c
}

// AddComponent registers a component health check.
func (c *Checker) AddComponent(name string, check CheckFunc) *Checker {
	c.mu.Lock()
	defer c.mu.Unlock()
	c.components[name] = check
	return c
}

// RemoveComponent unregisters a component health check.
func (c *Checker) RemoveComponent(name string) *Checker {
	c.mu.Lock()
	defer c.mu.Unlock()
	delete(c.components, name)
	return c
}

// SetStartupReady marks the service as ready for startup probe.
func (c *Checker) SetStartupReady() {
	c.mu.Lock()
	defer c.mu.Unlock()
	c.startupReady = true
}

// IsStartupReady returns true if the service is ready.
func (c *Checker) IsStartupReady() bool {
	c.mu.RLock()
	defer c.mu.RUnlock()
	return c.startupReady
}

// Check performs health checks on all registered components.
func (c *Checker) Check(ctx context.Context) *HealthResponse {
	c.mu.RLock()
	components := make(map[string]CheckFunc, len(c.components))
	for name, check := range c.components {
		components[name] = check
	}
	c.mu.RUnlock()

	response := HealthyResponse().
		WithServiceName(c.serviceName).
		WithVersion(c.version)

	if len(components) == 0 {
		return response
	}

	// Run all checks in parallel
	var wg sync.WaitGroup
	results := make(chan *ComponentHealth, len(components))

	for name, check := range components {
		wg.Add(1)
		go func(name string, check CheckFunc) {
			defer wg.Done()

			checkCtx, cancel := context.WithTimeout(ctx, c.timeout)
			defer cancel()

			start := time.Now()
			health := check(checkCtx)
			if health == nil {
				health = Healthy(name)
			}
			health.Name = name
			health.Duration = time.Since(start)
			health.LastChecked = start

			results <- health
		}(name, check)
	}

	// Close results channel when all checks complete
	go func() {
		wg.Wait()
		close(results)
	}()

	// Collect results
	for health := range results {
		response.AddComponent(health)
	}

	return response
}

// CheckComponent performs a health check on a single component.
func (c *Checker) CheckComponent(ctx context.Context, name string) *ComponentHealth {
	c.mu.RLock()
	check, exists := c.components[name]
	c.mu.RUnlock()

	if !exists {
		return Unhealthy(name, "component not found")
	}

	checkCtx, cancel := context.WithTimeout(ctx, c.timeout)
	defer cancel()

	start := time.Now()
	health := check(checkCtx)
	if health == nil {
		health = Healthy(name)
	}
	health.Name = name
	health.Duration = time.Since(start)
	health.LastChecked = start

	return health
}

// Liveness returns the liveness status.
// Liveness indicates if the application is running.
// A failure means the application should be restarted.
func (c *Checker) Liveness(ctx context.Context) *HealthResponse {
	// Liveness should be a simple check - just verify the app is running
	return HealthyResponse().
		WithServiceName(c.serviceName).
		WithVersion(c.version)
}

// Readiness returns the readiness status.
// Readiness indicates if the application is ready to receive traffic.
// It checks all registered components.
func (c *Checker) Readiness(ctx context.Context) *HealthResponse {
	return c.Check(ctx)
}

// Startup returns the startup status.
// Startup indicates if the application has completed its startup sequence.
func (c *Checker) Startup(ctx context.Context) *HealthResponse {
	c.mu.RLock()
	ready := c.startupReady
	c.mu.RUnlock()

	response := NewHealthResponse(StatusHealthy).
		WithServiceName(c.serviceName).
		WithVersion(c.version)

	if !ready {
		response.Status = StatusUnhealthy
		response.Message = "startup not complete"
	}

	return response
}

// ComponentNames returns the names of all registered components.
func (c *Checker) ComponentNames() []string {
	c.mu.RLock()
	defer c.mu.RUnlock()

	names := make([]string, 0, len(c.components))
	for name := range c.components {
		names = append(names, name)
	}
	return names
}
