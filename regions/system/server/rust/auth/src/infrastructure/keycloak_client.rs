use std::sync::Arc;

use async_trait::async_trait;
use k1s0_circuit_breaker::{CircuitBreaker, CircuitBreakerConfig};
// secrecy クレートを使用して Keycloak パスワードを Secret<String> で保持し、Debug 出力への漏洩を防ぐ（H-1 監査対応）。
use secrecy::{ExposeSecret, Secret};
use tokio::sync::{RwLock, Semaphore};

use crate::domain::entity::user::{Pagination, Role, User, UserListResult, UserRoles};
use crate::domain::error::AuthError;
use crate::domain::repository::UserRepository;

/// KeycloakConfig は Keycloak 接続の設定を表す。
#[derive(Debug, Clone, serde::Deserialize)]
pub struct KeycloakConfig {
    pub base_url: String,
    pub realm: String,
    pub client_id: String,
    // client_secret は Secret<String> で保持し、Debug トレイトでは [REDACTED] と表示される
    // クライアントシークレットは必須項目のため serde(default) を設定しない（Secret<String> は Default 未実装）
    pub client_secret: Secret<String>,
    #[serde(default = "default_admin_realm")]
    pub admin_realm: String,
    #[serde(default = "default_admin_client_id")]
    pub admin_client_id: String,
    #[serde(default)]
    pub admin_username: String,
    // admin_password は Secret<String> で保持し、Debug トレイトでは [REDACTED] と表示される
    pub admin_password: Secret<String>,
}

fn default_admin_realm() -> String {
    "master".to_string()
}

fn default_admin_client_id() -> String {
    "admin-cli".to_string()
}

impl KeycloakConfig {
    /// admin_password が設定されている場合は Resource Owner Password Grant を使用する。
    fn uses_admin_password_grant(&self) -> bool {
        !self.admin_username.is_empty() && !self.admin_password.expose_secret().is_empty()
    }

    pub(crate) fn admin_token_url(&self) -> String {
        let realm = if self.uses_admin_password_grant() {
            self.admin_realm.as_str()
        } else {
            self.realm.as_str()
        };
        format!(
            "{}/realms/{}/protocol/openid-connect/token",
            self.base_url, realm
        )
    }

    /// Keycloak Admin API トークン取得用のフォームデータを生成する。
    ///
    /// MED-15 監査対応: admin_username が設定されている場合は Resource Owner Password Credentials
    /// (ROPC) Grant を使用するが、ROPC は OAuth 2.1 (draft) で廃止予定のフローである。
    /// 将来的に Client Credentials Grant（client_id + client_secret のみ）への移行を検討すること。
    /// 移行計画は ADR-0061 で文書化済み。
    ///
    /// 参考: https://oauth.net/2/grant-types/password/
    pub(crate) fn admin_token_form(&self) -> Vec<(&'static str, String)> {
        if self.uses_admin_password_grant() {
            // TODO(ADR-0061): ROPC (password grant) は OAuth 2.1 で廃止予定。
            // Keycloak の Service Account を使った Client Credentials Grant に移行すること。
            // 移行手順は ADR-0061 を参照。
            vec![
                ("grant_type", "password".to_string()),
                ("client_id", self.admin_client_id.clone()),
                ("username", self.admin_username.clone()),
                // expose_secret() でパスワードを取り出してフォームに設定する
                ("password", self.admin_password.expose_secret().to_string()),
            ]
        } else {
            // Client Credentials Grant: OAuth 2.1 推奨フロー（ADR-0061 で採用決定）
            vec![
                ("grant_type", "client_credentials".to_string()),
                ("client_id", self.client_id.clone()),
                // expose_secret() でクライアントシークレットを取り出してフォームに設定する
                (
                    "client_secret",
                    self.client_secret.expose_secret().to_string(),
                ),
            ]
        }
    }
}

/// CachedToken はキャッシュされた管理トークンを表す。
struct CachedToken {
    token: String,
    expires_at: chrono::DateTime<chrono::Utc>,
}

/// KeycloakClient は Keycloak Admin API クライアント。
/// サーキットブレーカーにより、Keycloak 障害時の連鎖的な障害伝播を防止する。
pub struct KeycloakClient {
    config: KeycloakConfig,
    http_client: reqwest::Client,
    admin_token: Arc<RwLock<Option<CachedToken>>>,
    /// Keycloak への HTTP リクエストを保護するサーキットブレーカー。
    /// 外部サービス障害時にリクエストを遮断し、システム全体の安定性を確保する。
    circuit_breaker: CircuitBreaker,
    /// H-5 対応: トークン更新の同時実行を1タスクに制限し、Thundering Herd 問題を防止する。
    /// キャッシュ期限切れ時に複数の async タスクが同時に Keycloak へリクエストを送信することを防ぐ。
    token_refresh_semaphore: Semaphore,
}

impl KeycloakClient {
    /// 新しい KeycloakClient を生成する。
    /// TLS バックエンドの初期化に失敗した場合は Err を返す。
    pub fn new(config: KeycloakConfig) -> anyhow::Result<Self> {
        // サーキットブレーカー設定:
        // - failure_threshold: 5回連続失敗でOpen状態に遷移（Keycloakの一時的な遅延を許容）
        // - success_threshold: 3回連続成功でClosed状態に復帰（安定性を確認）
        // - timeout: 30秒後にHalfOpen状態で再試行（Keycloakの再起動時間を考慮）
        let cb_config = CircuitBreakerConfig::default();

        // reqwest の Client 構築: TLS バックエンドが利用不可の場合はエラーとして伝播する
        let http_client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .map_err(|e| anyhow::anyhow!("reqwest::Client の構築に失敗: {}", e))?;

        Ok(Self {
            config,
            http_client,
            admin_token: Arc::new(RwLock::new(None)),
            circuit_breaker: CircuitBreaker::new(cb_config),
            // H-5 対応: permit 数 1 の Semaphore でトークン更新を逐次化する
            token_refresh_semaphore: Semaphore::new(1),
        })
    }

    /// Keycloak のヘルスチェックを行う。
    /// サーキットブレーカーで保護し、Keycloak 停止時の不要なリクエストを抑制する。
    /// LOW-06 対応: AppState に Arc<KeycloakClient> が追加された場合は healthz/readyz から呼び出すこと。
    /// 現状は AppState が http_client+keycloak_url で直接確認しているため dead_code となっているが、
    /// TODO(LOW-06): AppState への KeycloakClient 組み込み後にこのアトリビュートを削除すること。
    #[allow(dead_code)]
    pub async fn healthy(&self) -> anyhow::Result<()> {
        let url = format!("{}/realms/{}", self.config.base_url, self.config.realm);
        let http = &self.http_client;
        self.circuit_breaker
            .call(|| async {
                let resp = http.get(&url).send().await?;
                resp.error_for_status()?;
                Ok::<(), reqwest::Error>(())
            })
            .await
            .map_err(|e| anyhow::anyhow!("keycloak health check failed: {}", e))
    }

    /// Admin API トークンを取得する（キャッシュ付き）。
    /// H-5 対応: 高速パス（Read lock）→ Semaphore 取得 → ダブルチェック → Write lock の
    /// 3段階構造により、Thundering Herd を防止しつつキャッシュヒット時の性能を最大化する。
    async fn get_admin_token(&self) -> anyhow::Result<String> {
        // 高速パス: 読み取りロックでキャッシュ確認（競合なし）
        {
            let cache = self.admin_token.read().await;
            if let Some(ref cached) = *cache {
                if chrono::Utc::now() < cached.expires_at {
                    return Ok(cached.token.clone());
                }
            }
        } // Read lock をここで drop する

        // H-5 対応: Semaphore で単一タスクのみトークン取得を実行し Thundering Herd を防ぐ
        let _permit = self
            .token_refresh_semaphore
            .acquire()
            .await
            .map_err(|e| anyhow::anyhow!("Semaphore acquire に失敗しました: {}", e))?;

        // Semaphore 取得後にダブルチェック（先行タスクがトークンを更新済みの場合はスキップ）
        {
            let cache = self.admin_token.read().await;
            if let Some(ref cached) = *cache {
                if chrono::Utc::now() < cached.expires_at {
                    return Ok(cached.token.clone());
                }
            }
        }

        // Write lock を取得してトークンを更新する
        let mut cache = self.admin_token.write().await;

        let token_url = self.config.admin_token_url();
        let form = self.config.admin_token_form();

        // サーキットブレーカーでトークン取得リクエストを保護する。
        // Keycloak 障害時にトークン取得の連続失敗を防ぎ、キャッシュ期限切れ時の負荷集中を回避する。
        let http = &self.http_client;
        let body: serde_json::Value = self
            .circuit_breaker
            .call(|| async {
                let resp = http.post(&token_url).form(&form).send().await?;
                let body: serde_json::Value = resp.error_for_status()?.json().await?;
                Ok::<serde_json::Value, reqwest::Error>(body)
            })
            .await
            .map_err(|e| anyhow::anyhow!("keycloak token request failed: {}", e))?;

        let token = body["access_token"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("missing access_token in response"))?
            .to_string();
        // Keycloak レスポンスに expires_in が含まれない場合はデフォルト 300 秒にフォールバックする
        // フォールバック使用時は警告ログを出力し、Keycloak 設定の確認を促す（HIGH-CODE-01 監査対応）
        let expires_in = match body["expires_in"].as_i64() {
            Some(v) => v,
            None => {
                tracing::warn!(
                    "Keycloak トークンレスポンスに expires_in が含まれていません。デフォルト値 300 秒を使用します。Keycloak の設定を確認してください。"
                );
                300
            }
        };

        *cache = Some(CachedToken {
            token: token.clone(),
            expires_at: chrono::Utc::now() + chrono::Duration::seconds(expires_in - 30), // 30 sec buffer
        });

        Ok(token)
    }
}

#[async_trait]
impl UserRepository for KeycloakClient {
    async fn find_by_id(&self, user_id: &str) -> anyhow::Result<User> {
        let token = self.get_admin_token().await?;
        let url = format!(
            "{}/admin/realms/{}/users/{}",
            self.config.base_url, self.config.realm, user_id
        );

        // サーキットブレーカーでユーザー取得リクエストを保護する
        let http = &self.http_client;
        let resp = self
            .circuit_breaker
            .call(|| async { http.get(&url).bearer_auth(&token).send().await })
            .await
            .map_err(|e| anyhow::anyhow!("keycloak find_by_id failed: {}", e))?;

        if resp.status() == reqwest::StatusCode::NOT_FOUND {
            // M-10 対応: 型安全なドメインエラーを使用して適切な HTTP ステータスコードに変換する
            return Err(
                AuthError::NotFound(format!("ユーザーが見つかりません: {}", user_id)).into(),
            );
        }

        let kc_user: KeycloakUser = resp.error_for_status()?.json().await?;
        Ok(kc_user.into())
    }

    async fn list(
        &self,
        page: i32,
        page_size: i32,
        search: Option<String>,
        enabled: Option<bool>,
    ) -> anyhow::Result<UserListResult> {
        let token = self.get_admin_token().await?;
        let first = (page - 1) * page_size;
        let mut url = format!(
            "{}/admin/realms/{}/users?first={}&max={}",
            self.config.base_url, self.config.realm, first, page_size
        );

        if let Some(ref q) = search {
            // CRIT-07 対応: search パラメータを URL エンコードしてインジェクションを防止する
            url.push_str(&format!("&search={}", urlencoding::encode(q)));
        }
        if let Some(e) = enabled {
            url.push_str(&format!("&enabled={}", e));
        }

        // サーキットブレーカーでユーザー一覧取得リクエストを保護する
        let http = &self.http_client;
        let resp = self
            .circuit_breaker
            .call(|| async { http.get(&url).bearer_auth(&token).send().await })
            .await
            .map_err(|e| anyhow::anyhow!("keycloak list users failed: {}", e))?;

        let kc_users: Vec<KeycloakUser> = resp.error_for_status()?.json().await?;

        // MED-04 監査対応: count_url にも search と enabled フィルターを適用する。
        // list_url と同じ条件でカウントしないと has_next 計算が誤った値を返す可能性がある。
        // urlencoding::encode を使用して検索文字列を URL エンコードし、インジェクションを防止する。
        let mut count_url = format!(
            "{}/admin/realms/{}/users/count",
            self.config.base_url, self.config.realm
        );
        let mut count_first_param = true;
        if let Some(ref q) = search {
            count_url.push_str(if count_first_param { "?" } else { "&" });
            count_url.push_str(&format!("search={}", urlencoding::encode(q)));
            count_first_param = false;
        }
        if let Some(e) = enabled {
            count_url.push_str(if count_first_param { "?" } else { "&" });
            count_url.push_str(&format!("enabled={}", e));
        }
        let count_token = self.get_admin_token().await?;
        // サーキットブレーカーでユーザー数取得リクエストを保護する
        let count_resp = self
            .circuit_breaker
            .call(|| async { http.get(&count_url).bearer_auth(&count_token).send().await })
            .await
            .map_err(|e| anyhow::anyhow!("keycloak user count failed: {}", e))?;
        let total_count: i64 = count_resp.error_for_status()?.json().await?;

        let users: Vec<User> = kc_users.into_iter().map(|u| u.into()).collect();
        let has_next = (page as i64 * page_size as i64) < total_count;

        Ok(UserListResult {
            users,
            pagination: Pagination {
                total_count,
                page,
                page_size,
                has_next,
            },
        })
    }

    async fn get_roles(&self, user_id: &str) -> anyhow::Result<UserRoles> {
        let token = self.get_admin_token().await?;

        // Realm roles
        let realm_url = format!(
            "{}/admin/realms/{}/users/{}/role-mappings/realm",
            self.config.base_url, self.config.realm, user_id
        );
        // サーキットブレーカーでロール取得リクエストを保護する
        let http = &self.http_client;
        let resp = self
            .circuit_breaker
            .call(|| async { http.get(&realm_url).bearer_auth(&token).send().await })
            .await
            .map_err(|e| anyhow::anyhow!("keycloak get_roles failed: {}", e))?;

        if resp.status() == reqwest::StatusCode::NOT_FOUND {
            // M-10 対応: 型安全なドメインエラーを使用して適切な HTTP ステータスコードに変換する
            return Err(
                AuthError::NotFound(format!("ユーザーが見つかりません: {}", user_id)).into(),
            );
        }

        let realm_roles: Vec<KeycloakRole> = resp.error_for_status()?.json().await?;

        Ok(UserRoles {
            user_id: user_id.to_string(),
            realm_roles: realm_roles.into_iter().map(|r| r.into()).collect(),
            client_roles: std::collections::HashMap::new(),
        })
    }
}

/// KeycloakUser は Keycloak Admin API のユーザー表現。
#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct KeycloakUser {
    id: String,
    username: String,
    #[serde(default)]
    email: String,
    #[serde(default)]
    first_name: String,
    #[serde(default)]
    last_name: String,
    #[serde(default = "default_true")]
    enabled: bool,
    #[serde(default)]
    email_verified: bool,
    #[serde(default)]
    created_timestamp: i64,
    #[serde(default)]
    attributes: std::collections::HashMap<String, Vec<String>>,
}

fn default_true() -> bool {
    true
}

impl From<KeycloakUser> for User {
    fn from(kc: KeycloakUser) -> Self {
        // MED-06 監査対応: created_timestamp が 0 または無効値でフォールバックが発生した場合に警告ログを出力する。
        // フォールバックが発生すると created_at が現在時刻になり、データの正確性が失われるため
        // Keycloak の設定や API レスポンスの確認を促す目的でログを残す。
        let created_at = if kc.created_timestamp == 0 {
            tracing::warn!(
                user_id = %kc.id,
                "Keycloak ユーザーの created_timestamp が 0 です。現在時刻でフォールバックします。Keycloak の設定を確認してください。"
            );
            chrono::Utc::now()
        } else {
            chrono::DateTime::from_timestamp_millis(kc.created_timestamp).unwrap_or_else(|| {
                tracing::warn!(
                    user_id = %kc.id,
                    created_timestamp = kc.created_timestamp,
                    "Keycloak ユーザーの created_timestamp が無効な値です。現在時刻でフォールバックします。"
                );
                chrono::Utc::now()
            })
        };

        User {
            id: kc.id,
            username: kc.username,
            email: kc.email,
            first_name: kc.first_name,
            last_name: kc.last_name,
            enabled: kc.enabled,
            email_verified: kc.email_verified,
            created_at,
            attributes: kc.attributes,
        }
    }
}

/// KeycloakRole は Keycloak Admin API のロール表現。
#[derive(Debug, serde::Deserialize)]
struct KeycloakRole {
    id: String,
    name: String,
    #[serde(default)]
    description: String,
}

impl From<KeycloakRole> for Role {
    fn from(kc: KeycloakRole) -> Self {
        Role {
            id: kc.id,
            name: kc.name,
            description: kc.description,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keycloak_user_to_domain_user() {
        let kc_user = KeycloakUser {
            id: "user-1".to_string(),
            username: "taro.yamada".to_string(),
            email: "taro@example.com".to_string(),
            first_name: "Taro".to_string(),
            last_name: "Yamada".to_string(),
            enabled: true,
            email_verified: true,
            created_timestamp: 1710000000000,
            attributes: std::collections::HashMap::from([(
                "department".to_string(),
                vec!["engineering".to_string()],
            )]),
        };

        let user: User = kc_user.into();
        assert_eq!(user.id, "user-1");
        assert_eq!(user.username, "taro.yamada");
        assert!(user.enabled);
        assert_eq!(
            user.attributes
                .get("department")
                .expect("department attribute should exist"),
            &vec!["engineering".to_string()]
        );
    }

    #[test]
    fn test_keycloak_role_to_domain_role() {
        let kc_role = KeycloakRole {
            id: "role-1".to_string(),
            name: "sys_admin".to_string(),
            description: "System administrator".to_string(),
        };

        let role: Role = kc_role.into();
        assert_eq!(role.id, "role-1");
        assert_eq!(role.name, "sys_admin");
    }

    #[test]
    fn test_keycloak_config_deserialization() {
        let yaml = r#"
base_url: "https://auth.k1s0.internal.example.com"
realm: "k1s0"
client_id: "auth-server"
client_secret: "secret"
admin_password: ""
"#;
        let config: KeycloakConfig =
            serde_yaml::from_str(yaml).expect("YAML deserialization should succeed");
        assert_eq!(config.base_url, "https://auth.k1s0.internal.example.com");
        assert_eq!(config.realm, "k1s0");
    }
}
