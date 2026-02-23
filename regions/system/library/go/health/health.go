package health

import (
	"context"
	"time"
)

// Status はヘルスチェックのステータス。
type Status string

const (
	StatusHealthy   Status = "healthy"
	StatusDegraded  Status = "degraded"
	StatusUnhealthy Status = "unhealthy"
)

// CheckResult はヘルスチェックの結果。
type CheckResult struct {
	Status  Status
	Message string
}

// HealthResponse はヘルスチェックのレスポンス。
type HealthResponse struct {
	Status    Status
	Checks    map[string]CheckResult
	Timestamp time.Time
}

// HealthCheck はヘルスチェックのインターフェース。
type HealthCheck interface {
	Name() string
	Check(ctx context.Context) error
}

// Checker はヘルスチェッカー。
type Checker struct {
	checks []HealthCheck
}

// NewChecker は新しい Checker を生成する。
func NewChecker() *Checker {
	return &Checker{}
}

// Add はヘルスチェックを追加する。
func (c *Checker) Add(check HealthCheck) {
	c.checks = append(c.checks, check)
}

// RunAll は全ヘルスチェックを実行する。
func (c *Checker) RunAll(ctx context.Context) HealthResponse {
	results := make(map[string]CheckResult, len(c.checks))
	overall := StatusHealthy

	for _, check := range c.checks {
		err := check.Check(ctx)
		if err != nil {
			results[check.Name()] = CheckResult{Status: StatusUnhealthy, Message: err.Error()}
			overall = StatusUnhealthy
		} else {
			results[check.Name()] = CheckResult{Status: StatusHealthy, Message: "OK"}
		}
	}

	return HealthResponse{
		Status:    overall,
		Checks:    results,
		Timestamp: time.Now(),
	}
}
