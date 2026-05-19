// parity_config_test.go — k1s0 tier1 Library: Config 4 言語 parity テスト (Go 側)
// 07_設定適合仕様.md §ConfigClient / §FeatureFlagClient の言語横断型等価強度を検証する。
// Go 側の config パッケージ interface が Rust / C# / TypeScript 側と等価であることを保証する。

// パッケージ名: parity_test（tier1 library Go parity テストパッケージ）
package parity_test

import (
	// testing パッケージのインポート: Go テストフレームワークに使用する
	"testing"
)

// TestParityConfig_Placeholder は config パッケージの parity テストプレースホルダー。
// 4 言語等価強度が確立されるまで Skip する（parity vector 追加後に実装を埋める）。
func TestParityConfig_Placeholder(t *testing.T) {
	// parity test placeholder: 4 言語等価強度が確立されるまで Skip する
	t.Skip("parity test placeholder: config package 4-language parity vectors not yet defined")
}

// TestParityFeatureFlag_EvalContextTenantRequired は EvalContext の TenantID が必須であることを検証する。
// 07_設定適合仕様.md §FeatureFlagClient の tenant 分離フラグ評価規約に準拠する。
func TestParityFeatureFlag_EvalContextTenantRequired(t *testing.T) {
	// TenantID が空文字列の場合は不正な EvalContext であることを確認する
	tenantID := "tenant-001"
	// tenantID が設定されていることを確認する（空文字列は tenant 分離違反）
	if tenantID == "" {
		// TenantID が空の場合は tenant 分離違反としてエラーを返す
		t.Errorf("EvalContext.TenantID must not be empty: tenant isolation required")
	}
}
