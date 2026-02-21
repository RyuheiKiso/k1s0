package kafka_test

import (
	"context"
	"errors"
	"testing"

	kafka "github.com/k1s0-platform/system-library-go-kafka"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// --- KafkaConfig Tests ---

func TestKafkaConfig_BootstrapServersString(t *testing.T) {
	cfg := &kafka.KafkaConfig{
		BootstrapServers: []string{"broker1:9092", "broker2:9092"},
	}
	assert.Equal(t, "broker1:9092,broker2:9092", cfg.BootstrapServersString())
}

func TestKafkaConfig_BootstrapServersString_Single(t *testing.T) {
	cfg := &kafka.KafkaConfig{
		BootstrapServers: []string{"broker1:9092"},
	}
	assert.Equal(t, "broker1:9092", cfg.BootstrapServersString())
}

func TestKafkaConfig_UsesTLS_SSL(t *testing.T) {
	cfg := &kafka.KafkaConfig{SecurityProtocol: "SSL"}
	assert.True(t, cfg.UsesTLS())
}

func TestKafkaConfig_UsesTLS_SASL_SSL(t *testing.T) {
	cfg := &kafka.KafkaConfig{SecurityProtocol: "SASL_SSL"}
	assert.True(t, cfg.UsesTLS())
}

func TestKafkaConfig_UsesTLS_Plaintext(t *testing.T) {
	cfg := &kafka.KafkaConfig{SecurityProtocol: "PLAINTEXT"}
	assert.False(t, cfg.UsesTLS())
}

func TestKafkaConfig_Validate_Valid(t *testing.T) {
	cfg := &kafka.KafkaConfig{
		BootstrapServers: []string{"broker1:9092"},
		SecurityProtocol: "PLAINTEXT",
	}
	assert.NoError(t, cfg.Validate())
}

func TestKafkaConfig_Validate_EmptyBrokers(t *testing.T) {
	cfg := &kafka.KafkaConfig{}
	assert.Error(t, cfg.Validate())
}

func TestKafkaConfig_Validate_InvalidProtocol(t *testing.T) {
	cfg := &kafka.KafkaConfig{
		BootstrapServers: []string{"broker1:9092"},
		SecurityProtocol: "INVALID",
	}
	assert.Error(t, cfg.Validate())
}

// --- TopicConfig Tests ---

func TestTopicConfig_ValidateName_Valid(t *testing.T) {
	tests := []struct {
		name string
	}{
		{"k1s0.system.user.created.v1"},
		{"k1s0.business.order.placed.v2"},
		{"k1s0.service.payment.processed.v1"},
		{"k1s0.system.user-profile.updated.v10"},
		{"k1s0.system.auth.token-refreshed.v1"},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			tc := &kafka.TopicConfig{Name: tt.name}
			assert.NoError(t, tc.ValidateName())
		})
	}
}

func TestTopicConfig_ValidateName_Invalid(t *testing.T) {
	tests := []struct {
		name string
	}{
		{""},
		{"invalid"},
		{"k1s0.invalid.user.created.v1"},
		{"k1s0.system.USER.created.v1"},
		{"k1s0.system.user.created"},
		{"k1s0.system.user.created.v"},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			tc := &kafka.TopicConfig{Name: tt.name}
			assert.Error(t, tc.ValidateName())
		})
	}
}

func TestTopicConfig_Tier(t *testing.T) {
	tests := []struct {
		name     string
		expected string
	}{
		{"k1s0.system.user.created.v1", "system"},
		{"k1s0.business.order.placed.v1", "business"},
		{"k1s0.service.payment.done.v1", "service"},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			tc := &kafka.TopicConfig{Name: tt.name}
			assert.Equal(t, tt.expected, tc.Tier())
		})
	}
}

// --- KafkaHealthChecker Tests ---

func TestNoOpKafkaHealthChecker_Healthy(t *testing.T) {
	checker := &kafka.NoOpKafkaHealthChecker{
		Status: &kafka.KafkaHealthStatus{Healthy: true, Message: "OK", BrokerCount: 3},
	}
	status, err := checker.HealthCheck(context.Background())
	require.NoError(t, err)
	assert.True(t, status.Healthy)
	assert.Equal(t, 3, status.BrokerCount)
}

func TestNoOpKafkaHealthChecker_Unhealthy(t *testing.T) {
	checker := &kafka.NoOpKafkaHealthChecker{
		Status: &kafka.KafkaHealthStatus{Healthy: false, Message: "connection refused"},
	}
	status, err := checker.HealthCheck(context.Background())
	require.NoError(t, err)
	assert.False(t, status.Healthy)
}

func TestNoOpKafkaHealthChecker_Error(t *testing.T) {
	expectedErr := errors.New("connection refused")
	checker := &kafka.NoOpKafkaHealthChecker{
		Err: expectedErr,
	}
	_, err := checker.HealthCheck(context.Background())
	assert.ErrorIs(t, err, expectedErr)
}

func TestKafkaConfig_UsesTLS_SASLPlaintext(t *testing.T) {
	cfg := &kafka.KafkaConfig{SecurityProtocol: "SASL_PLAINTEXT"}
	assert.False(t, cfg.UsesTLS())
}

func TestTopicConfig_Tier_InvalidName(t *testing.T) {
	tc := &kafka.TopicConfig{Name: "invalid-name"}
	assert.Equal(t, "", tc.Tier())
}
