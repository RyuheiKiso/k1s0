// 本ファイルは tier1 ServiceInvoke API の統合テスト雛形。
// testcontainers で Dapr sidecar を立ち上げ、ServiceInvoke エンドポイントを叩く。
//
// docs 正典: docs/05_実装/00_ディレクトリ設計/70_共通資産/02_tests配置.md
//
// リリース時点 では本シナリオは t.Skip で stub 化し、採用初期 で実装する。
package tier1facade

import "testing"

// ServiceInvoke API を Dapr sidecar 経由で呼び出し、応答を確認する統合テスト。
// 実装時は tests/integration/compose/dapr-compose.yaml を testcontainers で
// 起動してから tier1 facade の gRPC エンドポイントを叩く。
func TestServiceInvokeIntegration(t *testing.T) {
	t.Skip("TODO(release-initial): testcontainers + Dapr sidecar 統合テストを実装する")
}
