package consensus

import (
	"context"
	"encoding/json"
	"fmt"
	"time"
)

// SagaStatus represents the current state of a saga instance.
type SagaStatus string

const (
	// SagaStatusPending indicates the saga has not started.
	SagaStatusPending SagaStatus = "pending"

	// SagaStatusRunning indicates the saga is executing steps.
	SagaStatusRunning SagaStatus = "running"

	// SagaStatusCompleted indicates all steps completed successfully.
	SagaStatusCompleted SagaStatus = "completed"

	// SagaStatusCompensating indicates compensation is in progress.
	SagaStatusCompensating SagaStatus = "compensating"

	// SagaStatusCompensated indicates compensation completed successfully.
	SagaStatusCompensated SagaStatus = "compensated"

	// SagaStatusFailed indicates the saga failed and compensation succeeded.
	SagaStatusFailed SagaStatus = "failed"

	// SagaStatusDeadLetter indicates compensation also failed.
	SagaStatusDeadLetter SagaStatus = "dead_letter"
)

// SagaStep defines a single step in a saga with execute and compensate logic.
// The type parameter C is the saga context type shared across steps.
type SagaStep[C any] interface {
	// Name returns the unique name of this step.
	Name() string

	// Execute performs the forward action. The returned json.RawMessage
	// is stored and passed to Compensate if rollback is needed.
	Execute(ctx context.Context, sagaCtx *C) (json.RawMessage, error)

	// Compensate undoes the forward action. It receives the output
	// from Execute to know what to undo.
	Compensate(ctx context.Context, sagaCtx *C, executeOutput json.RawMessage) error
}

// RetryPolicy defines how a step should be retried on failure.
type RetryPolicy struct {
	// MaxRetries is the maximum number of retry attempts.
	MaxRetries int `yaml:"max_retries"`

	// Backoff is the backoff strategy.
	Backoff BackoffStrategy `yaml:"backoff"`

	// InitialInterval is the initial wait between retries.
	InitialInterval time.Duration `yaml:"initial_interval"`

	// MaxInterval is the maximum wait between retries.
	MaxInterval time.Duration `yaml:"max_interval"`

	// Multiplier is the backoff multiplier (for exponential backoff).
	Multiplier float64 `yaml:"multiplier"`
}

// BackoffStrategy defines the backoff algorithm.
type BackoffStrategy string

const (
	// BackoffFixed uses a fixed interval between retries.
	BackoffFixed BackoffStrategy = "fixed"

	// BackoffExponential uses exponential backoff.
	BackoffExponential BackoffStrategy = "exponential"
)

// DefaultRetryPolicy returns a RetryPolicy with sensible defaults.
func DefaultRetryPolicy() RetryPolicy {
	return RetryPolicy{
		MaxRetries:      3,
		Backoff:         BackoffExponential,
		InitialInterval: 100 * time.Millisecond,
		MaxInterval:     5 * time.Second,
		Multiplier:      2.0,
	}
}

// stepEntry holds a step and its configuration within a saga definition.
type stepEntry[C any] struct {
	step        SagaStep[C]
	retryPolicy *RetryPolicy
	timeout     time.Duration
}

// SagaDefinition describes a complete saga with its steps and configuration.
type SagaDefinition[C any] struct {
	// Name is the saga name, used for identification and logging.
	Name string

	// Steps is the ordered list of saga steps.
	steps []stepEntry[C]
}

// SagaBuilder builds a SagaDefinition using the builder pattern.
type SagaBuilder[C any] struct {
	name  string
	steps []stepEntry[C]
}

// NewSagaBuilder creates a new SagaBuilder with the given saga name.
func NewSagaBuilder[C any](name string) *SagaBuilder[C] {
	return &SagaBuilder[C]{
		name: name,
	}
}

// Step adds a step to the saga.
func (b *SagaBuilder[C]) Step(step SagaStep[C]) *SagaBuilder[C] {
	b.steps = append(b.steps, stepEntry[C]{
		step: step,
	})
	return b
}

// StepWithRetry adds a step with a custom retry policy.
func (b *SagaBuilder[C]) StepWithRetry(step SagaStep[C], policy RetryPolicy) *SagaBuilder[C] {
	b.steps = append(b.steps, stepEntry[C]{
		step:        step,
		retryPolicy: &policy,
	})
	return b
}

// StepWithTimeout adds a step with a custom timeout.
func (b *SagaBuilder[C]) StepWithTimeout(step SagaStep[C], timeout time.Duration) *SagaBuilder[C] {
	b.steps = append(b.steps, stepEntry[C]{
		step:    step,
		timeout: timeout,
	})
	return b
}

// Build creates the SagaDefinition.
func (b *SagaBuilder[C]) Build() (*SagaDefinition[C], error) {
	if b.name == "" {
		return nil, fmt.Errorf("consensus: saga name is required")
	}
	if len(b.steps) == 0 {
		return nil, fmt.Errorf("consensus: saga must have at least one step")
	}
	return &SagaDefinition[C]{
		Name:  b.name,
		steps: b.steps,
	}, nil
}

// SagaResult holds the outcome of a saga execution.
type SagaResult struct {
	// SagaID is the unique identifier for this saga instance.
	SagaID string

	// SagaName is the name of the saga definition.
	SagaName string

	// Status is the final status.
	Status SagaStatus

	// CompletedSteps is the number of steps that completed successfully.
	CompletedSteps int

	// Error is the error that caused the saga to fail, if any.
	Error error

	// StepOutputs maps step name to its Execute output.
	StepOutputs map[string]json.RawMessage

	// StartedAt is when the saga started.
	StartedAt time.Time

	// CompletedAt is when the saga finished.
	CompletedAt time.Time
}

// SagaInstance represents a persisted saga state.
type SagaInstance struct {
	// ID is the unique instance identifier.
	ID string

	// SagaName is the saga definition name.
	SagaName string

	// Status is the current status.
	Status SagaStatus

	// CurrentStep is the index of the current step.
	CurrentStep int

	// StepOutputs maps step name to its output.
	StepOutputs map[string]json.RawMessage

	// Error is the last error message, if any.
	Error string

	// CreatedAt is when the instance was created.
	CreatedAt time.Time

	// UpdatedAt is when the instance was last updated.
	UpdatedAt time.Time
}
