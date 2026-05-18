// flagd_publisher.rs — k1s0 tier1 control_plane: OpenFeature flagd feature flag 配布
// Tier1Service CRD の変更を ConfigMap 経由で OpenFeature flagd に配布する。
// Added/Modified: ConfigMap に feature flag 定義を書き込む。
// Deleted: ConfigMap から feature flag 定義を削除する。
// flagd は kubernetes sync で ConfigMap を監視して feature flag を更新する。

// anyhow: エラーハンドリング
use anyhow::{Context, Result};
// k8s-openapi: Kubernetes の ConfigMap / ObjectMeta 型
use k8s_openapi::api::core::v1::ConfigMap;
use k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta;
// kube: Kubernetes API クライアント + Api 型
use kube::{Api, Client};
// kube-core: PatchParams（ConfigMap の Server-Side Apply に使用する）/ Patch / DeleteParams
use kube::api::{DeleteParams, Patch, PatchParams};
// serde_json: ConfigMap の data フィールドの JSON 構築
use serde_json::json;
// std::collections::BTreeMap: ConfigMap の data フィールド型
use std::collections::BTreeMap;
// tracing: 構造化ロギング
use tracing::{debug, info, warn};

// FlagdPublisher は Kubernetes ConfigMap 経由で flagd に feature flag を配布する。
// flagd は `--sync-provider kubernetes` で FLAGD_CONFIGMAP_NAMESPACE / FLAGD_CONFIGMAP_NAME の
// ConfigMap を監視して自動的に feature flag を更新する。
pub struct FlagdPublisher {
    // client: Kubernetes API クライアント（CrdWatcher と共用する）
    client: Client,
    // flagd_namespace: feature flag ConfigMap を配置する namespace
    flagd_namespace: String,
}

impl FlagdPublisher {
    /// new は FlagdPublisher を構築する。
    /// flagd_namespace は環境変数 FLAGD_NAMESPACE から取得する（デフォルト: k1s0-system）。
    pub fn new(client: Client) -> Self {
        // FLAGD_NAMESPACE 環境変数から namespace を取得する
        let flagd_namespace = std::env::var("FLAGD_NAMESPACE")
            .unwrap_or_else(|_| "k1s0-system".to_string());
        Self { client, flagd_namespace }
    }

    /// publish は Tier1Service の spec に基づいて feature flag ConfigMap を作成 / 更新する。
    /// ConfigMap 名は `k1s0-flagd-{namespace}-{name}` 形式とする。
    pub async fn publish(
        &self,
        namespace: &str,
        name: &str,
        conformance_class: &str,
        adapter: &str,
    ) -> Result<()> {
        // ConfigMap 名を構築する（namespace + name をサニタイズしてハイフン連結する）
        let cm_name = sanitize_configmap_name(namespace, name);
        // flagd の feature flag 定義 JSON を構築する（OpenFeature flagd フォーマット）
        let flag_json = json!({
            "flags": {
                // flag key: tier1-service-{namespace}-{name}
                format!("tier1-service-{}-{}", sanitize_label(namespace), sanitize_label(name)): {
                    // 有効状態
                    "state": "ENABLED",
                    // 選択肢: conformance_class / adapter を variant として持つ
                    "variants": {
                        "conformance_class": conformance_class,
                        "adapter": adapter,
                        "disabled": "disabled"
                    },
                    // デフォルト variant は conformance_class とする
                    "defaultVariant": "conformance_class"
                }
            }
        })
        .to_string();
        // ConfigMap の data フィールドを構築する（flags.json キー）
        let mut data: BTreeMap<String, String> = BTreeMap::new();
        // flags.json キーに flagd feature flag 定義 JSON を設定する
        data.insert("flags.json".to_string(), flag_json);
        // ConfigMap オブジェクトを構築する
        let cm = ConfigMap {
            metadata: ObjectMeta {
                // ConfigMap 名を設定する
                name: Some(cm_name.clone()),
                // 配置先 namespace を設定する（FLAGD_NAMESPACE 環境変数から取得した値）
                namespace: Some(self.flagd_namespace.clone()),
                // 管理者ラベルを設定する（flagd が監視条件に使用できる）
                labels: Some(BTreeMap::from([
                    // 管理者ラベル: k1s0 control_plane が管理する ConfigMap を識別する
                    ("app.kubernetes.io/managed-by".to_string(), "k1s0-tier1-cp".to_string()),
                    // tier1-service のソース namespace ラベル
                    ("k1s0.io/source-namespace".to_string(), namespace.to_string()),
                    // tier1-service の名前ラベル
                    ("k1s0.io/source-name".to_string(), name.to_string()),
                ])),
                ..Default::default()
            },
            // data フィールドに flags.json を設定する
            data: Some(data),
            ..Default::default()
        };
        // Kubernetes API の ConfigMap エンドポイントを構築する
        let api: Api<ConfigMap> = Api::namespaced(self.client.clone(), &self.flagd_namespace);
        // Server-Side Apply で ConfigMap を作成または更新する
        let patch_params = PatchParams::apply("k1s0-tier1-cp");
        match api.patch(&cm_name, &patch_params, &Patch::Apply(&cm)).await {
            Ok(_) => {
                info!(
                    namespace = %namespace,
                    name = %name,
                    cm_name = %cm_name,
                    conformance_class = %conformance_class,
                    adapter = %adapter,
                    "FlagdPublisher: feature flag ConfigMap を作成/更新した"
                );
                Ok(())
            }
            Err(e) => {
                // ConfigMap の apply 失敗は警告のみ（flagd への配布失敗はオペレーション継続を妨げない）
                warn!(
                    error = %e,
                    cm_name = %cm_name,
                    "FlagdPublisher: feature flag ConfigMap の apply に失敗した（flagd 配布をスキップ）"
                );
                Err(anyhow::anyhow!("ConfigMap apply 失敗: {e}"))
            }
        }
    }

    /// delete は Tier1Service 削除時に対応する feature flag ConfigMap を削除する。
    pub async fn delete(&self, namespace: &str, name: &str) -> Result<()> {
        // ConfigMap 名を構築する
        let cm_name = sanitize_configmap_name(namespace, name);
        // Kubernetes API の ConfigMap エンドポイントを構築する
        let api: Api<ConfigMap> = Api::namespaced(self.client.clone(), &self.flagd_namespace);
        // ConfigMap を削除する（存在しない場合は NotFound を無視する）
        match api.delete(&cm_name, &kube::api::DeleteParams::default()).await {
            Ok(_) => {
                info!(
                    namespace = %namespace,
                    name = %name,
                    cm_name = %cm_name,
                    "FlagdPublisher: feature flag ConfigMap を削除した"
                );
                Ok(())
            }
            Err(kube::Error::Api(api_err)) if api_err.code == 404 => {
                // 削除対象が存在しない場合は成功として扱う（冪等性）
                debug!(cm_name = %cm_name, "FlagdPublisher: 削除対象 ConfigMap が存在しない（既に削除済み）");
                Ok(())
            }
            Err(e) => {
                // その他のエラーは警告のみ（削除失敗は継続を妨げない）
                warn!(
                    error = %e,
                    cm_name = %cm_name,
                    "FlagdPublisher: feature flag ConfigMap の削除に失敗した"
                );
                Err(anyhow::anyhow!("ConfigMap delete 失敗: {e}"))
            }
        }
    }
}

/// sanitize_configmap_name は namespace / name から ConfigMap 名を構築する。
/// Kubernetes の命名規則（RFC 1123 サブドメイン）に適合する名前を生成する。
fn sanitize_configmap_name(namespace: &str, name: &str) -> String {
    // ConfigMap 名: k1s0-flagd-{namespace}-{name} 形式（非英数字をハイフンに変換する）
    format!(
        "k1s0-flagd-{}-{}",
        sanitize_label(namespace),
        sanitize_label(name)
    )
}

/// sanitize_label は Kubernetes ラベル値 / 名前に使用できる文字列に変換する。
/// 大文字を小文字に変換し、英数字以外をハイフンに置換し、63 文字以内に切り詰める。
fn sanitize_label(s: &str) -> String {
    // 大文字を小文字に変換してから英数字以外をハイフンに置換する
    let sanitized: String = s
        .to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() || c == '-' { c } else { '-' })
        .collect();
    // Kubernetes 名前の最大長 63 文字に切り詰める
    sanitized.chars().take(63).collect()
}

#[cfg(test)]
mod tests {
    // テストモジュール内部でのみ使用するインポート
    use super::*;

    #[test]
    // sanitize_configmap_name が正しい形式を返すことを確認する
    fn test_sanitize_configmap_name() {
        // 通常の namespace / name で正しく変換されることを確認する
        let name = sanitize_configmap_name("default", "my-service");
        assert_eq!(name, "k1s0-flagd-default-my-service");
    }

    #[test]
    // sanitize_label が非英数字を正しくハイフンに変換することを確認する
    fn test_sanitize_label_replaces_special_chars() {
        // ピリオドやアンダースコアがハイフンに変換されることを確認する
        let label = sanitize_label("my.service_name");
        assert_eq!(label, "my-service-name");
    }

    #[test]
    // sanitize_label が 63 文字以内に切り詰めることを確認する
    fn test_sanitize_label_truncates_to_63() {
        // 70 文字の入力が 63 文字に切り詰められることを確認する
        let long_name = "a".repeat(70);
        let label = sanitize_label(&long_name);
        assert_eq!(label.len(), 63);
    }
}
