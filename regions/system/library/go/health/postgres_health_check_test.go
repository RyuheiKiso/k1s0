package health

import (
	"testing"
	"time"

	"github.com/stretchr/testify/assert"
)

// PostgresHealthCheck_Nameがデフォルト名「postgres」を返すことを検証する。
func TestPostgresHealthCheck_Name(t *testing.T) {
	h := NewPostgresHealthCheck(nil)
	assert.Equal(t, "postgres", h.Name())
}

// PostgresHealthCheck_CustomNameがWithPostgresNameオプションでカスタム名を設定できることを検証する。
func TestPostgresHealthCheck_CustomName(t *testing.T) {
	h := NewPostgresHealthCheck(nil, WithPostgresName("primary-db"))
	assert.Equal(t, "primary-db", h.Name())
}

// PostgresHealthCheck_DefaultTimeoutがデフォルトタイムアウトとして5秒が設定されることを検証する。
func TestPostgresHealthCheck_DefaultTimeout(t *testing.T) {
	h := NewPostgresHealthCheck(nil)
	assert.Equal(t, 5*time.Second, h.timeout)
}

// PostgresHealthCheck_CustomTimeoutがWithPostgresTimeoutオプションでカスタムタイムアウトを設定できることを検証する。
func TestPostgresHealthCheck_CustomTimeout(t *testing.T) {
	h := NewPostgresHealthCheck(nil, WithPostgresTimeout(2*time.Second))
	assert.Equal(t, 2*time.Second, h.timeout)
}

// PostgresHealthCheck_ImplementsHealthCheckがPostgresHealthCheckがHealthCheckインターフェースを実装していることを検証する。
func TestPostgresHealthCheck_ImplementsHealthCheck(t *testing.T) {
	var _ HealthCheck = (*PostgresHealthCheck)(nil)
}
