package consensus

import (
	"context"
	"encoding/json"
	"fmt"
	"time"

	"github.com/google/uuid"
	"github.com/jackc/pgx/v5"
	"github.com/jackc/pgx/v5/pgxpool"
)

// SagaOrchestrator executes sagas sequentially with persistence and compensation.
//
// It persists saga state to PostgreSQL so that interrupted sagas can be
// resumed. If a step fails, it runs compensation in reverse order.
// If compensation also fails, the saga is moved to the dead letter table.
type SagaOrchestrator struct {
	pool   *pgxpool.Pool
	config SagaConfig
}

// NewSagaOrchestrator creates a new SagaOrchestrator.
func NewSagaOrchestrator(pool *pgxpool.Pool, config SagaConfig) *SagaOrchestrator {
	config.Validate()
	return &SagaOrchestrator{
		pool:   pool,
		config: config,
	}
}

// EnsureTables creates the saga tables if they do not exist.
func (o *SagaOrchestrator) EnsureTables(ctx context.Context) error {
	sql := fmt.Sprintf(`
		CREATE TABLE IF NOT EXISTS %s (
			id           TEXT PRIMARY KEY,
			saga_name    TEXT NOT NULL,
			status       TEXT NOT NULL DEFAULT 'pending',
			current_step INT NOT NULL DEFAULT 0,
			step_outputs JSONB NOT NULL DEFAULT '{}',
			error        TEXT NOT NULL DEFAULT '',
			created_at   TIMESTAMPTZ NOT NULL DEFAULT NOW(),
			updated_at   TIMESTAMPTZ NOT NULL DEFAULT NOW()
		);
		CREATE TABLE IF NOT EXISTS %s (
			id           TEXT PRIMARY KEY,
			saga_name    TEXT NOT NULL,
			instance_id  TEXT NOT NULL,
			step_outputs JSONB NOT NULL DEFAULT '{}',
			error        TEXT NOT NULL DEFAULT '',
			created_at   TIMESTAMPTZ NOT NULL DEFAULT NOW()
		);
	`, o.config.TableName, o.config.DeadLetterTableName)

	_, err := o.pool.Exec(ctx, sql)
	if err != nil {
		return fmt.Errorf("consensus: failed to create saga tables: %w", err)
	}
	return nil
}

// executeStepWithRetry runs a single step with retry logic.
func executeStepWithRetry[C any](ctx context.Context, entry stepEntry[C], sagaCtx *C, defaultPolicy RetryPolicy) (json.RawMessage, error) {
	policy := defaultPolicy
	if entry.retryPolicy != nil {
		policy = *entry.retryPolicy
	}

	var lastErr error
	interval := policy.InitialInterval

	for attempt := 0; attempt <= policy.MaxRetries; attempt++ {
		output, err := entry.step.Execute(ctx, sagaCtx)
		if err == nil {
			return output, nil
		}
		lastErr = err

		if attempt < policy.MaxRetries {
			select {
			case <-ctx.Done():
				return nil, ctx.Err()
			case <-time.After(interval):
			}

			if policy.Backoff == BackoffExponential {
				interval = time.Duration(float64(interval) * policy.Multiplier)
				if interval > policy.MaxInterval {
					interval = policy.MaxInterval
				}
			}
		}
	}

	return nil, fmt.Errorf("consensus: step %q failed after %d attempts: %w", entry.step.Name(), policy.MaxRetries+1, lastErr)
}

// compensate runs compensation in reverse order for completed steps.
func compensate[C any](ctx context.Context, def *SagaDefinition[C], sagaCtx *C, outputs map[string]json.RawMessage, failedStep int) error {
	// Compensate in reverse order, from the step before the failed one.
	for i := failedStep - 1; i >= 0; i-- {
		stepName := def.steps[i].step.Name()
		output := outputs[stepName]

		err := def.steps[i].step.Compensate(ctx, sagaCtx, output)
		if err != nil {
			return fmt.Errorf("consensus: compensation failed at step %q: %w", stepName, err)
		}
	}
	return nil
}

func (o *SagaOrchestrator) persistInstance(ctx context.Context, id, sagaName string, status SagaStatus, currentStep int, outputs map[string]json.RawMessage, errMsg string) error {
	outputsJSON, err := json.Marshal(outputs)
	if err != nil {
		return fmt.Errorf("consensus: failed to marshal step outputs: %w", err)
	}

	sql := fmt.Sprintf(`
		INSERT INTO %s (id, saga_name, status, current_step, step_outputs, error)
		VALUES ($1, $2, $3, $4, $5, $6)
		ON CONFLICT (id) DO UPDATE
		SET status       = EXCLUDED.status,
		    current_step = EXCLUDED.current_step,
		    step_outputs = EXCLUDED.step_outputs,
		    error        = EXCLUDED.error,
		    updated_at   = NOW()
	`, o.config.TableName)

	_, execErr := o.pool.Exec(ctx, sql, id, sagaName, string(status), currentStep, outputsJSON, errMsg)
	if execErr != nil {
		return fmt.Errorf("consensus: failed to persist saga instance: %w", execErr)
	}
	return nil
}

func (o *SagaOrchestrator) persistDeadLetter(ctx context.Context, instanceID, sagaName string, outputs map[string]json.RawMessage, errMsg string) error {
	outputsJSON, err := json.Marshal(outputs)
	if err != nil {
		return fmt.Errorf("consensus: failed to marshal step outputs: %w", err)
	}

	dlID := uuid.New().String()
	sql := fmt.Sprintf(`
		INSERT INTO %s (id, saga_name, instance_id, step_outputs, error)
		VALUES ($1, $2, $3, $4, $5)
	`, o.config.DeadLetterTableName)

	_, execErr := o.pool.Exec(ctx, sql, dlID, sagaName, instanceID, outputsJSON, errMsg)
	if execErr != nil {
		return fmt.Errorf("consensus: failed to persist dead letter: %w", execErr)
	}
	return nil
}

// Resume loads a persisted saga instance and attempts to continue or retry it.
func (o *SagaOrchestrator) Resume(ctx context.Context, sagaID string) (*SagaInstance, error) {
	sql := fmt.Sprintf(`
		SELECT id, saga_name, status, current_step, step_outputs, error, created_at, updated_at
		FROM %s WHERE id = $1
	`, o.config.TableName)

	var inst SagaInstance
	var outputsJSON []byte
	err := o.pool.QueryRow(ctx, sql, sagaID).Scan(
		&inst.ID, &inst.SagaName, &inst.Status, &inst.CurrentStep,
		&outputsJSON, &inst.Error, &inst.CreatedAt, &inst.UpdatedAt,
	)
	if err != nil {
		if err == pgx.ErrNoRows {
			return nil, fmt.Errorf("consensus: saga instance %q not found: %w", sagaID, ErrSagaFailed)
		}
		return nil, fmt.Errorf("consensus: failed to load saga instance: %w", err)
	}

	if err := json.Unmarshal(outputsJSON, &inst.StepOutputs); err != nil {
		return nil, fmt.Errorf("consensus: failed to unmarshal step outputs: %w", err)
	}

	return &inst, nil
}

// DeadLetters returns all saga instances in the dead letter table.
func (o *SagaOrchestrator) DeadLetters(ctx context.Context) ([]SagaInstance, error) {
	sql := fmt.Sprintf(`
		SELECT id, saga_name, instance_id, step_outputs, error, created_at
		FROM %s ORDER BY created_at DESC
	`, o.config.DeadLetterTableName)

	rows, err := o.pool.Query(ctx, sql)
	if err != nil {
		return nil, fmt.Errorf("consensus: failed to query dead letters: %w", err)
	}
	defer rows.Close()

	var results []SagaInstance
	for rows.Next() {
		var inst SagaInstance
		var outputsJSON []byte
		var instanceID string
		if err := rows.Scan(&inst.ID, &inst.SagaName, &instanceID, &outputsJSON, &inst.Error, &inst.CreatedAt); err != nil {
			return nil, fmt.Errorf("consensus: failed to scan dead letter row: %w", err)
		}
		if err := json.Unmarshal(outputsJSON, &inst.StepOutputs); err != nil {
			return nil, fmt.Errorf("consensus: failed to unmarshal dead letter outputs: %w", err)
		}
		inst.Status = SagaStatusDeadLetter
		results = append(results, inst)
	}

	return results, rows.Err()
}

// ExecuteSaga is the top-level generic function to execute a saga.
// This exists because Go methods cannot have type parameters.
func ExecuteSaga[C any](o *SagaOrchestrator, ctx context.Context, def *SagaDefinition[C], sagaCtx *C) (*SagaResult, error) {
	sagaID := uuid.New().String()
	now := time.Now()

	result := &SagaResult{
		SagaID:      sagaID,
		SagaName:    def.Name,
		Status:      SagaStatusRunning,
		StepOutputs: make(map[string]json.RawMessage),
		StartedAt:   now,
	}

	if err := o.persistInstance(ctx, sagaID, def.Name, SagaStatusRunning, 0, result.StepOutputs, ""); err != nil {
		return nil, fmt.Errorf("consensus: failed to persist saga: %w", err)
	}

	metricsSagaExecutions.WithLabelValues(def.Name, "started").Inc()

	defaultPolicy := DefaultRetryPolicy()
	defaultPolicy.MaxRetries = o.config.MaxRetries

	var failedStep int
	var stepErr error

	for i, entry := range def.steps {
		stepName := entry.step.Name()

		timeout := o.config.StepTimeout
		if entry.timeout > 0 {
			timeout = entry.timeout
		}

		stepCtx, cancel := context.WithTimeout(ctx, timeout)
		output, err := executeStepWithRetry(stepCtx, entry, sagaCtx, defaultPolicy)
		cancel()

		if err != nil {
			failedStep = i
			stepErr = err
			break
		}

		result.StepOutputs[stepName] = output
		result.CompletedSteps = i + 1

		_ = o.persistInstance(ctx, sagaID, def.Name, SagaStatusRunning, i+1, result.StepOutputs, "")
	}

	if stepErr != nil {
		metricsSagaExecutions.WithLabelValues(def.Name, "compensating").Inc()
		_ = o.persistInstance(ctx, sagaID, def.Name, SagaStatusCompensating, failedStep, result.StepOutputs, stepErr.Error())

		compErr := compensate(ctx, def, sagaCtx, result.StepOutputs, failedStep)
		if compErr != nil {
			result.Status = SagaStatusDeadLetter
			result.Error = fmt.Errorf("%w: step error: %v, compensation error: %v", ErrDeadLetter, stepErr, compErr)
			result.CompletedAt = time.Now()

			_ = o.persistInstance(ctx, sagaID, def.Name, SagaStatusDeadLetter, failedStep, result.StepOutputs, result.Error.Error())
			_ = o.persistDeadLetter(ctx, sagaID, def.Name, result.StepOutputs, result.Error.Error())

			metricsSagaExecutions.WithLabelValues(def.Name, "dead_letter").Inc()
			return result, result.Error
		}

		result.Status = SagaStatusFailed
		result.Error = fmt.Errorf("%w: %v", ErrSagaFailed, stepErr)
		result.CompletedAt = time.Now()

		_ = o.persistInstance(ctx, sagaID, def.Name, SagaStatusFailed, failedStep, result.StepOutputs, stepErr.Error())
		metricsSagaExecutions.WithLabelValues(def.Name, "failed").Inc()
		return result, result.Error
	}

	result.Status = SagaStatusCompleted
	result.CompletedAt = time.Now()
	_ = o.persistInstance(ctx, sagaID, def.Name, SagaStatusCompleted, len(def.steps), result.StepOutputs, "")

	metricsSagaExecutions.WithLabelValues(def.Name, "completed").Inc()
	metricsSagaStepDuration.WithLabelValues(def.Name, "total").Observe(result.CompletedAt.Sub(result.StartedAt).Seconds())
	return result, nil
}
