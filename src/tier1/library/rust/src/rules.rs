// rules.rs — k1s0 tier1 Library: ルールエンジン L1+ facade trait
// ZEN engine 等の OSS 型を公開 API に露出しない（L1+ ラップ規約）。
// ルール評価の入出力は JSON bytes で表現する（OSS 型依存を排除する）。
// 全 trait は Send + Sync を要求する（スレッド安全性の強制）。

// async_trait: async fn in trait を stable で使用するためのマクロ
use async_trait::async_trait;

// RuleEngine は ZEN engine を L1+ ラップするルール評価 facade trait。
// 公開 API シグネチャに OSS 型（zen::engine::Engine 等）を一切含まない。
// 入力と出力は JSON bytes で表現する（型安全性は呼び出し元が担保する）。
#[async_trait]
pub trait RuleEngine: Send + Sync {
    // evaluate はルールを評価して結果を返す。
    // rule_id はルールセット識別子（例: "access_control_v1" / "pricing_v2"）。
    // input は評価入力の JSON bytes（例: b"{"user_role":"admin","resource":"secret"}"）。
    // 戻り値は評価結果の JSON bytes（例: b"{"allowed":true,"reason":"role_match"}"）。
    async fn evaluate(&self, rule_id: &str, input: Vec<u8>) -> crate::Result<Vec<u8>>;
}
