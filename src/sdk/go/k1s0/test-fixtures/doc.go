// Package testfixtures は k1s0 Go SDK と同梱される e2e test 用 fixture ライブラリ。
//
// 設計正典:
//   ADR-TEST-010（test-fixtures 4 言語 SDK 同梱、ADR-TEST-008 user suite を補完）
//   docs/05_実装/30_CI_CD設計/35_e2e_test_design/30_test_fixtures/01_4言語対称API.md
//
// 利用者の使い方:
//
//	import testfixtures "github.com/k1s0/k1s0/src/sdk/go/k1s0/test-fixtures"
//
//	func TestMyApp(t *testing.T) {
//	    fx := testfixtures.Setup(t, testfixtures.Options{
//	        KindNodes: 2,
//	        Stack:     testfixtures.MinimumStack,
//	    })
//	    client := fx.NewSDKClient(t, "tenant-a")
//	    // 利用者の test code
//	}
//
// バージョニング:
//   本 package は src/sdk/go/k1s0/ と同 module / 同 SemVer / 同 release tag で出る
//   （ADR-TEST-010 §2 versioning）。利用者は SDK 本体を pin した version で
//   test-fixtures も自動的に同 version を取得する。
//
// リリース時点の段階展開:
//   - 領域 1 (Setup/Teardown): skeleton + 採用初期で kind 起動 spawn
//   - 領域 2 (SDK client init): skeleton + 採用初期で k1s0.Client wrapper
//   - 領域 3 (Mock builder): State / Audit / PubSub の 3 service 先行（panic で他は警告）
//   - 領域 4 (Wait/Assertion): skeleton + 採用初期で k8s client-go 統合
//   - 領域 5 (Playwright): TS のみで提供（本 Go package では非対応）
package testfixtures
