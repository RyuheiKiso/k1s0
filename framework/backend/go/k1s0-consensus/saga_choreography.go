package consensus

import (
	"context"
	"encoding/json"
	"fmt"
	"sync"
	"time"

	"github.com/google/uuid"
)

// EventStepHandler defines a handler for a choreography saga step
// that processes incoming events and produces outgoing events.
type EventStepHandler interface {
	// Name returns the unique name of this step handler.
	Name() string

	// HandleEvent processes an incoming event and returns a result payload.
	// Returning an error triggers compensation.
	HandleEvent(ctx context.Context, eventType string, payload json.RawMessage) (json.RawMessage, error)

	// Compensate undoes the action performed by HandleEvent.
	Compensate(ctx context.Context, payload json.RawMessage) error
}

// choreographyStep holds a step handler and its trigger event type.
type choreographyStep struct {
	handler     EventStepHandler
	triggerType string
	timeout     time.Duration
}

// ChoreographySaga implements event-driven saga choreography.
// Each step is triggered by an event type rather than being called sequentially.
// Timeout monitoring ensures the saga does not run indefinitely.
type ChoreographySaga struct {
	name    string
	steps   []choreographyStep
	timeout time.Duration

	mu        sync.Mutex
	instances map[string]*choreographyInstance
}

// choreographyInstance tracks the state of a running choreography saga.
type choreographyInstance struct {
	id            string
	currentStep   int
	stepOutputs   map[string]json.RawMessage
	status        SagaStatus
	cancel        context.CancelFunc
	completedChan chan SagaResult
}

// ChoreographySagaBuilder builds a ChoreographySaga using the builder pattern.
type ChoreographySagaBuilder struct {
	name    string
	steps   []choreographyStep
	timeout time.Duration
}

// NewChoreographySagaBuilder creates a new builder for a choreography saga.
func NewChoreographySagaBuilder(name string) *ChoreographySagaBuilder {
	return &ChoreographySagaBuilder{
		name:    name,
		timeout: 5 * time.Minute,
	}
}

// OnEvent registers a step handler that triggers on the given event type.
func (b *ChoreographySagaBuilder) OnEvent(eventType string, handler EventStepHandler) *ChoreographySagaBuilder {
	b.steps = append(b.steps, choreographyStep{
		handler:     handler,
		triggerType: eventType,
	})
	return b
}

// OnEventWithTimeout registers a step handler with a custom timeout.
func (b *ChoreographySagaBuilder) OnEventWithTimeout(eventType string, handler EventStepHandler, timeout time.Duration) *ChoreographySagaBuilder {
	b.steps = append(b.steps, choreographyStep{
		handler:     handler,
		triggerType: eventType,
		timeout:     timeout,
	})
	return b
}

// Timeout sets the overall saga timeout.
func (b *ChoreographySagaBuilder) Timeout(d time.Duration) *ChoreographySagaBuilder {
	b.timeout = d
	return b
}

// Build creates the ChoreographySaga.
func (b *ChoreographySagaBuilder) Build() (*ChoreographySaga, error) {
	if b.name == "" {
		return nil, fmt.Errorf("consensus: choreography saga name is required")
	}
	if len(b.steps) == 0 {
		return nil, fmt.Errorf("consensus: choreography saga must have at least one step")
	}
	return &ChoreographySaga{
		name:      b.name,
		steps:     b.steps,
		timeout:   b.timeout,
		instances: make(map[string]*choreographyInstance),
	}, nil
}

// Start initiates a new choreography saga instance and returns its ID.
// The saga will be monitored for the configured timeout duration.
func (s *ChoreographySaga) Start(ctx context.Context) (string, <-chan SagaResult, error) {
	id := uuid.New().String()
	sagaCtx, cancel := context.WithTimeout(ctx, s.timeout)

	inst := &choreographyInstance{
		id:            id,
		currentStep:   0,
		stepOutputs:   make(map[string]json.RawMessage),
		status:        SagaStatusRunning,
		cancel:        cancel,
		completedChan: make(chan SagaResult, 1),
	}

	s.mu.Lock()
	s.instances[id] = inst
	s.mu.Unlock()

	// Timeout monitoring goroutine.
	go func() {
		<-sagaCtx.Done()

		s.mu.Lock()
		instance, ok := s.instances[id]
		s.mu.Unlock()

		if ok && instance.status == SagaStatusRunning {
			s.mu.Lock()
			instance.status = SagaStatusFailed
			s.mu.Unlock()

			instance.completedChan <- SagaResult{
				SagaID:      id,
				SagaName:    s.name,
				Status:      SagaStatusFailed,
				Error:       fmt.Errorf("consensus: choreography saga timed out: %w", ErrSagaFailed),
				StepOutputs: instance.stepOutputs,
				CompletedAt: time.Now(),
			}
			close(instance.completedChan)
		}
	}()

	metricsSagaExecutions.WithLabelValues(s.name, "started").Inc()
	return id, inst.completedChan, nil
}

// HandleEvent delivers an event to a running choreography saga instance.
func (s *ChoreographySaga) HandleEvent(ctx context.Context, sagaID string, eventType string, payload json.RawMessage) error {
	s.mu.Lock()
	inst, ok := s.instances[sagaID]
	s.mu.Unlock()

	if !ok {
		return fmt.Errorf("consensus: saga instance %q not found: %w", sagaID, ErrSagaFailed)
	}

	if inst.status != SagaStatusRunning {
		return fmt.Errorf("consensus: saga instance %q is not running (status: %s)", sagaID, inst.status)
	}

	// Find the matching step.
	if inst.currentStep >= len(s.steps) {
		return fmt.Errorf("consensus: saga instance %q has no more steps", sagaID)
	}

	step := s.steps[inst.currentStep]
	if step.triggerType != eventType {
		return fmt.Errorf("consensus: expected event type %q, got %q", step.triggerType, eventType)
	}

	// Apply step timeout if configured.
	stepCtx := ctx
	var cancel context.CancelFunc
	if step.timeout > 0 {
		stepCtx, cancel = context.WithTimeout(ctx, step.timeout)
	} else {
		stepCtx, cancel = context.WithCancel(ctx)
	}
	defer cancel()

	output, err := step.handler.HandleEvent(stepCtx, eventType, payload)
	if err != nil {
		// Compensate in reverse.
		s.mu.Lock()
		inst.status = SagaStatusCompensating
		s.mu.Unlock()

		compErr := s.compensateInstance(ctx, inst)

		s.mu.Lock()
		if compErr != nil {
			inst.status = SagaStatusDeadLetter
		} else {
			inst.status = SagaStatusFailed
		}
		s.mu.Unlock()

		result := SagaResult{
			SagaID:      sagaID,
			SagaName:    s.name,
			Status:      inst.status,
			StepOutputs: inst.stepOutputs,
			Error:       fmt.Errorf("consensus: step %q failed: %w", step.handler.Name(), err),
			CompletedAt: time.Now(),
		}

		inst.completedChan <- result
		close(inst.completedChan)
		inst.cancel()

		metricsSagaExecutions.WithLabelValues(s.name, string(inst.status)).Inc()
		return result.Error
	}

	s.mu.Lock()
	inst.stepOutputs[step.handler.Name()] = output
	inst.currentStep++
	allDone := inst.currentStep >= len(s.steps)
	if allDone {
		inst.status = SagaStatusCompleted
	}
	s.mu.Unlock()

	if allDone {
		result := SagaResult{
			SagaID:         sagaID,
			SagaName:       s.name,
			Status:         SagaStatusCompleted,
			CompletedSteps: len(s.steps),
			StepOutputs:    inst.stepOutputs,
			CompletedAt:    time.Now(),
		}
		inst.completedChan <- result
		close(inst.completedChan)
		inst.cancel()

		metricsSagaExecutions.WithLabelValues(s.name, "completed").Inc()
	}

	return nil
}

// compensateInstance runs compensation for all completed steps in reverse.
func (s *ChoreographySaga) compensateInstance(ctx context.Context, inst *choreographyInstance) error {
	for i := inst.currentStep - 1; i >= 0; i-- {
		step := s.steps[i]
		output := inst.stepOutputs[step.handler.Name()]

		if err := step.handler.Compensate(ctx, output); err != nil {
			return fmt.Errorf("consensus: choreography compensation failed at step %q: %w", step.handler.Name(), err)
		}
	}
	return nil
}

// Cancel cancels a running choreography saga instance.
func (s *ChoreographySaga) Cancel(sagaID string) {
	s.mu.Lock()
	inst, ok := s.instances[sagaID]
	s.mu.Unlock()

	if ok {
		inst.cancel()
	}
}
