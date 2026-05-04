// 本ファイルは PubSub の Dead Letter Queue (DLQ) 経路ヘルパ。
//
// 設計正典:
//   docs/03_要件定義/20_機能要件/10_tier1_API要件/03_PubSub_API.md (FR-T1-PUBSUB-004)
//   docs/03_要件定義/20_機能要件/40_tier1_API契約IDL/03_PubSub_API.md
//
// 役割:
//   FR-T1-PUBSUB-004「handler が連続 N 回失敗したイベントは自動的に
//   `k1s0.<tenant_id>.dlq.<service_name>` トピックへ転送される」を満たすため、
//   tier1 Subscribe handler が adapter に渡す DeadLetterTopic を統一規則で生成する。
//
//   実際の retry 上限カウントと DLQ 転送は Dapr Pub/Sub Component の標準機能
//   (`consumeRetryMax` + Subscription metadata `deadLetterTopic`) が担う。tier1 は
//   topic 名規則の統一に責務を絞ることで、Component 個別の DLQ topic 設定が
//   テナント / サービスごとにバラつくことを構造的に防止する。
//
// 関連要件:
//   FR-T1-PUBSUB-003 (Consumer Group 自動付与) — service_name は consumer_group から導出
//   FR-T1-PUBSUB-004 (Dead Letter Queue)
//
// 設定の流れ:
//   handler.Subscribe()
//     → normalizeConsumerGroup(tenantID, raw) で `k1s0.<tenant>.<service>` 形式に統一
//     → serviceNameFromConsumerGroup(...) で service 部を抽出
//     → dlqTopicName(...) で `k1s0.<tenant>.dlq.<service>` 生成
//     → SubscribeAdapterRequest.DeadLetterTopic に詰めて adapter へ
//     → Dapr SDK Subscription metadata `deadLetterTopic` に注入される

package state

// pubsubDLQMaxRetries は FR-T1-PUBSUB-004 受け入れ基準「連続 3 回失敗」。
// Component YAML (`infra/dapr/components/pubsub/kafka.yaml` の consumeRetryMax) と
// 値を揃えて運用する。本定数はコード側の正典で、Component 設定との同期は
// Test 24 (`tests/audit/test_audit_lib.sh`) で機械検証される。
const pubsubDLQMaxRetries = 3

// pubsubDLQTopicSegment は DLQ topic 名の中間セグメント。
// `k1s0.<tenant_id>.dlq.<service_name>` の `.dlq.` 部分。
const pubsubDLQTopicSegment = ".dlq."

// pubsubDLQMetaKey は Dapr Pub/Sub Subscription metadata 上の DLQ topic 指定キー。
// Dapr 1.x の標準キー名 `deadLetterTopic` に整合させる。
const pubsubDLQMetaKey = "deadLetterTopic"

// dlqTopicName は FR-T1-PUBSUB-004 の DLQ topic 名規則
// `k1s0.<tenant_id>.dlq.<service_name>` を生成する。
//
// 入力前提:
//   - tenantID: NFR-E-AC-003 検証済の確定 tenant_id（呼出側自己宣言ではない）
//   - serviceName: serviceNameFromConsumerGroup() で抽出した値（空文字なし）
//
// いずれかが空の場合は空文字を返し、呼出側は DeadLetterTopic を adapter に
// 渡さない（Dapr 側で DLQ 機能を発火させない）運用とする。誤って空 prefix の
// `k1s0..dlq.` 形式の topic 名で Kafka rejected を起こす経路を構造的に防ぐ。
func dlqTopicName(tenantID, serviceName string) string {
	// tenantID 空は越境防止の handler 段で既に弾かれているはずだが、
	// 万一の defense-in-depth として本関数でも空文字を返す。
	if tenantID == "" {
		return ""
	}
	// serviceName 空は consumer_group の規則違反（normalizeConsumerGroup が
	// 通過した値なら必ず非空）。caller が誤って空を渡した場合は DLQ topic を
	// 生成しない (adapter には DeadLetterTopic="" が渡り、Dapr 側で metadata
	// 注入をスキップする)。
	if serviceName == "" {
		return ""
	}
	// `k1s0.<tenant>.dlq.<service>` を組み立てる。
	return "k1s0." + tenantID + pubsubDLQTopicSegment + serviceName
}

// serviceNameFromConsumerGroup は consumer_group `k1s0.<tenant>.<service_name>` から
// `<service_name>` 部のみを抽出する。FR-T1-PUBSUB-003 の normalizeConsumerGroup を
// 通過した値を入力に取る前提。
//
// 形式不一致（prefix が一致しない / 長さが足りない）の場合は空文字を返す。
// 呼出側は空文字判定で「DLQ topic 生成不能」とみなし、DLQ を有効化しない。
func serviceNameFromConsumerGroup(tenantID, cg string) string {
	// 期待 prefix `k1s0.<tenant>.` を組み立てる。
	expected := "k1s0." + tenantID + "."
	// prefix 一致しない consumer_group は service 抽出不能。
	if !hasASCIIPrefix(cg, expected) {
		return ""
	}
	// prefix 後ろの全文字が service_name 部。空ならば不正値（normalizeConsumerGroup
	// が事前に弾いているため通常到達しないが、defensive で空チェック）。
	suffix := cg[len(expected):]
	if suffix == "" {
		return ""
	}
	return suffix
}
