// concurrency_guard.rs — aggregate_id 単位で max_one_in_flight を強制する ConcurrencyGuard（Rust 等価強度実装）
// TypeScript の withAggregateExclusivity と等価の抽象を Rust で実装する
// spec 11 §per-aggregate write 並行 1 件以下: layers.yaml invariant の max_one_in_flight_per_aggregate を物理化する

// 標準ライブラリの Mutex / HashSet / Arc をインポートする
use std::collections::HashSet;
// sync::Mutex で in_flight セットの排他制御を行う
use std::sync::Mutex;
// fmt::Display を ConcurrencyError に実装するためにインポートする
use std::fmt;

// ConcurrencyError は aggregate 単位 排他制御に関するエラー型
#[derive(Debug, PartialEq, Eq)]
pub enum ConcurrencyError {
    // 対象 aggregate が既に in-flight 状態のエラー（待機せずにリジェクトする場合に使用する）
    AggregateInFlight(String),
    // 処理自体が失敗したエラー（fn() の Result::Err を伝播する場合に使用する）
    Inner(String),
}

impl fmt::Display for ConcurrencyError {
    // Display トレイトの実装（エラーメッセージを人間が読める形式で返す）
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // エラー種別に応じたメッセージを生成する
        match self {
            // AggregateInFlight: aggregate_id を含むエラーメッセージを返す
            ConcurrencyError::AggregateInFlight(id) => {
                // aggregate_id を含むエラーメッセージを書き込む
                write!(f, "aggregate '{}' is already in flight", id)
            }
            // Inner: 内部エラーメッセージを返す
            ConcurrencyError::Inner(msg) => {
                // 内部エラーメッセージを書き込む
                write!(f, "inner error: {}", msg)
            }
        }
    }
}

// ConcurrencyGuard は aggregate_id 単位で max_one_in_flight を実現するガード構造体
// TypeScript の inFlightMap に対応する in-flight 集合を Mutex<HashSet<String>> で管理する
pub struct ConcurrencyGuard {
    // 現在 in-flight の aggregate_id の集合（Mutex で goroutine-safe に管理する）
    in_flight: Mutex<HashSet<String>>,
}

impl ConcurrencyGuard {
    // 新規の ConcurrencyGuard を生成する（in_flight は空集合で初期化する）
    pub fn new() -> Self {
        // Mutex で保護された空の HashSet を初期化する
        Self {
            in_flight: Mutex::new(HashSet::new()),
        }
    }

    // with_aggregate_exclusivity は aggregate_id で指定した aggregate に対して f を排他的に実行する
    // aggregate が既に in-flight の場合は ConcurrencyError::AggregateInFlight を返す
    // TypeScript の withAggregateExclusivity と等価（ただし Rust は待機せずリジェクトする設計）
    // aggregate_id: 排他制御の対象 aggregate の識別子
    // f: aggregate に対して排他的に実行する処理（Result<R, E> を返す）
    pub fn with_aggregate_exclusivity<F, R, E>(
        &self,
        // 排他制御の対象 aggregate の識別子を受け取る
        aggregate_id: &str,
        // 排他的に実行する処理（Result<R, E> を返すクロージャ）
        f: F,
    ) -> Result<R, ConcurrencyError>
    where
        // F は引数なしで Result<R, E> を返すクロージャ
        F: FnOnce() -> Result<R, E>,
        // E は fmt::Display トレイトを実装するエラー型
        E: fmt::Display,
    {
        // in_flight セットのロックを取得する（poisoned の場合は panic する）
        {
            // Mutex のロックを取得して in_flight セットを変更する
            let mut set = self
                .in_flight
                .lock()
                // Mutex が poisoned の場合は panic する（不変条件が壊れているため）
                .expect("ConcurrencyGuard: in_flight Mutex poisoned");
            // aggregate_id が既に in-flight の場合は AggregateInFlight エラーを返す
            if set.contains(aggregate_id) {
                // 既に in-flight のため処理を拒否する
                return Err(ConcurrencyError::AggregateInFlight(aggregate_id.to_owned()));
            }
            // in_flight セットに aggregate_id を登録する
            set.insert(aggregate_id.to_owned());
        }
        // 処理完了後に in_flight から aggregate_id を削除するガードを設定する
        // Drop による確実な削除（panic / early return でも削除を保証する）
        let guard = InFlightGuard {
            // in_flight セットへの参照を保持する
            in_flight: &self.in_flight,
            // 登録した aggregate_id を保持する
            aggregate_id: aggregate_id.to_owned(),
        };
        // 排他的な処理を実行する
        let result = f();
        // ガードを明示的にドロップして in_flight から削除する（drop が自動で呼ばれるが明示する）
        drop(guard);
        // f() の結果を ConcurrencyError::Inner にマップして返す
        result.map_err(|e| ConcurrencyError::Inner(e.to_string()))
    }

    // is_aggregate_in_flight は aggregate_id が現在 in-flight かどうかを返す
    // TypeScript の isAggregateInFlight と等価の確認関数
    pub fn is_aggregate_in_flight(&self, aggregate_id: &str) -> bool {
        // in_flight セットのロックを取得する
        let set = self
            .in_flight
            .lock()
            // Mutex が poisoned の場合は panic する
            .expect("ConcurrencyGuard: in_flight Mutex poisoned");
        // aggregate_id が in-flight セットに存在するか確認する
        set.contains(aggregate_id)
    }
}

impl Default for ConcurrencyGuard {
    // Default トレイトを new() に委譲する
    fn default() -> Self {
        // new() を呼び出して初期化する
        Self::new()
    }
}

// InFlightGuard は Drop 時に in_flight から aggregate_id を削除するガード
// 処理完了・panic・early return のいずれの場合でも確実に削除を保証する
struct InFlightGuard<'a> {
    // in_flight セットへの参照（&'a Mutex<HashSet<String>>）
    in_flight: &'a Mutex<HashSet<String>>,
    // 削除する aggregate_id
    aggregate_id: String,
}

impl<'a> Drop for InFlightGuard<'a> {
    // Drop 時に in_flight セットから aggregate_id を削除する
    fn drop(&mut self) {
        // in_flight セットのロックを取得する（poisoned の場合は abort する）
        if let Ok(mut set) = self.in_flight.lock() {
            // in_flight セットから aggregate_id を削除する
            set.remove(&self.aggregate_id);
        }
        // Mutex が poisoned の場合は黙って終了する（Drop は panic できないため）
    }
}

#[cfg(test)]
mod tests {
    // テストモジュール内部でのみ使用するインポート
    use super::*;

    #[test]
    // 同一 aggregate_id で同時に 2 件実行しようとすると AggregateInFlight エラーが返ることを確認する
    fn test_concurrent_same_aggregate_returns_in_flight_error() {
        // テスト用の ConcurrencyGuard を生成する
        let guard = ConcurrencyGuard::new();
        // 最初の処理: in_flight に登録してから処理内で再度 with_aggregate_exclusivity を呼ぶ
        let result = guard.with_aggregate_exclusivity("agg-001", || {
            // ネストした呼び出しで同一 aggregate_id を試みる（in-flight 中に再度実行しようとする）
            let nested = guard.with_aggregate_exclusivity("agg-001", || Ok::<_, String>(42));
            // ネストは AggregateInFlight エラーになることを確認する
            assert!(matches!(nested, Err(ConcurrencyError::AggregateInFlight(_))));
            // 外側の処理は成功する
            Ok::<_, String>(1)
        });
        // 外側の処理が成功することを確認する
        assert!(result.is_ok());
    }

    #[test]
    // 異なる aggregate_id は同時実行できることを確認する
    fn test_different_aggregates_can_run_concurrently() {
        // テスト用の ConcurrencyGuard を生成する
        let guard = ConcurrencyGuard::new();
        // agg-001 で処理を実行する
        let result1 = guard.with_aggregate_exclusivity("agg-001", || Ok::<_, String>(1));
        // agg-002 で処理を実行する（agg-001 とは独立して実行できる）
        let result2 = guard.with_aggregate_exclusivity("agg-002", || Ok::<_, String>(2));
        // 両方の処理が成功することを確認する
        assert!(result1.is_ok());
        assert!(result2.is_ok());
    }

    #[test]
    // 処理完了後に in_flight から aggregate_id が削除されることを確認する
    fn test_in_flight_cleared_after_completion() {
        // テスト用の ConcurrencyGuard を生成する
        let guard = ConcurrencyGuard::new();
        // 処理を実行して完了させる
        let _ = guard.with_aggregate_exclusivity("agg-003", || Ok::<_, String>(42));
        // 処理完了後は in_flight から削除されていることを確認する
        assert!(!guard.is_aggregate_in_flight("agg-003"));
    }

    #[test]
    // is_aggregate_in_flight が in-flight 中に true を返すことを確認する
    fn test_is_aggregate_in_flight_returns_true_during_execution() {
        // テスト用の ConcurrencyGuard を生成する
        let guard = ConcurrencyGuard::new();
        // with_aggregate_exclusivity の内部で is_aggregate_in_flight を確認する
        let _ = guard.with_aggregate_exclusivity("agg-004", || {
            // 処理中は in-flight が true であることを確認する
            assert!(guard.is_aggregate_in_flight("agg-004"));
            Ok::<_, String>(())
        });
        // 処理完了後は false になることを確認する
        assert!(!guard.is_aggregate_in_flight("agg-004"));
    }
}
