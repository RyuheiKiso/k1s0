package k1s0resilience

import (
	"errors"
	"sync"

	"github.com/sony/gobreaker"
)

// State represents the state of the circuit breaker.
type State int

const (
	// StateClosed indicates the circuit is closed (allowing requests).
	StateClosed State = iota

	// StateHalfOpen indicates the circuit is half-open (testing if the service recovered).
	StateHalfOpen

	// StateOpen indicates the circuit is open (blocking requests).
	StateOpen
)

// String returns the string representation of the state.
func (s State) String() string {
	switch s {
	case StateClosed:
		return "closed"
	case StateHalfOpen:
		return "half-open"
	case StateOpen:
		return "open"
	default:
		return "unknown"
	}
}

// fromGobreakerState converts a gobreaker.State to State.
func fromGobreakerState(s gobreaker.State) State {
	switch s {
	case gobreaker.StateClosed:
		return StateClosed
	case gobreaker.StateHalfOpen:
		return StateHalfOpen
	case gobreaker.StateOpen:
		return StateOpen
	default:
		return StateClosed
	}
}

// CircuitBreakerError represents an error when the circuit is open.
type CircuitBreakerError struct {
	Name  string
	State State
}

// Error implements the error interface.
func (e *CircuitBreakerError) Error() string {
	return "circuit breaker is " + e.State.String() + " for " + e.Name
}

// ErrCircuitOpen is returned when the circuit breaker is open.
var ErrCircuitOpen = errors.New("circuit breaker is open")

// CircuitBreaker provides circuit breaker functionality.
type CircuitBreaker struct {
	config *CircuitBreakerConfig
	cb     *gobreaker.CircuitBreaker
	mu     sync.RWMutex
}

// NewCircuitBreaker creates a new CircuitBreaker with the given configuration.
//
// Example:
//
//	config := k1s0resilience.DefaultCircuitBreakerConfig("user-service")
//	config.FailureThreshold = 10
//	config.Timeout = 30 * time.Second
//
//	cb := k1s0resilience.NewCircuitBreaker(config)
//
//	result, err := cb.Execute(func() (interface{}, error) {
//	    return userService.GetUser(id)
//	})
func NewCircuitBreaker(config *CircuitBreakerConfig) *CircuitBreaker {
	config = config.Validate()

	settings := gobreaker.Settings{
		Name:        config.Name,
		MaxRequests: config.MaxRequests,
		Interval:    config.Interval,
		Timeout:     config.Timeout,
		ReadyToTrip: createReadyToTripFunc(config),
	}

	if config.OnStateChange != nil {
		settings.OnStateChange = func(name string, from, to gobreaker.State) {
			config.OnStateChange(name, fromGobreakerState(from), fromGobreakerState(to))
		}
	}

	return &CircuitBreaker{
		config: config,
		cb:     gobreaker.NewCircuitBreaker(settings),
	}
}

// createReadyToTripFunc creates the ReadyToTrip function based on configuration.
func createReadyToTripFunc(config *CircuitBreakerConfig) func(counts gobreaker.Counts) bool {
	if config.FailureRatio > 0 {
		// Use failure ratio
		return func(counts gobreaker.Counts) bool {
			if counts.Requests < config.MinRequestsForRatio {
				return false
			}
			ratio := float64(counts.TotalFailures) / float64(counts.Requests)
			return ratio >= config.FailureRatio
		}
	}

	// Use failure threshold
	return func(counts gobreaker.Counts) bool {
		return counts.ConsecutiveFailures >= config.FailureThreshold
	}
}

// Execute runs the given function through the circuit breaker.
// If the circuit is open, it returns a CircuitBreakerError immediately.
//
// Example:
//
//	result, err := cb.Execute(func() (interface{}, error) {
//	    return httpClient.Get(url)
//	})
func (cb *CircuitBreaker) Execute(fn func() (interface{}, error)) (interface{}, error) {
	result, err := cb.cb.Execute(fn)
	if err != nil {
		if errors.Is(err, gobreaker.ErrOpenState) || errors.Is(err, gobreaker.ErrTooManyRequests) {
			return nil, &CircuitBreakerError{
				Name:  cb.config.Name,
				State: cb.State(),
			}
		}
		return nil, err
	}
	return result, nil
}

// ExecuteTyped runs the given function through the circuit breaker with type safety.
// This is a generic version of Execute that preserves the return type.
//
// Example:
//
//	user, err := k1s0resilience.ExecuteTyped(cb, func() (*User, error) {
//	    return userRepo.FindByID(id)
//	})
func ExecuteTyped[T any](cb *CircuitBreaker, fn func() (T, error)) (T, error) {
	var zero T
	result, err := cb.Execute(func() (interface{}, error) {
		return fn()
	})
	if err != nil {
		return zero, err
	}
	if result == nil {
		return zero, nil
	}
	return result.(T), nil
}

// State returns the current state of the circuit breaker.
func (cb *CircuitBreaker) State() State {
	return fromGobreakerState(cb.cb.State())
}

// Name returns the name of the circuit breaker.
func (cb *CircuitBreaker) Name() string {
	return cb.config.Name
}

// Counts returns the current counts of the circuit breaker.
func (cb *CircuitBreaker) Counts() Counts {
	counts := cb.cb.Counts()
	return Counts{
		Requests:             counts.Requests,
		TotalSuccesses:       counts.TotalSuccesses,
		TotalFailures:        counts.TotalFailures,
		ConsecutiveSuccesses: counts.ConsecutiveSuccesses,
		ConsecutiveFailures:  counts.ConsecutiveFailures,
	}
}

// Counts holds the counts of the circuit breaker.
type Counts struct {
	Requests             uint32
	TotalSuccesses       uint32
	TotalFailures        uint32
	ConsecutiveSuccesses uint32
	ConsecutiveFailures  uint32
}

// FailureRatio returns the failure ratio (0.0 to 1.0).
func (c Counts) FailureRatio() float64 {
	if c.Requests == 0 {
		return 0
	}
	return float64(c.TotalFailures) / float64(c.Requests)
}

// IsCircuitBreakerError checks if the error is a CircuitBreakerError.
func IsCircuitBreakerError(err error) bool {
	var cbErr *CircuitBreakerError
	return errors.As(err, &cbErr)
}

// CircuitBreakerGroup manages multiple circuit breakers.
type CircuitBreakerGroup struct {
	breakers sync.Map
	factory  func(name string) *CircuitBreakerConfig
}

// NewCircuitBreakerGroup creates a new CircuitBreakerGroup.
// The factory function is used to create configurations for new circuit breakers.
func NewCircuitBreakerGroup(factory func(name string) *CircuitBreakerConfig) *CircuitBreakerGroup {
	return &CircuitBreakerGroup{
		factory: factory,
	}
}

// Get returns the circuit breaker for the given name.
// If the circuit breaker doesn't exist, it creates a new one.
func (g *CircuitBreakerGroup) Get(name string) *CircuitBreaker {
	if cb, ok := g.breakers.Load(name); ok {
		return cb.(*CircuitBreaker)
	}

	config := g.factory(name)
	newCB := NewCircuitBreaker(config)

	actual, _ := g.breakers.LoadOrStore(name, newCB)
	return actual.(*CircuitBreaker)
}

// Execute runs the given function through the circuit breaker for the given name.
func (g *CircuitBreakerGroup) Execute(name string, fn func() (interface{}, error)) (interface{}, error) {
	return g.Get(name).Execute(fn)
}

// States returns the current states of all circuit breakers.
func (g *CircuitBreakerGroup) States() map[string]State {
	states := make(map[string]State)
	g.breakers.Range(func(key, value interface{}) bool {
		states[key.(string)] = value.(*CircuitBreaker).State()
		return true
	})
	return states
}
