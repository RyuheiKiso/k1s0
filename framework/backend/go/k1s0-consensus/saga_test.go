package consensus

import (
	"context"
	"encoding/json"
	"fmt"
	"testing"
)

// mockStep is a test helper implementing SagaStep.
type mockStep struct {
	name          string
	executeFunc   func(ctx context.Context, sagaCtx *testSagaCtx) (json.RawMessage, error)
	compensateFunc func(ctx context.Context, sagaCtx *testSagaCtx, output json.RawMessage) error
}

type testSagaCtx struct {
	ExecutedSteps    []string
	CompensatedSteps []string
}

func (s *mockStep) Name() string { return s.name }

func (s *mockStep) Execute(ctx context.Context, sagaCtx *testSagaCtx) (json.RawMessage, error) {
	sagaCtx.ExecutedSteps = append(sagaCtx.ExecutedSteps, s.name)
	if s.executeFunc != nil {
		return s.executeFunc(ctx, sagaCtx)
	}
	return json.RawMessage(`{"ok":true}`), nil
}

func (s *mockStep) Compensate(ctx context.Context, sagaCtx *testSagaCtx, output json.RawMessage) error {
	sagaCtx.CompensatedSteps = append(sagaCtx.CompensatedSteps, s.name)
	if s.compensateFunc != nil {
		return s.compensateFunc(ctx, sagaCtx, output)
	}
	return nil
}

func TestSagaBuilder_Build(t *testing.T) {
	t.Run("success", func(t *testing.T) {
		def, err := NewSagaBuilder[testSagaCtx]("test-saga").
			Step(&mockStep{name: "step1"}).
			Step(&mockStep{name: "step2"}).
			Build()

		if err != nil {
			t.Fatalf("Build() error: %v", err)
		}
		if def.Name != "test-saga" {
			t.Errorf("Name = %q, want test-saga", def.Name)
		}
		if len(def.steps) != 2 {
			t.Errorf("len(steps) = %d, want 2", len(def.steps))
		}
	})

	t.Run("empty name", func(t *testing.T) {
		_, err := NewSagaBuilder[testSagaCtx]("").
			Step(&mockStep{name: "step1"}).
			Build()

		if err == nil {
			t.Error("Build() expected error for empty name")
		}
	})

	t.Run("no steps", func(t *testing.T) {
		_, err := NewSagaBuilder[testSagaCtx]("test-saga").Build()

		if err == nil {
			t.Error("Build() expected error for no steps")
		}
	})
}

func TestSagaBuilder_StepWithRetry(t *testing.T) {
	policy := RetryPolicy{MaxRetries: 5}
	def, err := NewSagaBuilder[testSagaCtx]("retry-saga").
		StepWithRetry(&mockStep{name: "step1"}, policy).
		Build()

	if err != nil {
		t.Fatalf("Build() error: %v", err)
	}
	if def.steps[0].retryPolicy == nil {
		t.Fatal("expected retry policy to be set")
	}
	if def.steps[0].retryPolicy.MaxRetries != 5 {
		t.Errorf("MaxRetries = %d, want 5", def.steps[0].retryPolicy.MaxRetries)
	}
}

func TestSagaStatus_Values(t *testing.T) {
	statuses := []SagaStatus{
		SagaStatusPending,
		SagaStatusRunning,
		SagaStatusCompleted,
		SagaStatusCompensating,
		SagaStatusCompensated,
		SagaStatusFailed,
		SagaStatusDeadLetter,
	}

	// Verify all statuses are distinct.
	seen := make(map[SagaStatus]bool)
	for _, s := range statuses {
		if seen[s] {
			t.Errorf("duplicate status: %s", s)
		}
		seen[s] = true
	}
}

func TestCompensate_ReverseOrder(t *testing.T) {
	sagaCtx := &testSagaCtx{}
	steps := []stepEntry[testSagaCtx]{
		{step: &mockStep{name: "step1"}},
		{step: &mockStep{name: "step2"}},
		{step: &mockStep{name: "step3"}},
	}

	def := &SagaDefinition[testSagaCtx]{
		Name:  "test",
		steps: steps,
	}

	outputs := map[string]json.RawMessage{
		"step1": json.RawMessage(`{}`),
		"step2": json.RawMessage(`{}`),
	}

	// Failed at step 2 (index 2), so compensate step2 and step1.
	err := compensate(context.Background(), def, sagaCtx, outputs, 2)
	if err != nil {
		t.Fatalf("compensate() error: %v", err)
	}

	if len(sagaCtx.CompensatedSteps) != 2 {
		t.Fatalf("compensated %d steps, want 2", len(sagaCtx.CompensatedSteps))
	}

	// Should be in reverse order.
	if sagaCtx.CompensatedSteps[0] != "step2" {
		t.Errorf("first compensated = %q, want step2", sagaCtx.CompensatedSteps[0])
	}
	if sagaCtx.CompensatedSteps[1] != "step1" {
		t.Errorf("second compensated = %q, want step1", sagaCtx.CompensatedSteps[1])
	}
}

func TestCompensate_Error(t *testing.T) {
	sagaCtx := &testSagaCtx{}
	steps := []stepEntry[testSagaCtx]{
		{step: &mockStep{name: "step1"}},
		{step: &mockStep{
			name: "step2",
			compensateFunc: func(_ context.Context, _ *testSagaCtx, _ json.RawMessage) error {
				return fmt.Errorf("compensation failure")
			},
		}},
	}

	def := &SagaDefinition[testSagaCtx]{
		Name:  "test",
		steps: steps,
	}

	outputs := map[string]json.RawMessage{
		"step1": json.RawMessage(`{}`),
		"step2": json.RawMessage(`{}`),
	}

	err := compensate(context.Background(), def, sagaCtx, outputs, 2)
	if err == nil {
		t.Error("compensate() expected error")
	}
}

func TestExecuteStepWithRetry_Success(t *testing.T) {
	entry := stepEntry[testSagaCtx]{
		step: &mockStep{name: "step1"},
	}
	sagaCtx := &testSagaCtx{}

	output, err := executeStepWithRetry(context.Background(), entry, sagaCtx, DefaultRetryPolicy())
	if err != nil {
		t.Fatalf("executeStepWithRetry() error: %v", err)
	}
	if output == nil {
		t.Error("expected non-nil output")
	}
}

func TestExecuteStepWithRetry_RetryThenSuccess(t *testing.T) {
	attempts := 0
	entry := stepEntry[testSagaCtx]{
		step: &mockStep{
			name: "flaky-step",
			executeFunc: func(_ context.Context, sagaCtx *testSagaCtx) (json.RawMessage, error) {
				attempts++
				if attempts < 3 {
					return nil, fmt.Errorf("transient error")
				}
				return json.RawMessage(`{"ok":true}`), nil
			},
		},
	}
	sagaCtx := &testSagaCtx{}

	policy := RetryPolicy{
		MaxRetries:      3,
		Backoff:         BackoffFixed,
		InitialInterval: 1, // Minimal for tests.
		MaxInterval:     1,
	}

	output, err := executeStepWithRetry(context.Background(), entry, sagaCtx, policy)
	if err != nil {
		t.Fatalf("executeStepWithRetry() error: %v", err)
	}
	if output == nil {
		t.Error("expected non-nil output")
	}
	if attempts != 3 {
		t.Errorf("attempts = %d, want 3", attempts)
	}
}
