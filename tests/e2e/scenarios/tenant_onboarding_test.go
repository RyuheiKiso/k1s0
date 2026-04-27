// 本ファイルはテナント新規オンボーディング E2E シナリオの雛形。
// 設計正典: docs/05_実装/00_ディレクトリ設計/70_共通資産/02_tests配置.md
//
// リリース時点 段階では各ステップを t.Skip で stub 化し、採用初期 で実装する。
// 実装時は kind cluster + infra/environments/dev/ を up してから実行する。
package scenarios

import "testing"

// テナント作成 → ユーザ登録 → 初回ログイン → ダッシュボード取得の一連フロー。
// 全ステップが完了するまで他テストと並列実行できないため t.Parallel() は使わない。
func TestTenantOnboarding(t *testing.T) {
	// 採用初期 で helpers.SetupCluster(t) / AuthenticateAsAdmin / CreateTenant /
	// RegisterUser / AuthenticateAsUser / GetDashboard を実装する。
	t.Skip("TODO(release-initial): tenant onboarding e2e flow を実装する")
}
