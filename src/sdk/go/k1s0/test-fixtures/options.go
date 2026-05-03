// src/sdk/go/k1s0/test-fixtures/options.go
//
// k1s0 Go SDK test-fixtures: Options 構造定義。
// 4 言語対称 API の Options field 名を Go イディオムで実装する。
//
// 設計正典:
//   ADR-TEST-010（test-fixtures 4 言語 SDK 同梱）
//   docs/05_実装/30_CI_CD設計/35_e2e_test_design/30_test_fixtures/01_4言語対称API.md
package testfixtures

// Stack は kind cluster に install する k1s0 stack の規模を表す
type Stack int

const (
	// MinimumStack は Dapr + tier1 facade + Keycloak + 1 backend のみ install
	MinimumStack Stack = iota
	// FullStack は user suite 任意 stack 全部入り（owner 経路ではない）
	FullStack
)

// Options は Setup の動作を制御するパラメータ。
// 4 言語対称化のため field 名は PascalCase で固定（Go: PascalCase, Rust: snake_case, etc）。
type Options struct {
	// KindNodes は kind cluster の node 数（control-plane + worker、既定 2）
	KindNodes int
	// Stack は install する k1s0 stack（既定 MinimumStack）
	Stack Stack
	// AddOns は Setup 時に追加で install する任意 component の名前一覧
	// 既定 minimum stack に追加で workflow / decision / strimzi 等を opt-in する
	AddOns []string
	// Tenant はデフォルトの tenant ID（既定 "tenant-a"）
	Tenant string
	// Namespace は k1s0 install 先 namespace（既定 "k1s0"）
	Namespace string
}

// DefaultOptions は試験で典型的な Options を返す
func DefaultOptions() Options {
	return Options{
		// kind 既定 2 node（CP1 + W1）
		KindNodes: 2,
		// minimum stack（最小成立形）
		Stack: MinimumStack,
		// AddOns は空（opt-in なし）
		AddOns: nil,
		// 既定 tenant 名
		Tenant: "tenant-a",
		// 既定 namespace
		Namespace: "k1s0",
	}
}
