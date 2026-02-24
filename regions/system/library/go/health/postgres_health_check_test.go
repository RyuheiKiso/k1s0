package health

import (
	"testing"
	"time"

	"github.com/stretchr/testify/assert"
)

func TestPostgresHealthCheck_Name(t *testing.T) {
	h := NewPostgresHealthCheck(nil)
	assert.Equal(t, "postgres", h.Name())
}

func TestPostgresHealthCheck_CustomName(t *testing.T) {
	h := NewPostgresHealthCheck(nil, WithPostgresName("primary-db"))
	assert.Equal(t, "primary-db", h.Name())
}

func TestPostgresHealthCheck_DefaultTimeout(t *testing.T) {
	h := NewPostgresHealthCheck(nil)
	assert.Equal(t, 5*time.Second, h.timeout)
}

func TestPostgresHealthCheck_CustomTimeout(t *testing.T) {
	h := NewPostgresHealthCheck(nil, WithPostgresTimeout(2*time.Second))
	assert.Equal(t, 2*time.Second, h.timeout)
}

func TestPostgresHealthCheck_ImplementsHealthCheck(t *testing.T) {
	var _ HealthCheck = (*PostgresHealthCheck)(nil)
}
