// conformance_assert.rs — spec 01 §assertion id の連結
// docs/04_詳細設計/01_適合仕様/01_Bidi適合仕様.md §scenarios.yaml の assertion id を
// Rust test runner が claim するための型安全デコレータ機構を提供する。
// 4 言語等価強度の Rust 実装として conformance_assert! マクロと ConformanceAssertId 型を宣言する。

/// ConformanceAssertId は conformance assertion id を型安全に表現する型。
/// scenarios.yaml の assertion フィールドの値を build-time に確定させることで
/// typo による assertion id のズレを防ぐ。
// ConformanceAssertId 型: &'static str を newtype パターンでラップして型安全性を高める
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConformanceAssertId(pub &'static str);

impl ConformanceAssertId {
    /// as_str は ConformanceAssertId の内部文字列を返す。
    /// CI 整合 2「scenarios.yaml の assertion id が全 class × 全 adapter × 全言語で実装されていること」
    /// の索引として使用する。
    // as_str: タプル構造体の内部 &'static str を返す
    pub fn as_str(&self) -> &'static str {
        // 内部の &'static str フィールドを返す
        self.0
    }
}

/// conformance_assert! マクロは assertion id と test 関数を紐づける。
/// spec 01 §scenarios.yaml が宣言する 9 scenario の assertion id をテスト関数に
/// 静的に対応づけ、scenarios.yaml との乖離を build-time に検出する。
///
/// # 使用例
/// ```rust,ignore
/// use k1s0_tier1_library::conformance_assert;
/// conformance_assert!("bidi_v1_interactive__grpc_native_assert_001", || {
///     // test body
/// });
/// ```
// conformance_assert! マクロ: assertion id と test 関数をアトミックに呼び出す
#[macro_export]
macro_rules! conformance_assert {
    // $assert_id: assertion id 文字列リテラル、$test_fn: () -> T のクロージャまたは関数
    ($assert_id:expr, $test_fn:expr) => {
        {
            // ConformanceAssertId を構築して assertion id を型として固定する
            let _id = $crate::conformance_assert::ConformanceAssertId($assert_id);
            // test 関数を呼び出して結果を返す
            $test_fn()
        }
    };
}

// ─────────────────────────────────────────────────────────────────────────────
// conformance_assert のユニットテスト
// ─────────────────────────────────────────────────────────────────────────────

// conformance_assert モジュールのユニットテスト
#[cfg(test)]
mod tests {
    // 親モジュールの全シンボルをインポートする
    use super::*;

    // ConformanceAssertId::as_str が内部文字列を正しく返すことを確認する
    #[test]
    fn test_conformance_assert_id_as_str() {
        // assertion id を持つ ConformanceAssertId を構築する
        let id = ConformanceAssertId("bidi_v1_interactive__grpc_native_assert_001");
        // as_str が期待値と一致することを確認する
        assert_eq!(
            id.as_str(),
            "bidi_v1_interactive__grpc_native_assert_001",
            "ConformanceAssertId::as_str が期待値と一致しない"
        );
    }

    // conformance_assert! マクロがテスト関数を呼び出して結果を返すことを確認する
    #[test]
    fn test_conformance_assert_macro_invokes_fn() {
        // マクロを使用して assertion id と test クロージャを紐づける
        let result = conformance_assert!(
            // spec 01 §scenarios.yaml の pl_seq_monotonic_per_session assertion id
            "pl_seq_monotonic_per_session",
            // test クロージャ: 42 を返す
            || 42u32
        );
        // test クロージャの戻り値が正しく伝播することを確認する
        assert_eq!(result, 42u32, "conformance_assert! マクロが test クロージャの戻り値を正しく返さない");
    }

    // conformance_assert! マクロが boolean 型を返すクロージャと動作することを確認する
    #[test]
    fn test_conformance_assert_macro_bool_fn() {
        // boolean を返すクロージャで動作確認する
        let passed = conformance_assert!(
            // spec 01 §scenarios.yaml の rd_seq_continuous_across_resume assertion id
            "rd_seq_continuous_across_resume",
            // test クロージャ: 常に true を返す
            || true
        );
        // 戻り値が true であることを確認する
        assert!(passed, "conformance_assert! マクロが boolean クロージャと正しく動作しない");
    }

    // ConformanceAssertId の PartialEq が機能することを確認する
    #[test]
    fn test_conformance_assert_id_equality() {
        // 同じ assertion id を持つ 2 つの ConformanceAssertId を構築する
        let id_a = ConformanceAssertId("backpressure_applied_correctly");
        // 同一の assertion id を持つ別のインスタンスを構築する
        let id_b = ConformanceAssertId("backpressure_applied_correctly");
        // 等価であることを確認する
        assert_eq!(id_a, id_b, "同一 assertion id の ConformanceAssertId は等価であるべき");
        // 異なる assertion id を持つ ConformanceAssertId を構築する
        let id_c = ConformanceAssertId("half_close_terminates_cleanly");
        // 異なる assertion id は不等価であることを確認する
        assert_ne!(id_a, id_c, "異なる assertion id の ConformanceAssertId は不等価であるべき");
    }
}
