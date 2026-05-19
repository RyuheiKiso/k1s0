// webhook_handler.rs — k1s0 tier2 Mattermost ChatOps webhook ハンドラー
// Mattermost スラッシュコマンドを受信して Argo CD API にディスパッチする
// slash_commands.yaml の定義に従い /k1s0 deploy / rollback / status を処理する

// anyhow クレートのインポート: エラーハンドリングに使用する
use anyhow::{anyhow, Result};
// serde Deserialize のインポート: JSON / フォームデコードに使用する
use serde::Deserialize;
// std::env のインポート: 環境変数からの設定取得に使用する
use std::env;

// MattermostSlashPayload は Mattermost スラッシュコマンドのリクエストペイロード構造体
// Mattermost が送信する application/x-www-form-urlencoded ボディに対応する
#[derive(Debug, Deserialize)]
pub struct MattermostSlashPayload {
    // token は Mattermost スラッシュコマンドのバリデーショントークン
    pub token: String,
    // team_id は コマンドを発行した Mattermost チームの ID
    pub team_id: String,
    // command は スラッシュコマンド名（例: /k1s0）
    pub command: String,
    // text は コマンドに続くテキスト引数（例: "deploy tier2-api"）
    pub text: String,
}

// ArgoCdSyncRequest は Argo CD アプリケーション同期リクエストの構造体
#[derive(Debug, serde::Serialize)]
struct ArgoCdSyncRequest {
    // revision は 同期対象のリビジョン（HEAD またはタグ名）
    revision: String,
    // prune は 不要なリソースを削除するかどうかのフラグ
    prune: bool,
    // dry_run は ドライランモードで実行するかのフラグ
    dry_run: bool,
}

// ArgoCdRollbackRequest は Argo CD アプリケーションロールバックリクエストの構造体
#[derive(Debug, serde::Serialize)]
struct ArgoCdRollbackRequest {
    // id は ロールバック先のヒストリー ID
    id: i64,
    // prune は 不要なリソースを削除するかどうかのフラグ
    prune: bool,
    // dry_run は ドライランモードで実行するかのフラグ
    dry_run: bool,
}

// MattermostResponse は Mattermost に返すレスポンスの構造体
#[derive(Debug, serde::Serialize)]
struct MattermostResponse {
    // response_type は レスポンスの種別（"in_channel" または "ephemeral"）
    response_type: String,
    // text は チャンネルに表示するメッセージテキスト
    text: String,
}

// validate_token は Mattermost スラッシュコマンドのトークンを検証する
// 環境変数 MATTERMOST_SLASH_TOKEN と一致しない場合はエラーを返す
fn validate_token(token: &str) -> Result<()> {
    // 環境変数からバリデーショントークンを取得する
    let expected = env::var("MATTERMOST_SLASH_TOKEN")
        // 環境変数が設定されていない場合はエラーを返す
        .map_err(|_| anyhow!("MATTERMOST_SLASH_TOKEN 環境変数が未設定"))?;
    // トークンが一致しない場合は認証エラーを返す
    if token != expected {
        // 不正なトークンエラーを返す
        return Err(anyhow!("Mattermost スラッシュコマンドトークン検証失敗"));
    }
    // 検証成功を返す
    Ok(())
}

// get_argocd_base_url は環境変数から Argo CD API の base URL を取得する
fn get_argocd_base_url() -> String {
    // 環境変数 ARGOCD_SERVER_URL から Argo CD サーバー URL を取得する
    env::var("ARGOCD_SERVER_URL")
        // 未設定の場合は k1s0 内部 Argo CD サービスのデフォルト URL を使用する
        .unwrap_or_else(|_| "http://argocd-server.k1s0-ops.svc:8080".to_string())
}

// get_argocd_token は環境変数から Argo CD API トークンを取得する
fn get_argocd_token() -> Result<String> {
    // 環境変数 ARGOCD_API_TOKEN から Argo CD API トークンを取得する
    env::var("ARGOCD_API_TOKEN")
        // 未設定の場合はエラーを返す
        .map_err(|_| anyhow!("ARGOCD_API_TOKEN 環境変数が未設定"))
}

// dispatch_deploy は Argo CD の sync API を呼び出してデプロイを実行する
// POST /api/v1/applications/{app_name}/sync を呼び出す
async fn dispatch_deploy(app_name: &str, image_tag: Option<&str>) -> Result<String> {
    // Argo CD base URL を取得する
    let base_url = get_argocd_base_url();
    // Argo CD API トークンを取得する
    let token = get_argocd_token()?;
    // sync リクエストの URL を構築する
    let url = format!("{}/api/v1/applications/{}/sync", base_url, app_name);
    // sync リクエストボディを構築する
    let req_body = ArgoCdSyncRequest {
        // image_tag が指定された場合はそのタグに同期し、そうでなければ HEAD を使用する
        revision: image_tag.unwrap_or("HEAD").to_string(),
        // 不要なリソースは削除しない（安全側の設定）
        prune: false,
        // ドライランモードは無効
        dry_run: false,
    };
    // HTTP クライアントを生成する
    let client = reqwest::Client::new();
    // Argo CD sync API に POST リクエストを送信する
    let response = client
        // URL を設定する
        .post(&url)
        // Bearer トークン認証ヘッダーを設定する
        .header("Authorization", format!("Bearer {}", token))
        // JSON リクエストボディを設定する
        .json(&req_body)
        // リクエストを送信する
        .send()
        .await
        // リクエスト送信エラーをラップして返す
        .map_err(|e| anyhow!("Argo CD sync リクエスト送信失敗: {}", e))?;
    // HTTP ステータスを確認する
    let status = response.status();
    // レスポンスボディを取得する
    let body = response
        .text()
        .await
        // ボディ取得エラーをラップして返す
        .map_err(|e| anyhow!("Argo CD sync レスポンスボディ取得失敗: {}", e))?;
    // ステータスが成功（2xx）以外の場合はエラーを返す
    if !status.is_success() {
        // Argo CD sync エラーをラップして返す
        return Err(anyhow!(
            "Argo CD sync 失敗: HTTP {} body={}",
            status,
            body
        ));
    }
    // デプロイ開始メッセージを返す
    Ok(format!(
        ":rocket: `{}` のデプロイを開始しました（revision: {}）",
        app_name,
        image_tag.unwrap_or("HEAD")
    ))
}

// dispatch_rollback は Argo CD の rollback API を呼び出して前バージョンに戻す
// POST /api/v1/applications/{app_name}/rollback を呼び出す
async fn dispatch_rollback(app_name: &str) -> Result<String> {
    // Argo CD base URL を取得する
    let base_url = get_argocd_base_url();
    // Argo CD API トークンを取得する
    let token = get_argocd_token()?;
    // rollback リクエストの URL を構築する
    let url = format!("{}/api/v1/applications/{}/rollback", base_url, app_name);
    // rollback リクエストボディを構築する（id=0 は直前のバージョンを意味する）
    let req_body = ArgoCdRollbackRequest {
        // 直前のヒストリー ID にロールバックする（0 は最新の前）
        id: 0,
        // 不要なリソースは削除しない
        prune: false,
        // ドライランモードは無効
        dry_run: false,
    };
    // HTTP クライアントを生成する
    let client = reqwest::Client::new();
    // Argo CD rollback API に POST リクエストを送信する
    let response = client
        // URL を設定する
        .post(&url)
        // Bearer トークン認証ヘッダーを設定する
        .header("Authorization", format!("Bearer {}", token))
        // JSON リクエストボディを設定する
        .json(&req_body)
        // リクエストを送信する
        .send()
        .await
        // リクエスト送信エラーをラップして返す
        .map_err(|e| anyhow!("Argo CD rollback リクエスト送信失敗: {}", e))?;
    // HTTP ステータスを確認する
    let status = response.status();
    // レスポンスボディを取得する
    let body = response
        .text()
        .await
        // ボディ取得エラーをラップして返す
        .map_err(|e| anyhow!("Argo CD rollback レスポンスボディ取得失敗: {}", e))?;
    // ステータスが成功（2xx）以外の場合はエラーを返す
    if !status.is_success() {
        // Argo CD rollback エラーをラップして返す
        return Err(anyhow!(
            "Argo CD rollback 失敗: HTTP {} body={}",
            status,
            body
        ));
    }
    // ロールバック開始メッセージを返す
    Ok(format!(
        ":arrow_left: `{}` のロールバックを開始しました（前バージョンに戻します）",
        app_name
    ))
}

// dispatch_status は Argo CD のアプリケーション情報 API を呼び出してデプロイ状況を取得する
// GET /api/v1/applications/{app_name} を呼び出す
async fn dispatch_status(app_name: &str) -> Result<String> {
    // Argo CD base URL を取得する
    let base_url = get_argocd_base_url();
    // Argo CD API トークンを取得する
    let token = get_argocd_token()?;
    // applications GET エンドポイント URL を構築する
    let url = format!("{}/api/v1/applications/{}", base_url, app_name);
    // HTTP クライアントを生成する
    let client = reqwest::Client::new();
    // Argo CD アプリケーション情報 API に GET リクエストを送信する
    let response = client
        // URL を設定する
        .get(&url)
        // Bearer トークン認証ヘッダーを設定する
        .header("Authorization", format!("Bearer {}", token))
        // リクエストを送信する
        .send()
        .await
        // リクエスト送信エラーをラップして返す
        .map_err(|e| anyhow!("Argo CD status リクエスト送信失敗: {}", e))?;
    // HTTP ステータスを確認する
    let status = response.status();
    // レスポンスボディを取得する
    let body = response
        .text()
        .await
        // ボディ取得エラーをラップして返す
        .map_err(|e| anyhow!("Argo CD status レスポンスボディ取得失敗: {}", e))?;
    // ステータスが成功（2xx）以外の場合はエラーを返す
    if !status.is_success() {
        // Argo CD status エラーをラップして返す
        return Err(anyhow!(
            "Argo CD status 取得失敗: HTTP {} body={}",
            status,
            body
        ));
    }
    // ステータスメッセージを返す（JSON ボディは簡略化してそのまま表示する）
    Ok(format!(":information_source: `{}` のステータス:\n```\n{}\n```", app_name, body))
}

// parse_subcommand_and_app は payload.text をパースしてサブコマンドとアプリ名を取得する
// 例: "deploy tier2-api" → ("deploy", "tier2-api", None)
//     "deploy tier2-api --tag v1.2.3" → ("deploy", "tier2-api", Some("v1.2.3"))
fn parse_subcommand_and_app(text: &str) -> Result<(&str, &str, Option<&str>)> {
    // テキストをスペースで分割してトークン列を取得する
    let tokens: Vec<&str> = text.split_whitespace().collect();
    // トークンが 2 個未満の場合は引数不足エラーを返す
    if tokens.len() < 2 {
        // 引数不足エラーを返す
        return Err(anyhow!(
            "使用方法: /k1s0 <deploy|rollback|status> <service-name> [--tag image-tag]"
        ));
    }
    // 最初のトークンがサブコマンド
    let subcommand = tokens[0];
    // 2 番目のトークンがアプリケーション名
    let app_name = tokens[1];
    // --tag オプションを検索する
    let image_tag = tokens
        // トークン列を走査する
        .windows(2)
        // --tag フラグを持つウィンドウを検索する
        .find(|w| w[0] == "--tag")
        // 見つかった場合は次のトークンをタグ名として返す
        .map(|w| w[1]);
    // サブコマンド、アプリ名、タグを返す
    Ok((subcommand, app_name, image_tag))
}

// handle_slash_command は Mattermost スラッシュコマンドペイロードを受け取り、
// Argo CD API にディスパッチして結果メッセージを返す
// /k1s0 deploy / rollback / status の 3 サブコマンドを処理する
pub async fn handle_slash_command(payload: MattermostSlashPayload) -> Result<String> {
    // トークンを検証する（不正なリクエストを早期に拒否する）
    validate_token(&payload.token)?;
    // command フィールドが /k1s0 であることを確認する
    if payload.command != "/k1s0" {
        // 不明なコマンドエラーを返す
        return Err(anyhow!("不明なコマンド: {}", payload.command));
    }
    // テキストを正規化してトリムする
    let text = payload.text.trim();
    // テキストが空の場合は使用方法を返す
    if text.is_empty() {
        // 使用方法メッセージを返す
        return Ok(
            "使用方法: `/k1s0 <deploy|rollback|status> <service-name> [--tag image-tag]`\n\
            例: `/k1s0 deploy tier2-api --tag v1.2.3`"
                .to_string(),
        );
    }
    // サブコマンドとアプリ名をパースする
    let (subcommand, app_name, image_tag) = parse_subcommand_and_app(text)?;
    // サブコマンドに応じた処理をディスパッチする
    let message = match subcommand {
        // deploy サブコマンド: Argo CD sync API を呼び出す
        "deploy" => dispatch_deploy(app_name, image_tag).await?,
        // rollback サブコマンド: Argo CD rollback API を呼び出す
        "rollback" => dispatch_rollback(app_name).await?,
        // status サブコマンド: Argo CD applications API を呼び出す
        "status" => dispatch_status(app_name).await?,
        // 不明なサブコマンドはエラーを返す
        unknown => {
            // 不明なサブコマンドエラーを返す
            return Err(anyhow!(
                "不明なサブコマンド: `{}`. deploy / rollback / status のいずれかを指定してください",
                unknown
            ));
        }
    };
    // 処理結果メッセージを返す
    Ok(message)
}

// handle_slash_command_to_json は Mattermost に返す JSON レスポンス文字列を生成する
// handle_slash_command の結果を MattermostResponse JSON にラップして返す
pub async fn handle_slash_command_to_json(payload: MattermostSlashPayload) -> String {
    // スラッシュコマンドを処理する
    match handle_slash_command(payload).await {
        // 成功した場合は in_channel レスポンスを返す
        Ok(text) => {
            // MattermostResponse を構築する
            let response = MattermostResponse {
                // チャンネル全体に表示する
                response_type: "in_channel".to_string(),
                // 処理結果メッセージを設定する
                text,
            };
            // JSON にシリアライズして返す
            serde_json::to_string(&response)
                // シリアライズ失敗時はフォールバックメッセージを返す
                .unwrap_or_else(|_| r#"{"response_type":"ephemeral","text":"内部エラー"}"#.to_string())
        }
        // エラーの場合は ephemeral レスポンスでエラーメッセージを返す
        Err(e) => {
            // エラー MattermostResponse を構築する
            let response = MattermostResponse {
                // コマンド発行者のみに表示する（in_channel には出さない）
                response_type: "ephemeral".to_string(),
                // エラーメッセージを設定する
                text: format!(":warning: エラー: {}", e),
            };
            // JSON にシリアライズして返す
            serde_json::to_string(&response)
                // シリアライズ失敗時はフォールバックメッセージを返す
                .unwrap_or_else(|_| r#"{"response_type":"ephemeral","text":"内部エラー"}"#.to_string())
        }
    }
}
