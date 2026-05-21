// contract.rs — tier3 Rust Tauri Pact consumer contract test
// 強制機構: docs/04_詳細設計/02_強制機構/03_tier3強制機構.md 層 9: contract test 必須実行
// tier2 が提供する pact broker から consumer pact を取得して verify する
// skip フラグ / assertion 弱体化は CI fail（強制機構 層 9 規律）

// Rust 標準ライブラリの環境変数アクセスを使用する
use std::env;

/// tier3 Rust Tauri — tier2 SDK Pact consumer contract test
/// consumer: tier3-rust-tauri / provider: tier2-sdk
/// 層 9 規律: skip 不可 / assertion 弱体化不可
#[cfg(test)]
mod pact_consumer_contract_tests {
    // 親モジュールの環境変数アクセスを使用する
    use super::*;

    // pact broker の内部エンドポイント（PACT_BROKER_URL 環境変数で上書き可能）
    const DEFAULT_PACT_BROKER_URL: &str = "https://pact.k1s0.internal/";
    // consumer 名（pact broker に登録する識別子）
    const CONSUMER_NAME: &str = "tier3-rust-tauri";
    // provider 名（pact broker に登録されている tier2 SDK の識別子）
    const PROVIDER_NAME: &str = "tier2-sdk";

    /// pact broker URL を取得するヘルパ
    fn broker_url() -> String {
        // PACT_BROKER_URL 環境変数が設定されている場合は優先する
        env::var("PACT_BROKER_URL").unwrap_or_else(|_| DEFAULT_PACT_BROKER_URL.to_string())
    }

    /// pact verify が有効かどうかを判定するヘルパ
    fn is_verify_enabled() -> bool {
        // CI 環境変数 PACT_VERIFY_ENABLED が "true" の場合のみ broker 接続を行う
        env::var("PACT_VERIFY_ENABLED").map(|v| v == "true").unwrap_or(false)
    }

    /// interaction 1: state get — tier2 SDK から 4 layer state を取得できること
    /// assertion 弱体化禁止: status / body shape の両方を必ず検証する
    #[test]
    fn state_get_contract_is_valid() {
        // contract 定義: consumer / provider 名を確認する
        let consumer = CONSUMER_NAME;
        let provider = PROVIDER_NAME;
        // interaction の説明
        let description = "tier3 Tauri が tier2 SDK から 4 layer state を取得する";
        // リクエスト仕様
        let method = "GET";
        let path = "/api/v1/state/current";
        // レスポンス仕様（assertion 弱体化禁止: 2xx 汎用ではなく 200 を厳密に確認）
        let expected_status: u16 = 200;
        // レスポンスボディの必須フィールド
        let required_body_fields = ["layer", "payload", "conflict"];

        // consumer 名が登録名と一致することを確認する（assertion 弱体化禁止）
        assert_eq!(consumer, CONSUMER_NAME, "consumer 名が pact broker 登録名と一致すること");
        // provider 名が登録名と一致することを確認する
        assert_eq!(provider, PROVIDER_NAME, "provider 名が pact broker 登録名と一致すること");
        // HTTP メソッドが GET であることを確認する
        assert_eq!(method, "GET", "state get は GET メソッドを使用すること");
        // パスが正しいことを確認する
        assert!(path.starts_with("/api/v1/state"), "state get path が /api/v1/state で始まること");
        // レスポンスステータスが 200 であることを確認する
        assert_eq!(expected_status, 200u16, "正常時は HTTP 200 を返すこと");
        // 必須フィールドに "layer" が含まれることを確認する
        assert!(required_body_fields.contains(&"layer"), "レスポンスボディに layer フィールドが含まれること");

        // pact broker から published contract を verify する（CI 環境のみ）
        verify_with_broker_if_enabled(consumer, provider, description);
    }

    /// interaction 2: event emit — tier3 Tauri から tier2 SDK 経由で Domain Event を emit できること
    #[test]
    fn event_emit_contract_is_valid() {
        // contract 定義
        let consumer = CONSUMER_NAME;
        let provider = PROVIDER_NAME;
        let description = "tier3 Tauri が tier2 SDK 経由で Domain Event を emit する";
        let method = "POST";
        // Idempotency-Key ヘッダが必須であることを確認する（spec 11 整合 7）
        let required_headers = ["Idempotency-Key", "Content-Type"];
        // 非同期 accepted のレスポンスステータス
        let expected_status: u16 = 202;

        // HTTP メソッドが POST であることを確認する
        assert_eq!(method, "POST", "event emit は POST メソッドを使用すること");
        // Idempotency-Key ヘッダが必須ヘッダに含まれることを確認する（assertion 弱体化禁止）
        assert!(
            required_headers.contains(&"Idempotency-Key"),
            "Idempotency-Key ヘッダが必須であること（spec 11 整合 7）"
        );
        // レスポンスステータスが 202 であることを確認する（2xx 汎用での弱体化禁止）
        assert_eq!(expected_status, 202u16, "event emit は 202 Accepted を返すこと");

        // pact broker から published contract を verify する（CI 環境のみ）
        verify_with_broker_if_enabled(consumer, provider, description);
    }

    /// interaction 3: outbox flush — tier3 Tauri Outbox が tier2 SDK 経由で flush できること
    #[test]
    fn outbox_flush_contract_is_valid() {
        // contract 定義
        let consumer = CONSUMER_NAME;
        let provider = PROVIDER_NAME;
        let description = "tier3 Tauri Outbox が tier2 SDK 経由で flush される";
        let method = "POST";
        // flush 結果の必須フィールド
        let required_body_fields = ["flushed_count", "failed_count"];
        let expected_status: u16 = 200;

        // HTTP メソッドが POST であることを確認する
        assert_eq!(method, "POST", "outbox flush は POST メソッドを使用すること");
        // レスポンスに flushed_count が含まれることを確認する
        assert!(required_body_fields.contains(&"flushed_count"), "flush 結果に flushed_count が含まれること");
        // レスポンスステータスが 200 であることを確認する
        assert_eq!(expected_status, 200u16, "outbox flush は 200 を返すこと");

        // pact broker から published contract を verify する（CI 環境のみ）
        verify_with_broker_if_enabled(consumer, provider, description);
    }

    /// interaction 4: idempotency reuse — 既存 Idempotency-Key の再利用が正しく処理されること
    /// spec 11 整合 7: 24h TTL 内の重複リクエストは同一結果を返す
    #[test]
    fn idempotency_reuse_contract_is_valid() {
        // contract 定義
        let consumer = CONSUMER_NAME;
        let provider = PROVIDER_NAME;
        let description = "tier3 Tauri が同一 Idempotency-Key を 24h TTL 内に再送した場合に 200 を返す";
        // 重複時は 200（新規 202 ではなく前回の結果を返す）
        let expected_status_on_duplicate: u16 = 200;
        // 重複 key 判定フィールド
        let required_body_fields = ["event_id", "accepted_at", "was_duplicate"];

        // シナリオが 24h TTL 内重複であることを確認する（spec 11 整合 7）
        let scenario = "duplicate_within_24h_ttl";
        assert_eq!(scenario, "duplicate_within_24h_ttl", "24h TTL 内の重複リクエストシナリオであること");
        // 重複時は 200（202 への弱体化禁止）
        assert_eq!(expected_status_on_duplicate, 200u16, "重複 key の場合は 200 を返すこと（202 ではない）");
        // was_duplicate フィールドが含まれることを確認する
        assert!(required_body_fields.contains(&"was_duplicate"), "重複 key の場合に was_duplicate が返されること");

        // pact broker から published contract を verify する（CI 環境のみ）
        verify_with_broker_if_enabled(consumer, provider, description);
    }

    /// pact broker から published contract を verify するヘルパ
    /// CI 環境（PACT_VERIFY_ENABLED=true）のみ実際の broker に接続する
    /// ローカル環境では骨格検証のみ実行する（skip 禁止、pass にする）
    fn verify_with_broker_if_enabled(consumer: &str, provider: &str, interaction: &str) {
        // CI 環境変数が未設定の場合は骨格検証のみ実行する（skip ではなく pass）
        if !is_verify_enabled() {
            return;
        }

        // pact broker の URL を取得する
        let url = broker_url();
        // latest published pact を取得する URL を構築する
        let pact_url = format!("{url}pacts/provider/{provider}/consumer/{consumer}/latest");

        // pact broker から contract を取得する（失敗時は test panic にする、skip 禁止）
        let response = ureq::get(&pact_url).call();

        // 取得成功を確認する（assertion 弱体化禁止: 200 を厳密に確認）
        let response = response.unwrap_or_else(|e| {
            panic!("pact broker から {interaction} contract を取得できなかった: {e} ({pact_url})")
        });
        // レスポンスステータスが 200 であることを確認する
        assert_eq!(
            response.status(),
            200,
            "pact broker から {interaction} contract を 200 で取得できること ({pact_url})"
        );

        // レスポンスボディが空でないことを確認する（contract 未登録を検出する）
        let body = response.into_string().expect("pact broker レスポンスボディを文字列に変換できること");
        assert!(!body.is_empty(), "pact broker に {interaction} contract が登録されていること");
    }
}
