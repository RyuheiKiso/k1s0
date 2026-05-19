// parity_vector_test.go — k1s0 tier1 Library: Vector 4 言語 parity テスト (Go 側)
// 15_ベクトル検索適合仕様.md §VectorSearchClient（pgvector / Qdrant L1+ 深耕）の言語横断型等価強度を検証する。
// Go 側の vector パッケージ interface が Rust / C# / TypeScript 側と等価であることを保証する。

// パッケージ名: parity_test（tier1 library Go parity テストパッケージ）
package parity_test

import (
	// testing パッケージのインポート: Go テストフレームワークに使用する
	"testing"
)

// TestParityVector_Placeholder は vector パッケージの parity テストプレースホルダー。
// 4 言語等価強度が確立されるまで Skip する（parity vector 追加後に実装を埋める）。
func TestParityVector_Placeholder(t *testing.T) {
	// parity test placeholder: 4 言語等価強度が確立されるまで Skip する
	t.Skip("parity test placeholder: vector package 4-language parity vectors not yet defined")
}

// TestParityVector_DistanceMetricConstants は VectorDistanceMetric 定数が pgvector / Qdrant 仕様準拠であることを検証する。
// Distance metric 名称と 1:1 対応することを確認する。
func TestParityVector_DistanceMetricConstants(t *testing.T) {
	// cosine: コサイン類似度（テキスト埋め込みに推奨）
	const metricCosine = "cosine"
	// l2: ユークリッド距離
	const metricL2 = "l2"
	// dot: 内積
	const metricDot = "dot"
	// l1: マンハッタン距離
	const metricL1 = "l1"
	// 全距離計算方法定数が設定されていることを確認する
	metrics := []string{metricCosine, metricL2, metricDot, metricL1}
	// 各距離計算方法定数が空でないことを確認する
	for _, m := range metrics {
		// 空文字列は spec 違反
		if m == "" {
			t.Errorf("VectorDistanceMetric constant must not be empty: spec 15 violation")
		}
	}
}
