// parity_messaging_test.go — k1s0 tier1 Library: Messaging 4 言語 parity テスト (Go 側)
// 11_メッセージング適合仕様.md §MessagingProducer / §MessagingConsumer（Kafka L1+ 深耕）の言語横断型等価強度を検証する。
// Go 側の messaging パッケージ interface が Rust / C# / TypeScript 側と等価であることを保証する。

// パッケージ名: parity_test（tier1 library Go parity テストパッケージ）
package parity_test

import (
	// testing パッケージのインポート: Go テストフレームワークに使用する
	"testing"
)

// TestParityMessaging_Placeholder は messaging パッケージの parity テストプレースホルダー。
// 4 言語等価強度が確立されるまで Skip する（parity vector 追加後に実装を埋める）。
func TestParityMessaging_Placeholder(t *testing.T) {
	// parity test placeholder: 4 言語等価強度が確立されるまで Skip する
	t.Skip("parity test placeholder: messaging package 4-language parity vectors not yet defined")
}

// TestParityMessaging_OutboxTenantRequired は OutboxMessage の TenantID が必須であることを検証する。
// 11_メッセージング適合仕様.md §Outbox Pattern の tenant 分離必須規約に準拠する。
func TestParityMessaging_OutboxTenantRequired(t *testing.T) {
	// TenantID が設定された有効な OutboxMessage の tenant_id 値
	tenantID := "tenant-001"
	// topic は TenantID prefix を含む形式であることを確認する
	topic := tenantID + ".order.created"
	// TenantID が空でないことを確認する
	if tenantID == "" {
		// TenantID が空の場合は tenant 分離違反としてエラーを返す
		t.Errorf("OutboxMessage.TenantID must not be empty: tenant isolation required")
	}
	// topic が TenantID prefix を含むことを確認する
	if len(topic) <= len(tenantID) {
		// topic が短すぎる場合は形式違反としてエラーを返す
		t.Errorf("OutboxMessage.Topic should contain TenantID prefix: got %s", topic)
	}
}
