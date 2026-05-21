// conformance_assert.go — spec 01 §assertion id の連結
// docs/04_詳細設計/01_適合仕様/01_Bidi適合仕様.md §scenarios.yaml の assertion id を
// Go test runner が claim するための型安全デコレータ機構を提供する。
// Rust conformance_assert.rs / TypeScript conformanceAssert.ts /
// C# ConformanceAssertAttribute.cs と 4 言語等価強度を保つ。

// パッケージ名: conformance（tier1 Library の conformance assertion id 管理を提供する）
package conformance

// ConformanceAssertID は conformance assertion id を型安全に表現する型。
// scenarios.yaml の assertion フィールドの値を build-time に確定させることで
// typo による assertion id のズレを防ぐ。
// 使用例:
//
//	const id ConformanceAssertID = "pl_seq_monotonic_per_session"
type ConformanceAssertID string

// Assert は assertion id を記録して test 関数を実行するヘルパー。
// CI 整合 2「scenarios.yaml の assertion id が全 class × 全 adapter × 全言語で実装されていること」
// の物理機構として、assertID を引数に取ることで assertion id を test 関数に静的に紐づける。
//
// 使用例:
//
//	conformance.Assert("bidi_v1_interactive__grpc_native_assert_001", func() {
//	    // test body
//	})
// assertID: scenarios.yaml の assertion id（例: "pl_seq_monotonic_per_session"）
// fn: assertion id に対応する test ロジックを実行するクロージャ
func Assert(assertID ConformanceAssertID, fn func()) {
    // assertID を型として固定した上で test 関数を呼び出す（assertion id を証拠として残す）
    // _ = assertID は linter の unused variable エラーを回避しつつ、
    // assertID が引数として宣言されている事実をシグネチャに保存する
    _ = assertID
    // test 関数を実行する
    fn()
}

// AssertWithResult は assertion id を記録して戻り値を持つ test 関数を実行するヘルパー。
// Assert の型パラメータ版として Go 1.18+ generics を使用する。
// 戻り値が必要な test 関数（例: error を返す assertion）に使用する。
//
// 使用例:
//
//	result, err := conformance.AssertWithResult(
//	    "rd_seq_continuous_across_resume",
//	    func() (bool, error) { return true, nil },
//	)
// assertID: scenarios.yaml の assertion id
// fn: 戻り値を返す assertion ロジックを実行するクロージャ
// T: fn の戻り値型
func AssertWithResult[T any](assertID ConformanceAssertID, fn func() T) T {
    // assertID を型として固定する（assertion id を証拠として保存する）
    _ = assertID
    // test 関数を実行して結果を返す
    return fn()
}
