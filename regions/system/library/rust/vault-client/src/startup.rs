//! サーバー起動時の Vault シークレット取得とフォールバック処理。
//!
//! このモジュールは、サーバー起動時に Vault からシークレットを取得する際の
//! フォールバック戦略を提供する。`vault_required` フラグに基づいて、
//! Vault 接続失敗時の動作を制御する。
//!
//! ## フォールバック戦略
//!
//! - `vault_required=true` (本番デフォルト): Vault 接続失敗時はエラーを返し、
//!   サーバー起動を中断する。シークレットなしでの運用はセキュリティリスク。
//! - `vault_required=false` (開発環境): Vault 接続失敗時は警告ログを出力し、
//!   空の HashMap を返す。ローカル設定ファイルの値がそのまま使用される。

use std::collections::HashMap;

use tracing::{info, warn};

use crate::client::VaultClient;
use crate::error::VaultError;

/// サーバー起動時に Vault からシークレットを取得し、取得できない場合は
/// vault_required フラグに基づいてフォールバック処理を行う。
///
/// # 引数
/// - `client`: VaultClient 実装（HttpVaultClient, InMemoryVaultClient など）
/// - `secret_path`: Vault 上のシークレットパス
/// - `keys`: 取得するシークレットのキー一覧
/// - `vault_required`: Vault 接続が必須かどうか
///
/// # 戻り値
/// - `Ok(HashMap)`: シークレットのキーと値のマップ。フォールバック時は空の HashMap。
/// - `Err(VaultError)`: vault_required=true かつ Vault 接続失敗時のエラー。
///
/// # フォールバック判定
/// - ConnectionUnavailable エラーかつ vault_required=false の場合のみフォールバック。
/// - NotFound, PermissionDenied 等のアプリケーションエラーは vault_required に関わらず
///   エラーとして返す（設定ミスの可能性があるため黙殺しない）。
pub async fn fetch_secrets_with_fallback(
    client: &dyn VaultClient,
    secret_path: &str,
    keys: &[&str],
    vault_required: bool,
) -> Result<HashMap<String, String>, VaultError> {
    // Vault からシークレットを取得する
    let secret_result = client.get_secret(secret_path).await;

    match secret_result {
        // シークレット取得成功: 指定されたキーのみを抽出して返す
        Ok(secret) => {
            info!(
                path = %secret_path,
                version = secret.version,
                "Vault からシークレットを取得しました"
            );
            let mut result = HashMap::new();
            for key in keys {
                if let Some(value) = secret.data.get(*key) {
                    result.insert((*key).to_string(), value.clone());
                }
            }
            Ok(result)
        }
        // 接続不可エラー: vault_required に基づいてフォールバック判定
        Err(VaultError::ConnectionUnavailable(ref msg)) if !vault_required => {
            warn!(
                path = %secret_path,
                error = %msg,
                "Vault 接続に失敗しました。vault_required=false のためローカル設定で続行します。\
                 本番環境では vault_required=true に設定し、Vault の可用性を確保してください。"
            );
            Ok(HashMap::new())
        }
        // その他のエラー: vault_required=true の場合または接続不可以外のエラー
        Err(e) => {
            // vault_required=false でも NotFound や PermissionDenied はフォールバックしない。
            // これらは設定ミスの可能性が高く、黙殺するとデバッグが困難になるため。
            Err(e)
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::client::InMemoryVaultClient;
    use crate::config::VaultClientConfig;
    use crate::secret::Secret;
    use chrono::Utc;

    /// テスト用のシークレットを作成するヘルパー。
    fn make_secret(path: &str) -> Secret {
        let mut data = HashMap::new();
        data.insert("database.password".to_string(), "vault-db-pass".to_string());
        data.insert("redis.password".to_string(), "vault-redis-pass".to_string());
        Secret {
            path: path.to_string(),
            data,
            version: 1,
            created_at: Utc::now(),
        }
    }

    // Vault にシークレットが存在する場合、指定キーの値が返されることを確認する。
    #[tokio::test]
    async fn fetch_secrets_success() {
        let client = InMemoryVaultClient::new();
        client.put_secret(make_secret("system/order/secrets"));

        let result = fetch_secrets_with_fallback(
            &client,
            "system/order/secrets",
            &["database.password", "redis.password"],
            true,
        )
        .await
        .expect("シークレット取得に失敗");

        assert_eq!(result.get("database.password").unwrap(), "vault-db-pass");
        assert_eq!(result.get("redis.password").unwrap(), "vault-redis-pass");
    }

    // 指定キーが存在しない場合、結果マップに含まれないことを確認する。
    #[tokio::test]
    async fn fetch_secrets_partial_keys() {
        let client = InMemoryVaultClient::new();
        client.put_secret(make_secret("system/order/secrets"));

        let result = fetch_secrets_with_fallback(
            &client,
            "system/order/secrets",
            &["database.password", "nonexistent.key"],
            true,
        )
        .await
        .expect("シークレット取得に失敗");

        assert_eq!(result.len(), 1);
        assert_eq!(result.get("database.password").unwrap(), "vault-db-pass");
        assert!(!result.contains_key("nonexistent.key"));
    }

    // vault_required=true でシークレットが見つからない場合にエラーが返されることを確認する。
    #[tokio::test]
    async fn fetch_secrets_not_found_with_required() {
        let client = InMemoryVaultClient::new();

        let result = fetch_secrets_with_fallback(
            &client,
            "system/missing/secrets",
            &["database.password"],
            true,
        )
        .await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), VaultError::NotFound(_)));
    }

    // vault_required=false でも NotFound エラーはフォールバックせず、
    // エラーとして返されることを確認する（設定ミスの検出）。
    #[tokio::test]
    async fn fetch_secrets_not_found_without_required_still_errors() {
        let client = InMemoryVaultClient::new();

        let result = fetch_secrets_with_fallback(
            &client,
            "system/missing/secrets",
            &["database.password"],
            false,
        )
        .await;

        // NotFound はフォールバック対象外（ConnectionUnavailable のみがフォールバック対象）
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), VaultError::NotFound(_)));
    }

    // vault_required=true でシークレットが存在する場合、正常に取得できることを確認する。
    #[tokio::test]
    async fn fetch_secrets_with_required_true_succeeds() {
        let config = VaultClientConfig::new("http://vault:8200").vault_required(true);
        let client = InMemoryVaultClient::with_config(config);
        client.put_secret(make_secret("system/test/secrets"));

        let result = fetch_secrets_with_fallback(
            &client,
            "system/test/secrets",
            &["database.password"],
            true,
        )
        .await
        .expect("シークレット取得に失敗");

        assert_eq!(result.get("database.password").unwrap(), "vault-db-pass");
    }

    // vault_required ビルダーメソッドが正しく値を設定することを確認する。
    #[test]
    fn config_vault_required_builder() {
        let config = VaultClientConfig::new("http://vault:8200").vault_required(false);
        assert!(!config.vault_required);

        let config = VaultClientConfig::new("http://vault:8200").vault_required(true);
        assert!(config.vault_required);
    }

    // VaultClientConfig のデフォルトで vault_required が true であることを確認する。
    #[test]
    fn config_vault_required_default_is_true() {
        let config = VaultClientConfig::new("http://vault:8200");
        assert!(config.vault_required);
    }
}
