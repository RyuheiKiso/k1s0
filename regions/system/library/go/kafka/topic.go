package kafka

import (
	"fmt"
	"regexp"
)

// トピック名のバリデーション正規表現
// パターン: k1s0.(system|business|service).<domain>.<event>.v<version>
var topicNameRegex = regexp.MustCompile(`^k1s0\.(system|business|service)\.[a-z0-9-]+\.[a-z0-9-]+\.v[0-9]+$`)

// TopicPartitionInfo はトピックのパーティション情報を表す。
type TopicPartitionInfo struct {
	// Topic はトピック名。
	Topic string
	// Partition はパーティション番号。
	Partition int32
	// Leader はリーダーブローカーの ID。
	Leader int32
	// Replicas はレプリカが配置されているブローカー ID のリスト。
	Replicas []int32
	// ISR (In-Sync Replicas) は同期しているレプリカのブローカー ID のリスト。
	ISR []int32
}

// TopicConfig は Kafka トピック設定。
type TopicConfig struct {
	// Name はトピック名。
	Name string
	// Partitions はパーティション数。
	Partitions int
	// ReplicationFactor はレプリケーション係数。
	ReplicationFactor int
	// RetentionMs はメッセージ保持期間 (ミリ秒)。
	RetentionMs int64
}

// ValidateName はトピック名を検証する。
// 正しい形式: k1s0.(system|business|service).<domain>.<event>.v<version>
func (t *TopicConfig) ValidateName() error {
	if t.Name == "" {
		return fmt.Errorf("topic name must not be empty")
	}
	if !topicNameRegex.MatchString(t.Name) {
		return fmt.Errorf("invalid topic name: %s (expected format: k1s0.(system|business|service).<domain>.<event>.v<version>)", t.Name)
	}
	return nil
}

// Tier はトピック名からティアを返す (system, business, service)。
func (t *TopicConfig) Tier() string {
	matches := topicNameRegex.FindStringSubmatch(t.Name)
	if len(matches) < 2 {
		return ""
	}
	return matches[1]
}
