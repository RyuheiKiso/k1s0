// config.rs — k1s0 tier1 Library: フィーチャーフラグ操作の L2* facade trait
// flagd/OpenFeature 等の OSS 型を公開 API に露出しない（L2* ラップ規約）。
// bool / string の 2 プリミティブ型のみを公開 API として提供する。
// 全 trait は Send + Sync を要求する（スレッド安全性の強制）。

// async_trait: async fn in trait を stable で使用するためのマクロ
use async_trait::async_trait;

// FlagdClient は flagd/OpenFeature を L2* ラップするフィーチャーフラグ操作 facade trait。
// 公開 API シグネチャに OSS 型（openfeature::Client 等）を一切含まない。
// フラグ評価失敗時はデフォルト値を返す（エラー伝播禁止: フラグ評価はベストエフォート）。
#[async_trait]
pub trait FlagdClient: Send + Sync {
    // bool_flag はブール型フラグを評価する。
    // フラグが存在しない場合または評価失敗時は default 値を返す。
    async fn bool_flag(&self, key: &str, default: bool) -> bool;

    // string_flag は文字列型フラグを評価する。
    // フラグが存在しない場合または評価失敗時は default 値を返す。
    async fn string_flag(&self, key: &str, default: &str) -> String;
}
