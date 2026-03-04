package saga

import "time"

// SagaStatus は Saga の実行ステータス。
type SagaStatus string

const (
	SagaStatusStarted      SagaStatus = "STARTED"
	SagaStatusRunning      SagaStatus = "RUNNING"
	SagaStatusCompleted    SagaStatus = "COMPLETED"
	SagaStatusCompensating SagaStatus = "COMPENSATING"
	SagaStatusFailed       SagaStatus = "FAILED"
	SagaStatusCancelled    SagaStatus = "CANCELLED"
)

// SagaStepLog は Saga の各ステップの実行ログ。
type SagaStepLog struct {
	ID              string     `json:"id"`
	SagaID          string     `json:"saga_id"`
	StepIndex       int        `json:"step_index"`
	StepName        string     `json:"step_name"`
	Action          string     `json:"action"`
	Status          string     `json:"status"`
	RequestPayload  any        `json:"request_payload"`
	ResponsePayload any        `json:"response_payload"`
	ErrorMessage    string     `json:"error_message"`
	StartedAt       time.Time  `json:"started_at"`
	CompletedAt     *time.Time `json:"completed_at"`
}

// SagaState は Saga の現在状態。
type SagaState struct {
	SagaID        string         `json:"saga_id"`
	WorkflowName  string         `json:"workflow_name"`
	CurrentStep   int            `json:"current_step"`
	Status        SagaStatus     `json:"status"`
	Payload       map[string]any `json:"payload"`
	CorrelationID *string        `json:"correlation_id"`
	InitiatedBy   *string        `json:"initiated_by"`
	ErrorMessage  *string        `json:"error_message"`
	StepLogs      []SagaStepLog  `json:"step_logs"`
	CreatedAt     time.Time      `json:"created_at"`
	UpdatedAt     time.Time      `json:"updated_at"`
}

// StartSagaRequest は Saga 開始リクエスト。
type StartSagaRequest struct {
	WorkflowName string `json:"workflow_name"`
	Payload  any    `json:"payload"`
}

// StartSagaResponse は Saga 開始レスポンス。
type StartSagaResponse struct {
	SagaID string `json:"saga_id"`
}
