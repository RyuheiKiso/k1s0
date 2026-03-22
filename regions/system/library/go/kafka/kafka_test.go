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

// KafkaConfig_BootstrapServersStringが複数ブローカーをカンマ区切りの文字列として返すことを検証する。
func TestKafkaConfig_BootstrapServersString(t *testing.T) {
	cfg := &kafka.KafkaConfig{
		BootstrapServers: []string{"broker1:9092", "broker2:9092"},
	}
	assert.Equal(t, "broker1:9092,broker2:9092", cfg.BootstrapServersString())
}

// KafkaConfig_BootstrapServersString_Singleが単一ブローカーの場合にカンマなしでサーバー文字列を返すことを検証する。
func TestKafkaConfig_BootstrapServersString_Single(t *testing.T) {
	cfg := &kafka.KafkaConfig{
		BootstrapServers: []string{"broker1:9092"},
	}
	assert.Equal(t, "broker1:9092", cfg.BootstrapServersString())
}

// KafkaConfig_UsesTLS_SSLがセキュリティプロトコルSSLの場合にUsesTLSがtrueを返すことを検証する。
func TestKafkaConfig_UsesTLS_SSL(t *testing.T) {
	cfg := &kafka.KafkaConfig{SecurityProtocol: "SSL"}
	assert.True(t, cfg.UsesTLS())
}

// KafkaConfig_UsesTLS_SASL_SSLがセキュリティプロトコルSASL_SSLの場合にUsesTLSがtrueを返すことを検証する。
func TestKafkaConfig_UsesTLS_SASL_SSL(t *testing.T) {
	cfg := &kafka.KafkaConfig{SecurityProtocol: "SASL_SSL"}
	assert.True(t, cfg.UsesTLS())
}

// KafkaConfig_UsesTLS_PlaintextがセキュリティプロトコルPLAINTEXTの場合にUsesTLSがfalseを返すことを検証する。
func TestKafkaConfig_UsesTLS_Plaintext(t *testing.T) {
	cfg := &kafka.KafkaConfig{SecurityProtocol: "PLAINTEXT"}
	assert.False(t, cfg.UsesTLS())
}

// KafkaConfig_Validate_Validが有効な設定のバリデーションが成功することを検証する。
func TestKafkaConfig_Validate_Valid(t *testing.T) {
	cfg := &kafka.KafkaConfig{
		BootstrapServers: []string{"broker1:9092"},
		SecurityProtocol: "PLAINTEXT",
	}
	assert.NoError(t, cfg.Validate())
}

// KafkaConfig_Validate_EmptyBrokersがブローカーリストが空の場合にバリデーションエラーを返すことを検証する。
func TestKafkaConfig_Validate_EmptyBrokers(t *testing.T) {
	cfg := &kafka.KafkaConfig{}
	assert.Error(t, cfg.Validate())
}

// KafkaConfig_Validate_InvalidProtocolが無効なセキュリティプロトコルの場合にバリデーションエラーを返すことを検証する。
func TestKafkaConfig_Validate_InvalidProtocol(t *testing.T) {
	cfg := &kafka.KafkaConfig{
		BootstrapServers: []string{"broker1:9092"},
		SecurityProtocol: "INVALID",
	}
	assert.Error(t, cfg.Validate())
}

// --- TopicConfig Tests ---

// TopicConfig_ValidateName_Validが有効なトピック名に対してバリデーションが成功することを検証する。
func TestTopicConfig_ValidateName_Valid(t *testing.T) {
	tests := []struct {
		name string
	}{
		{"k1s0.system.user.created.v1"},
		{"k1s0.business.order.placed.v2"},
		{"k1s0.service.activity.created.v1"},
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

// TopicConfig_ValidateName_Invalidが無効なトピック名に対してバリデーションエラーを返すことを検証する。
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

// TopicConfig_TierがトピックのTierメソッドがトピック名からシステム/ビジネス/サービスの層を正しく抽出することを検証する。
func TestTopicConfig_Tier(t *testing.T) {
	tests := []struct {
		name     string
		expected string
	}{
		{"k1s0.system.user.created.v1", "system"},
		{"k1s0.business.order.placed.v1", "business"},
		{"k1s0.service.activity.approved.v1", "service"},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			tc := &kafka.TopicConfig{Name: tt.name}
			assert.Equal(t, tt.expected, tc.Tier())
		})
	}
}

// --- KafkaHealthChecker Tests ---

// NoOpKafkaHealthChecker_HealthyがHealthCheckが正常ステータスとブローカー数を返すことを検証する。
func TestNoOpKafkaHealthChecker_Healthy(t *testing.T) {
	checker := &kafka.NoOpKafkaHealthChecker{
		Status: &kafka.KafkaHealthStatus{Healthy: true, Message: "OK", BrokerCount: 3},
	}
	status, err := checker.HealthCheck(context.Background())
	require.NoError(t, err)
	assert.True(t, status.Healthy)
	assert.Equal(t, 3, status.BrokerCount)
}

// NoOpKafkaHealthChecker_Unhealthyがステータス未健全の場合にHealthCheckがhealthy=falseを返すことを検証する。
func TestNoOpKafkaHealthChecker_Unhealthy(t *testing.T) {
	checker := &kafka.NoOpKafkaHealthChecker{
		Status: &kafka.KafkaHealthStatus{Healthy: false, Message: "connection refused"},
	}
	status, err := checker.HealthCheck(context.Background())
	require.NoError(t, err)
	assert.False(t, status.Healthy)
}

// NoOpKafkaHealthChecker_Errorが設定されたエラーをHealthCheckが返すことを検証する。
func TestNoOpKafkaHealthChecker_Error(t *testing.T) {
	expectedErr := errors.New("connection refused")
	checker := &kafka.NoOpKafkaHealthChecker{
		Err: expectedErr,
	}
	_, err := checker.HealthCheck(context.Background())
	assert.ErrorIs(t, err, expectedErr)
}

// KafkaConfig_UsesTLS_SASLPlaintextがSASL_PLAINTEXTプロトコルの場合にUsesTLSがfalseを返すことを検証する。
func TestKafkaConfig_UsesTLS_SASLPlaintext(t *testing.T) {
	cfg := &kafka.KafkaConfig{SecurityProtocol: "SASL_PLAINTEXT"}
	assert.False(t, cfg.UsesTLS())
}

// TopicConfig_Tier_InvalidNameが無効なトピック名の場合にTierが空文字を返すことを検証する。
func TestTopicConfig_Tier_InvalidName(t *testing.T) {
	tc := &kafka.TopicConfig{Name: "invalid-name"}
	assert.Equal(t, "", tc.Tier())
}

// --- KafkaConfig 追加フィールド Tests ---

// KafkaConfig_ConsumerGroupがConsumerGroupフィールドを設定した場合に正しく保存されバリデーションが成功することを検証する。
func TestKafkaConfig_ConsumerGroup(t *testing.T) {
	cfg := &kafka.KafkaConfig{
		BootstrapServers: []string{"broker1:9092"},
		ConsumerGroup:    "my-consumer-group",
	}
	assert.Equal(t, "my-consumer-group", cfg.ConsumerGroup)
	assert.NoError(t, cfg.Validate())
}

// KafkaConfig_EffectiveConnectionTimeoutMs_DefaultがConnectionTimeoutMs未設定時にデフォルト値を返すことを検証する。
func TestKafkaConfig_EffectiveConnectionTimeoutMs_Default(t *testing.T) {
	cfg := &kafka.KafkaConfig{
		BootstrapServers: []string{"broker1:9092"},
	}
	assert.Equal(t, kafka.DefaultConnectionTimeoutMs, cfg.EffectiveConnectionTimeoutMs())
}

// KafkaConfig_EffectiveConnectionTimeoutMs_CustomがConnectionTimeoutMs設定時にカスタム値を返すことを検証する。
func TestKafkaConfig_EffectiveConnectionTimeoutMs_Custom(t *testing.T) {
	cfg := &kafka.KafkaConfig{
		BootstrapServers:    []string{"broker1:9092"},
		ConnectionTimeoutMs: 10000,
	}
	assert.Equal(t, 10000, cfg.EffectiveConnectionTimeoutMs())
}

// KafkaConfig_EffectiveRequestTimeoutMs_DefaultがRequestTimeoutMs未設定時にデフォルト値を返すことを検証する。
func TestKafkaConfig_EffectiveRequestTimeoutMs_Default(t *testing.T) {
	cfg := &kafka.KafkaConfig{
		BootstrapServers: []string{"broker1:9092"},
	}
	assert.Equal(t, kafka.DefaultRequestTimeoutMs, cfg.EffectiveRequestTimeoutMs())
}

// KafkaConfig_EffectiveRequestTimeoutMs_CustomがRequestTimeoutMs設定時にカスタム値を返すことを検証する。
func TestKafkaConfig_EffectiveRequestTimeoutMs_Custom(t *testing.T) {
	cfg := &kafka.KafkaConfig{
		BootstrapServers: []string{"broker1:9092"},
		RequestTimeoutMs: 60000,
	}
	assert.Equal(t, 60000, cfg.EffectiveRequestTimeoutMs())
}

// KafkaConfig_EffectiveMaxMessageBytes_DefaultがMaxMessageBytes未設定時にデフォルト値を返すことを検証する。
func TestKafkaConfig_EffectiveMaxMessageBytes_Default(t *testing.T) {
	cfg := &kafka.KafkaConfig{
		BootstrapServers: []string{"broker1:9092"},
	}
	assert.Equal(t, kafka.DefaultMaxMessageBytes, cfg.EffectiveMaxMessageBytes())
}

// KafkaConfig_EffectiveMaxMessageBytes_CustomがMaxMessageBytes設定時にカスタム値を返すことを検証する。
func TestKafkaConfig_EffectiveMaxMessageBytes_Custom(t *testing.T) {
	cfg := &kafka.KafkaConfig{
		BootstrapServers: []string{"broker1:9092"},
		MaxMessageBytes:  2000000,
	}
	assert.Equal(t, 2000000, cfg.EffectiveMaxMessageBytes())
}

// KafkaConfig_Validate_NegativeConnectionTimeoutが負のConnectionTimeoutMsでバリデーションエラーを返すことを検証する。
func TestKafkaConfig_Validate_NegativeConnectionTimeout(t *testing.T) {
	cfg := &kafka.KafkaConfig{
		BootstrapServers:    []string{"broker1:9092"},
		ConnectionTimeoutMs: -1,
	}
	assert.Error(t, cfg.Validate())
}

// KafkaConfig_Validate_NegativeRequestTimeoutが負のRequestTimeoutMsでバリデーションエラーを返すことを検証する。
func TestKafkaConfig_Validate_NegativeRequestTimeout(t *testing.T) {
	cfg := &kafka.KafkaConfig{
		BootstrapServers: []string{"broker1:9092"},
		RequestTimeoutMs: -1,
	}
	assert.Error(t, cfg.Validate())
}

// KafkaConfig_Validate_NegativeMaxMessageBytesが負のMaxMessageBytesでバリデーションエラーを返すことを検証する。
func TestKafkaConfig_Validate_NegativeMaxMessageBytes(t *testing.T) {
	cfg := &kafka.KafkaConfig{
		BootstrapServers: []string{"broker1:9092"},
		MaxMessageBytes:  -1,
	}
	assert.Error(t, cfg.Validate())
}

// KafkaConfig_AllFieldsが全フィールドを設定した場合にバリデーション・TLS判定・各有効値取得が正常に動作することを検証する。
func TestKafkaConfig_AllFields(t *testing.T) {
	cfg := &kafka.KafkaConfig{
		BootstrapServers:    []string{"broker1:9092", "broker2:9092"},
		SecurityProtocol:    "SASL_SSL",
		SASLMechanism:       "SCRAM-SHA-256",
		SASLUsername:         "user",
		SASLPassword:         "pass",
		ConsumerGroup:       "test-group",
		ConnectionTimeoutMs: 10000,
		RequestTimeoutMs:    60000,
		MaxMessageBytes:     2000000,
	}
	assert.NoError(t, cfg.Validate())
	assert.True(t, cfg.UsesTLS())
	assert.Equal(t, "broker1:9092,broker2:9092", cfg.BootstrapServersString())
	assert.Equal(t, 10000, cfg.EffectiveConnectionTimeoutMs())
	assert.Equal(t, 60000, cfg.EffectiveRequestTimeoutMs())
	assert.Equal(t, 2000000, cfg.EffectiveMaxMessageBytes())
}

// --- KafkaError Tests ---

// KafkaError_ErrorWithWrappedErrorがラップエラーを持つKafkaErrorのErrorメソッドが操作名・メッセージ・原因エラーを含む文字列を返すことを検証する。
func TestKafkaError_ErrorWithWrappedError(t *testing.T) {
	cause := errors.New("connection refused")
	err := &kafka.KafkaError{
		Op:      "connect",
		Message: "failed to connect to broker",
		Err:     cause,
	}
	assert.Contains(t, err.Error(), "connect")
	assert.Contains(t, err.Error(), "failed to connect to broker")
	assert.Contains(t, err.Error(), "connection refused")
}

// KafkaError_ErrorWithoutWrappedErrorがラップエラーなしのKafkaErrorが「op: message」形式の文字列を返すことを検証する。
func TestKafkaError_ErrorWithoutWrappedError(t *testing.T) {
	err := &kafka.KafkaError{
		Op:      "publish",
		Message: "topic not found",
	}
	assert.Equal(t, "publish: topic not found", err.Error())
}

// KafkaError_UnwrapがUnwrapでラップされた原因エラーを取り出せることを検証する。
func TestKafkaError_Unwrap(t *testing.T) {
	cause := errors.New("timeout")
	err := &kafka.KafkaError{
		Op:      "subscribe",
		Message: "subscription failed",
		Err:     cause,
	}
	assert.ErrorIs(t, err, cause)
}

// KafkaError_UnwrapNilがErrフィールドなしのKafkaErrorのUnwrapがnilを返すことを検証する。
func TestKafkaError_UnwrapNil(t *testing.T) {
	err := &kafka.KafkaError{
		Op:      "subscribe",
		Message: "subscription failed",
	}
	assert.Nil(t, err.Unwrap())
}

// --- TopicPartitionInfo Tests ---

// TopicPartitionInfo_FieldsがTopicPartitionInfoの全フィールドが正しく設定・取得できることを検証する。
func TestTopicPartitionInfo_Fields(t *testing.T) {
	info := &kafka.TopicPartitionInfo{
		Topic:     "k1s0.system.auth.login.v1",
		Partition: 0,
		Leader:    1,
		Replicas:  []int32{1, 2, 3},
		ISR:       []int32{1, 2},
	}
	assert.Equal(t, "k1s0.system.auth.login.v1", info.Topic)
	assert.Equal(t, int32(0), info.Partition)
	assert.Equal(t, int32(1), info.Leader)
	assert.Equal(t, []int32{1, 2, 3}, info.Replicas)
	assert.Equal(t, []int32{1, 2}, info.ISR)
}

// TopicPartitionInfo_EmptyReplicasがレプリカ未設定のTopicPartitionInfoのReplicasとISRがnilであることを検証する。
func TestTopicPartitionInfo_EmptyReplicas(t *testing.T) {
	info := &kafka.TopicPartitionInfo{
		Topic:     "k1s0.system.auth.login.v1",
		Partition: 0,
		Leader:    1,
	}
	assert.Nil(t, info.Replicas)
	assert.Nil(t, info.ISR)
}

// --- DefaultPartitionsForTier Tests ---

// DefaultPartitionsForTier_SystemがSystemティアのデフォルトパーティション数が6であることを検証する。
func TestDefaultPartitionsForTier_System(t *testing.T) {
	assert.Equal(t, 6, kafka.DefaultPartitionsForTier("system"))
}

// DefaultPartitionsForTier_BusinessがBusinessティアのデフォルトパーティション数が6であることを検証する。
func TestDefaultPartitionsForTier_Business(t *testing.T) {
	assert.Equal(t, 6, kafka.DefaultPartitionsForTier("business"))
}

// DefaultPartitionsForTier_ServiceがServiceティアのデフォルトパーティション数が3であることを検証する。
func TestDefaultPartitionsForTier_Service(t *testing.T) {
	assert.Equal(t, 3, kafka.DefaultPartitionsForTier("service"))
}

// DefaultPartitionsForTier_Unknownが未知のティアに対してデフォルトパーティション数3を返すことを検証する。
func TestDefaultPartitionsForTier_Unknown(t *testing.T) {
	assert.Equal(t, 3, kafka.DefaultPartitionsForTier("other"))
}

// --- WithTierDefaults Tests ---

// TopicConfig_WithTierDefaults_SystemがSystemティアのトピックにデフォルトパーティション数6を設定することを検証する。
func TestTopicConfig_WithTierDefaults_System(t *testing.T) {
	tc := &kafka.TopicConfig{Name: "k1s0.system.auth.login.v1"}
	tc.WithTierDefaults()
	assert.Equal(t, 6, tc.Partitions)
}

// TopicConfig_WithTierDefaults_BusinessがBusinessティアのトピックにデフォルトパーティション数6を設定することを検証する。
func TestTopicConfig_WithTierDefaults_Business(t *testing.T) {
	tc := &kafka.TopicConfig{Name: "k1s0.business.order.placed.v1"}
	tc.WithTierDefaults()
	assert.Equal(t, 6, tc.Partitions)
}

// TopicConfig_WithTierDefaults_ServiceがServiceティアのトピックにデフォルトパーティション数3を設定することを検証する。
func TestTopicConfig_WithTierDefaults_Service(t *testing.T) {
	tc := &kafka.TopicConfig{Name: "k1s0.service.activity.approved.v1"}
	tc.WithTierDefaults()
	assert.Equal(t, 3, tc.Partitions)
}

// TopicConfig_WithTierDefaults_InvalidNameが無効なトピック名の場合にパーティション数が変更されないことを検証する。
func TestTopicConfig_WithTierDefaults_InvalidName(t *testing.T) {
	tc := &kafka.TopicConfig{Name: "invalid", Partitions: 5}
	tc.WithTierDefaults()
	assert.Equal(t, 5, tc.Partitions) // 変更されないこと
}
