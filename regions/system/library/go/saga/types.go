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
	StepName  string    `json:"step_name"`
	Status    string    `json:"status"`
	Message   string    `json:"message"`
	CreatedAt time.Time `json:"created_at"`
}

// SagaState は Saga の現在状態。
type SagaState struct {
	SagaID    string        `json:"saga_id"`
	WorkflowName string     `json:"workflow_name"`
	Status    SagaStatus    `json:"status"`
	StepLogs  []SagaStepLog `json:"step_logs"`
	CreatedAt time.Time     `json:"created_at"`
	UpdatedAt time.Time     `json:"updated_at"`
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
