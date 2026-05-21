// apicurio_gitops.rs — spec 06 §apicurio_gitops_sot: Apicurio GitOps SoT controller
// GitOps パイプラインからスキーマ artifact を同期し、additive_only policy を強制する実装
// docs/04_詳細設計/01_適合仕様 に基づく schema registry GitOps 連携の物理化

// 非同期 HTTP クライアント（Apicurio REST API 呼び出しに使用する）
use reqwest::Client;
// serde: JSON デシリアライズ（Apicurio API レスポンスのパースに使用する）
use serde::{Deserialize, Serialize};
// anyhow: エラーハンドリング（Result 型の統一）
use anyhow::{anyhow, Result};
// Path: ディレクトリ走査で yaml ファイルを探索するために使用する
use std::path::Path;
// env: 環境変数取得（APICURIO_URL の読み込みに使用する）
use std::env;
// sha2: content hash 比較に使用する SHA-256 ハッシュ計算
use sha2::{Sha256, Digest};

// ============================================================
// ApicurioGitOpsSyncer 構造体
// ============================================================

// ApicurioGitOpsSyncer は Apicurio Schema Registry の GitOps 同期コントローラーを表す
pub struct ApicurioGitOpsSyncer {
    // apicurio_url: Apicurio Registry の base URL（例: http://apicurio:8080/apis/registry/v2）
    pub apicurio_url: String,
    // client: reqwest の非同期 HTTP クライアントインスタンス
    pub client: Client,
    // group: Apicurio Registry の group 名（artifact 管理単位）
    pub group: String,
}

// ============================================================
// SchemaArtifact 構造体
// ============================================================

// SchemaArtifact は GitOps リポジトリから読み込んだスキーマ artifact を表す
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaArtifact {
    // artifact_id: artifact の識別子（Apicurio の artifact_id に対応する）
    pub artifact_id: String,
    // content: スキーマの raw コンテンツ文字列（JSON / Avro / Protobuf 等）
    pub content: String,
    // schema_class: スキーマの種別（JSON / AVRO / PROTOBUF 等）
    pub schema_class: String,
    // version: 任意のバージョン文字列（省略可能）
    pub version: Option<String>,
}

// ============================================================
// SyncResult 構造体
// ============================================================

// SyncResult は 1 artifact の同期処理結果を表す
#[derive(Debug, Clone)]
pub struct SyncResult {
    // artifact_id: 同期処理対象の artifact 識別子
    pub artifact_id: String,
    // action: 実際に実行した同期アクション
    pub action: SyncAction,
    // drift_detected: リモートとローカルの内容に差異が検出されたかどうか
    pub drift_detected: bool,
}

// ============================================================
// SyncAction 列挙型
// ============================================================

// SyncAction は sync_schema が実行した操作種別を表す
#[derive(Debug, Clone, PartialEq)]
pub enum SyncAction {
    // Created: artifact が存在しなかったため新規作成した
    Created,
    // Updated: drift が検出されたため artifact を更新した
    Updated,
    // Unchanged: drift がなく更新不要だった
    Unchanged,
    // DriftBlocked: additive_only policy 違反のため更新をブロックした
    DriftBlocked,
}

// ============================================================
// Apicurio API レスポンス型（内部用）
// ============================================================

// ArtifactSearchResults は Apicurio の GET /groups/{group}/artifacts レスポンスを表す
#[derive(Debug, Deserialize)]
struct ArtifactSearchResults {
    // artifacts: artifact メタデータのリスト
    artifacts: Vec<ArtifactMeta>,
}

// ArtifactMeta は Apicurio artifact 一覧エントリを表す
#[derive(Debug, Deserialize)]
struct ArtifactMeta {
    // id: artifact の識別子
    id: String,
}

// CreateVersionResponse は Apicurio の PUT .../versions レスポンスを表す
#[derive(Debug, Deserialize)]
struct CreateVersionResponse {
    // version: 登録されたバージョン番号（文字列形式）
    #[allow(dead_code)]
    // version フィールドはデシリアライズするが直接は使用しない
    version: Option<String>,
}

// ============================================================
// ApicurioGitOpsSyncer 実装
// ============================================================

// ApicurioGitOpsSyncer の実装
impl ApicurioGitOpsSyncer {
    // ApicurioGitOpsSyncer を環境変数から構築する
    // APICURIO_URL: Apicurio Registry の base URL（デフォルト: http://apicurio:8080/apis/registry/v2）
    // APICURIO_GROUP: artifact group 名（デフォルト: default）
    pub fn from_env() -> Result<Self> {
        // APICURIO_URL 環境変数を取得する（デフォルト値を使用する）
        let apicurio_url = env::var("APICURIO_URL")
            // 未設定時のデフォルト値を設定する
            .unwrap_or_else(|_| "http://apicurio:8080/apis/registry/v2".to_string());
        // APICURIO_GROUP 環境変数を取得する（デフォルト: default）
        let group = env::var("APICURIO_GROUP")
            // 未設定時のデフォルト値を設定する
            .unwrap_or_else(|_| "default".to_string());
        // reqwest の非同期 HTTP クライアントを生成する
        let client = Client::new();
        // フィールドを初期化して返す
        Ok(Self {
            // apicurio_url を保存する
            apicurio_url,
            // reqwest クライアントを保存する
            client,
            // group を保存する
            group,
        })
    }

    // ============================================================
    // artifact 存在確認（内部ヘルパー）
    // ============================================================

    // artifact_exists は指定 artifact が Apicurio Registry に存在するか確認する
    // artifact_id: 存在確認対象の artifact 識別子
    // Returns: 存在する場合は true、存在しない場合は false を返す
    async fn artifact_exists(&self, artifact_id: &str) -> Result<bool> {
        // GET エンドポイント URL を構築する（artifact 一覧から確認する）
        let url = format!(
            "{}/groups/{}/artifacts",
            self.apicurio_url, self.group
        );
        // HTTP GET リクエストを送信する
        let response = self
            .client
            // GET メソッドで URL にリクエストを送信する
            .get(&url)
            // Accept ヘッダーに JSON を指定する
            .header("Accept", "application/json")
            // リクエストを送信する
            .send()
            .await
            // reqwest エラーを anyhow エラーに変換する
            .map_err(|e| anyhow!("Apicurio GET /artifacts 失敗: {}", e))?;

        // HTTP ステータスコードを確認する
        if !response.status().is_success() {
            // エラーステータスの場合はエラーを返す
            let status = response.status();
            // レスポンスボディをテキストとして取得する
            let body = response.text().await.unwrap_or_default();
            // エラーメッセージを返す
            return Err(anyhow!(
                "Apicurio artifact_exists 一覧取得失敗: status={}, body={}",
                status,
                body
            ));
        }

        // レスポンス JSON を ArtifactSearchResults にデシリアライズする
        let results: ArtifactSearchResults = response
            .json()
            .await
            // デシリアライズエラーを anyhow エラーに変換する
            .map_err(|e| anyhow!("Apicurio artifact_exists レスポンスのパース失敗: {}", e))?;

        // artifact_id が一覧に含まれているか確認する
        let exists = results.artifacts.iter().any(|a| a.id == artifact_id);
        // 結果を返す
        Ok(exists)
    }

    // ============================================================
    // content hash 比較（内部ヘルパー）
    // ============================================================

    // fetch_remote_hash は Apicurio からリモートの artifact content hash を取得する
    // artifact_id: 対象の artifact 識別子
    // Returns: リモート content の SHA-256 ハッシュ文字列
    async fn fetch_remote_hash(&self, artifact_id: &str) -> Result<String> {
        // GET エンドポイント URL を構築する（artifact の最新 content を取得する）
        let url = format!(
            "{}/groups/{}/artifacts/{}",
            self.apicurio_url, self.group, artifact_id
        );
        // HTTP GET リクエストを送信する
        let response = self
            .client
            // GET メソッドで URL にリクエストを送信する
            .get(&url)
            // Accept ヘッダーには */* を指定して raw content を取得する
            .header("Accept", "*/*")
            // リクエストを送信する
            .send()
            .await
            // reqwest エラーを anyhow エラーに変換する
            .map_err(|e| anyhow!("Apicurio GET /artifacts/{} 失敗: {}", artifact_id, e))?;

        // HTTP ステータスコードを確認する
        if !response.status().is_success() {
            // エラーステータスの場合はエラーを返す
            let status = response.status();
            // レスポンスボディをテキストとして取得する
            let body = response.text().await.unwrap_or_default();
            // エラーメッセージを返す
            return Err(anyhow!(
                "Apicurio fetch_remote_hash 失敗: artifact_id={}, status={}, body={}",
                artifact_id,
                status,
                body
            ));
        }

        // レスポンス body を bytes として取得する
        let remote_bytes = response
            .bytes()
            .await
            // bytes 取得エラーを anyhow エラーに変換する
            .map_err(|e| anyhow!("Apicurio fetch_remote_hash bytes 取得失敗: {}", e))?;

        // SHA-256 ハッシュを計算して hex 文字列で返す
        let hash = format!("{:x}", Sha256::digest(&remote_bytes));
        // ハッシュ文字列を返す
        Ok(hash)
    }

    // ============================================================
    // additive_only policy チェック（内部ヘルパー）
    // ============================================================

    // check_additive_only は新しいコンテンツがフィールドを削除していないか確認する
    // remote_content: リモートの schema コンテンツ文字列
    // new_content: 新しい schema コンテンツ文字列
    // Returns: additive_only policy 違反がある場合は Err を返す
    fn check_additive_only(remote_content: &str, new_content: &str) -> Result<()> {
        // JSON として両方をパースして field を比較する（JSON schema の場合のみ検証する）
        // JSON パースに成功した場合のみ検証する（失敗時は検証スキップ）
        let remote_json: serde_json::Value = match serde_json::from_str(remote_content) {
            // パース成功時は JSON Value を使用する
            Ok(v) => v,
            // JSON パース失敗時は検証をスキップして Ok を返す（非 JSON schema の場合）
            Err(_) => return Ok(()),
        };
        // 新しいコンテンツを JSON としてパースする
        let new_json: serde_json::Value = match serde_json::from_str(new_content) {
            // パース成功時は JSON Value を使用する
            Ok(v) => v,
            // JSON パース失敗時は検証をスキップして Ok を返す
            Err(_) => return Ok(()),
        };
        // properties キーが存在する場合（JSON Schema）にフィールド削除を検証する
        if let (Some(remote_props), Some(new_props)) = (
            // リモートの properties を取得する
            remote_json.get("properties").and_then(|v| v.as_object()),
            // 新しいコンテンツの properties を取得する
            new_json.get("properties").and_then(|v| v.as_object()),
        ) {
            // リモートの全フィールドが新しいコンテンツにも存在するか確認する
            for key in remote_props.keys() {
                // フィールドが新しいコンテンツに存在しない場合はエラーを返す
                if !new_props.contains_key(key) {
                    // additive_only policy 違反: フィールドが削除されているため Err を返す
                    return Err(anyhow!(
                        "additive_only policy 違反: フィールド '{}' が削除されています。スキーマ変更は additive のみ許可されます",
                        key
                    ));
                }
            }
        }
        // additive_only policy 違反なし
        Ok(())
    }

    // ============================================================
    // artifact 新規作成（内部ヘルパー）
    // ============================================================

    // create_artifact は Apicurio Registry に artifact を新規作成する
    // artifact: 作成する SchemaArtifact
    // Returns: 作成した artifact の ID
    async fn create_artifact(&self, artifact: &SchemaArtifact) -> Result<String> {
        // POST エンドポイント URL を構築する
        let url = format!(
            "{}/groups/{}/artifacts",
            self.apicurio_url, self.group
        );
        // HTTP POST リクエストを送信する
        let response = self
            .client
            // POST メソッドで URL にリクエストを送信する
            .post(&url)
            // Content-Type は application/json を指定する（Apicurio v2 API 仕様）
            .header("Content-Type", "application/json")
            // X-Registry-ArtifactId ヘッダーで artifact ID を指定する
            .header("X-Registry-ArtifactId", &artifact.artifact_id)
            // X-Registry-ArtifactType ヘッダーで schema タイプを指定する
            .header("X-Registry-ArtifactType", &artifact.schema_class)
            // content を body として送信する
            .body(artifact.content.clone())
            // リクエストを送信する
            .send()
            .await
            // reqwest エラーを anyhow エラーに変換する
            .map_err(|e| anyhow!("Apicurio POST /artifacts 失敗: {}", e))?;

        // HTTP ステータスコードを確認する
        if !response.status().is_success() {
            // エラーステータスの場合はエラーを返す
            let status = response.status();
            // レスポンスボディをテキストとして取得する
            let body = response.text().await.unwrap_or_default();
            // エラーメッセージを返す
            return Err(anyhow!(
                "Apicurio create_artifact 失敗: artifact_id={}, status={}, body={}",
                artifact.artifact_id,
                status,
                body
            ));
        }

        // レスポンス JSON から artifact ID を取得する
        // Apicurio v2 は作成時に artifact メタデータを返す
        let body_text = response
            .text()
            .await
            // テキスト取得エラーを anyhow エラーに変換する
            .map_err(|e| anyhow!("Apicurio create_artifact レスポンス取得失敗: {}", e))?;

        // JSON から id フィールドを取得する
        let json_val: serde_json::Value = serde_json::from_str(&body_text)
            // JSON パースエラーを anyhow エラーに変換する
            .map_err(|e| anyhow!("Apicurio create_artifact レスポンスのパース失敗: {}, body={}", e, body_text))?;

        // id フィールドを文字列として取得する
        let id = json_val
            .get("id")
            // id フィールドが存在しない場合はエラーを返す
            .and_then(|v| v.as_str())
            // Option から Result に変換する
            .ok_or_else(|| anyhow!("Apicurio create_artifact: id フィールドが見つかりません, body={}", body_text))?
            // 文字列スライスを String に変換する
            .to_string();

        // 作成した artifact の ID を返す
        Ok(id)
    }

    // ============================================================
    // artifact バージョン更新（内部ヘルパー）
    // ============================================================

    // update_artifact_version は Apicurio Registry の artifact に新バージョンを追加する
    // artifact: 更新する SchemaArtifact
    // Returns: 作成したバージョン番号
    async fn update_artifact_version(&self, artifact: &SchemaArtifact) -> Result<()> {
        // PUT エンドポイント URL を構築する（versions エンドポイント）
        let url = format!(
            "{}/groups/{}/artifacts/{}/versions",
            self.apicurio_url, self.group, artifact.artifact_id
        );
        // HTTP PUT リクエストを送信する
        let response = self
            .client
            // POST メソッドで versions エンドポイントにリクエストを送信する（Apicurio v2 API）
            .post(&url)
            // Content-Type は application/json を指定する
            .header("Content-Type", "application/json")
            // X-Registry-ArtifactType ヘッダーで schema タイプを指定する
            .header("X-Registry-ArtifactType", &artifact.schema_class)
            // 新しい content を body として送信する
            .body(artifact.content.clone())
            // リクエストを送信する
            .send()
            .await
            // reqwest エラーを anyhow エラーに変換する
            .map_err(|e| anyhow!("Apicurio POST /versions 失敗: {}", e))?;

        // HTTP ステータスコードを確認する
        if !response.status().is_success() {
            // エラーステータスの場合はエラーを返す
            let status = response.status();
            // レスポンスボディをテキストとして取得する
            let body = response.text().await.unwrap_or_default();
            // エラーメッセージを返す
            return Err(anyhow!(
                "Apicurio update_artifact_version 失敗: artifact_id={}, status={}, body={}",
                artifact.artifact_id,
                status,
                body
            ));
        }

        // レスポンスを読み捨てる（バージョン番号は戻り値として使用しない）
        let _ = response.json::<CreateVersionResponse>().await;
        // 更新成功を返す
        Ok(())
    }

    // ============================================================
    // sync_schema: artifact 同期の主要メソッド
    // ============================================================

    // sync_schema は 1 つの SchemaArtifact を Apicurio Registry に同期する
    // artifact が存在しない場合: POST /groups/{group}/artifacts で create する
    // artifact が存在する場合: content hash を比較し drift があれば PUT .../versions で update する
    // additive_only policy の検証を行い、field 削除は Err を返す
    // artifact: 同期対象の SchemaArtifact
    // Returns: 同期処理の結果を表す SyncResult
    pub async fn sync_schema(&self, artifact: &SchemaArtifact) -> Result<SyncResult> {
        // artifact が Apicurio Registry に存在するか確認する
        let exists = self.artifact_exists(&artifact.artifact_id).await?;

        // artifact が存在しない場合は新規作成する
        if !exists {
            // Apicurio Registry に artifact を新規作成する
            let _created_id = self.create_artifact(artifact).await?;
            // Created アクションで SyncResult を返す
            return Ok(SyncResult {
                // artifact_id を設定する
                artifact_id: artifact.artifact_id.clone(),
                // 新規作成アクションを設定する
                action: SyncAction::Created,
                // drift は新規作成のため false とする
                drift_detected: false,
            });
        }

        // artifact が存在する場合はリモートの content hash を取得して比較する
        let remote_hash = self.fetch_remote_hash(&artifact.artifact_id).await?;
        // ローカルの content の SHA-256 ハッシュを計算する
        let local_hash = format!("{:x}", Sha256::digest(artifact.content.as_bytes()));

        // hash を比較して drift を検出する
        let drift_detected = remote_hash != local_hash;

        // drift がない場合は更新不要として Unchanged を返す
        if !drift_detected {
            // drift がないため Unchanged アクションで SyncResult を返す
            return Ok(SyncResult {
                // artifact_id を設定する
                artifact_id: artifact.artifact_id.clone(),
                // 更新不要アクションを設定する
                action: SyncAction::Unchanged,
                // drift なし
                drift_detected: false,
            });
        }

        // drift が検出された場合は additive_only policy を検証する
        // リモートの content を取得して field 削除チェックを行う
        let url = format!(
            "{}/groups/{}/artifacts/{}",
            self.apicurio_url, self.group, artifact.artifact_id
        );
        // リモートの content をテキストとして取得する
        let remote_response = self
            .client
            // GET メソッドでリモート content を取得する
            .get(&url)
            // Accept ヘッダーには */* を指定する
            .header("Accept", "*/*")
            // リクエストを送信する
            .send()
            .await
            // reqwest エラーを anyhow エラーに変換する
            .map_err(|e| anyhow!("additive_only チェック: リモート content 取得失敗: {}", e))?;

        // リモートの content をテキストとして取得する
        let remote_content = remote_response
            .text()
            .await
            // テキスト取得エラーを anyhow エラーに変換する
            .map_err(|e| anyhow!("additive_only チェック: リモート content テキスト取得失敗: {}", e))?;

        // additive_only policy を検証する（フィールド削除チェック）
        if let Err(policy_err) = Self::check_additive_only(&remote_content, &artifact.content) {
            // additive_only policy 違反のため DriftBlocked アクションで SyncResult を返す
            // エラーは呼び出し元にログ出力を委ねる（SyncResult の action で判断する）
            let _ = policy_err;
            // DriftBlocked アクションで SyncResult を返す
            return Ok(SyncResult {
                // artifact_id を設定する
                artifact_id: artifact.artifact_id.clone(),
                // DriftBlocked アクションを設定する
                action: SyncAction::DriftBlocked,
                // drift は検出されているが policy により blocked
                drift_detected: true,
            });
        }

        // additive_only policy 通過後に artifact バージョンを更新する
        self.update_artifact_version(artifact).await?;

        // Updated アクションで SyncResult を返す
        Ok(SyncResult {
            // artifact_id を設定する
            artifact_id: artifact.artifact_id.clone(),
            // 更新アクションを設定する
            action: SyncAction::Updated,
            // drift が検出されて更新した
            drift_detected: true,
        })
    }

    // ============================================================
    // sync_all_from_yaml_dir: ディレクトリ内の全 YAML を同期する
    // ============================================================

    // sync_all_from_yaml_dir はディレクトリ内の全 .yaml ファイルを読み込んで sync_schema を呼び出す
    // yaml_dir: スキーマ YAML ファイルが格納されたディレクトリパス
    // Returns: 各 artifact の同期結果のベクター
    pub async fn sync_all_from_yaml_dir(&self, yaml_dir: &Path) -> Result<Vec<SyncResult>> {
        // 結果を格納するベクターを初期化する
        let mut results: Vec<SyncResult> = Vec::new();

        // ディレクトリが存在するか確認する
        if !yaml_dir.is_dir() {
            // ディレクトリが存在しない場合はエラーを返す
            return Err(anyhow!(
                "sync_all_from_yaml_dir: ディレクトリが存在しません: {}",
                yaml_dir.display()
            ));
        }

        // ディレクトリ内のエントリを走査する
        let mut entries = tokio::fs::read_dir(yaml_dir).await
            // ディレクトリ読み込みエラーを anyhow エラーに変換する
            .map_err(|e| anyhow!("sync_all_from_yaml_dir: ディレクトリ読み込み失敗: {}", e))?;

        // 各エントリを処理する
        while let Some(entry) = entries.next_entry().await
            // エントリ取得エラーを anyhow エラーに変換する
            .map_err(|e| anyhow!("sync_all_from_yaml_dir: エントリ取得失敗: {}", e))?
        {
            // ファイルパスを取得する
            let path = entry.path();

            // .yaml 拡張子のファイルのみを処理する
            if path.extension().and_then(|e| e.to_str()) != Some("yaml") {
                // .yaml 拡張子でないファイルはスキップする
                continue;
            }

            // YAML ファイルを読み込む
            let content = tokio::fs::read_to_string(&path).await
                // ファイル読み込みエラーを anyhow エラーに変換する
                .map_err(|e| anyhow!(
                    "sync_all_from_yaml_dir: ファイル読み込み失敗: {}, エラー: {}",
                    path.display(),
                    e
                ))?;

            // YAML を SchemaArtifact にデシリアライズする
            let artifact: SchemaArtifact = serde_yaml::from_str(&content)
                // デシリアライズエラーを anyhow エラーに変換する
                .map_err(|e| anyhow!(
                    "sync_all_from_yaml_dir: YAML パース失敗: {}, エラー: {}",
                    path.display(),
                    e
                ))?;

            // artifact を Apicurio Registry に同期する
            let result = self.sync_schema(&artifact).await?;
            // 同期結果をベクターに追加する
            results.push(result);
        }

        // 全 artifact の同期結果を返す
        Ok(results)
    }
}

// ============================================================
// ユニットテスト
// ============================================================

// apicurio_gitops のユニットテスト
#[cfg(test)]
mod tests {
    // 親モジュールの全シンボルをインポートする
    use super::*;

    // ApicurioGitOpsSyncer::from_env のテスト（環境変数なし = デフォルト値を確認する）
    #[test]
    fn test_from_env_defaults() {
        // 環境変数を削除してデフォルト値を使用する
        std::env::remove_var("APICURIO_URL");
        // APICURIO_GROUP も削除する
        std::env::remove_var("APICURIO_GROUP");
        // from_env を呼び出してデフォルト値を確認する
        let syncer = ApicurioGitOpsSyncer::from_env().expect("from_env は成功するはず");
        // デフォルト URL が正しく設定されていることを確認する
        assert_eq!(syncer.apicurio_url, "http://apicurio:8080/apis/registry/v2");
        // デフォルト group が正しく設定されていることを確認する
        assert_eq!(syncer.group, "default");
    }

    // check_additive_only: フィールド削除がない場合は Ok を返すことを確認するテスト
    #[test]
    fn test_check_additive_only_no_removal() {
        // リモートのスキーマ（フィールド: name）
        let remote = r#"{"type": "object", "properties": {"name": {"type": "string"}}}"#;
        // 新しいスキーマ（フィールドを追加）
        let new_content = r#"{"type": "object", "properties": {"name": {"type": "string"}, "age": {"type": "integer"}}}"#;
        // additive_only チェック: フィールド追加のみで削除なし → Ok を期待する
        let result = ApicurioGitOpsSyncer::check_additive_only(remote, new_content);
        // Ok を期待する
        assert!(result.is_ok(), "フィールド追加のみの場合は Ok を返すべき");
    }

    // check_additive_only: フィールド削除がある場合は Err を返すことを確認するテスト
    #[test]
    fn test_check_additive_only_with_removal() {
        // リモートのスキーマ（フィールド: name, age）
        let remote = r#"{"type": "object", "properties": {"name": {"type": "string"}, "age": {"type": "integer"}}}"#;
        // 新しいスキーマ（age フィールドが削除されている）
        let new_content = r#"{"type": "object", "properties": {"name": {"type": "string"}}}"#;
        // additive_only チェック: age フィールド削除 → Err を期待する
        let result = ApicurioGitOpsSyncer::check_additive_only(remote, new_content);
        // Err を期待する
        assert!(result.is_err(), "フィールド削除がある場合は Err を返すべき");
    }

    // SyncAction の PartialEq テスト
    #[test]
    fn test_sync_action_equality() {
        // Created 同士は等しい
        assert_eq!(SyncAction::Created, SyncAction::Created);
        // Updated 同士は等しい
        assert_eq!(SyncAction::Updated, SyncAction::Updated);
        // Unchanged 同士は等しい
        assert_eq!(SyncAction::Unchanged, SyncAction::Unchanged);
        // DriftBlocked 同士は等しい
        assert_eq!(SyncAction::DriftBlocked, SyncAction::DriftBlocked);
        // Created と Updated は等しくない
        assert_ne!(SyncAction::Created, SyncAction::Updated);
    }
}
