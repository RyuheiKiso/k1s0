/// HIGH-RUST-001 / CRIT-003 Phase B: 全シークレットの AAD 付き再暗号化バッチ処理。
///
/// 目的: vault.secret_versions に AAD なし（旧形式）で暗号化された行が存在する場合、
/// AAD あり（新形式）で再暗号化することで decrypt_with_legacy_fallback を不要にする。
///
/// 実行前提:
///   - DATABASE_URL 環境変数に vault-db の接続文字列を設定すること
///   - VAULT_MASTER_KEY 環境変数に現在稼働中と同じ鍵を設定すること
///   - DB 接続ロールは row_security = off を実行できる権限（SUPERUSER または BYPASSRLS）が必要
///
/// 実行手順:
///   1. vault サービスを停止する（書き込み競合を回避）
///   2. このバイナリを実行してすべてのシークレットを再暗号化する
///   3. 再暗号化が完了したことを確認する
///   4. 新しい vault サービス（decrypt_with_legacy_fallback 削除済み）をデプロイする
///
/// 使用例:
///   cargo run --bin migrate-secrets

use sqlx::PgPool;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // ロギング初期化
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("migrate_secrets=info".parse()?),
        )
        .init();

    let database_url = std::env::var("DATABASE_URL")
        .map_err(|_| anyhow::anyhow!("DATABASE_URL 環境変数が設定されていません"))?;

    tracing::info!("vault-db に接続しています...");
    let pool = PgPool::connect(&database_url).await?;

    let master_key = k1s0_vault_server::infrastructure::encryption::MasterKey::from_env()?;

    // RLS を無効化して全テナントのシークレットを読み書きする。
    // このコマンドは SUPERUSER または BYPASSRLS 権限が必要。
    tracing::info!("RLS を一時的に無効化します（BYPASSRLS 権限が必要）");
    sqlx::query("SET row_security = off")
        .execute(&pool)
        .await
        .map_err(|e| {
            anyhow::anyhow!(
                "row_security の無効化に失敗しました。SUPERUSER または BYPASSRLS 権限が必要です: {}",
                e
            )
        })?;

    // 全シークレットバージョンをパスと共に取得する
    // sqlx::query_as! マクロは DATABASE_URL が必要なため、
    // 実行時型マッピングを使う sqlx::query_as 関数を使用する
    #[derive(sqlx::FromRow)]
    struct SecretVersionRecord {
        id: uuid::Uuid,
        key_path: String,
        version: i32,
        encrypted_data: Vec<u8>,
        nonce: Vec<u8>,
    }

    let rows: Vec<SecretVersionRecord> = sqlx::query_as(
        "SELECT sv.id, s.key_path, sv.version, sv.encrypted_data, sv.nonce \
         FROM vault.secret_versions sv \
         JOIN vault.secrets s ON sv.secret_id = s.id \
         ORDER BY s.key_path, sv.version",
    )
    .fetch_all(&pool)
    .await?;

    tracing::info!("対象シークレットバージョン数: {}", rows.len());

    let mut migrated = 0usize;
    let mut already_new_format = 0usize;
    let mut errors = 0usize;

    for row in &rows {
        let aad = row.key_path.as_bytes();

        // まず新形式（AAD あり）で復号を試みる: 成功すれば既に新形式のためスキップ
        if master_key.decrypt(&row.encrypted_data, &row.nonce, aad).is_ok() {
            already_new_format += 1;
            continue;
        }

        // 旧形式（AAD なし）で復号する
        let plaintext = match master_key.decrypt(&row.encrypted_data, &row.nonce, b"") {
            Ok(pt) => pt,
            Err(e) => {
                tracing::error!(
                    key_path = %row.key_path,
                    version = row.version,
                    error = %e,
                    "復号に失敗しました（新形式・旧形式ともに失敗）。このレコードをスキップします"
                );
                errors += 1;
                continue;
            }
        };

        // 新形式（AAD あり）で再暗号化する
        let (new_encrypted_data, new_nonce) = match master_key.encrypt(&plaintext, aad) {
            Ok(pair) => pair,
            Err(e) => {
                tracing::error!(
                    key_path = %row.key_path,
                    version = row.version,
                    error = %e,
                    "再暗号化に失敗しました"
                );
                errors += 1;
                continue;
            }
        };

        // DB を更新する（RLS は row_security = off で無効化済み）
        match sqlx::query(
            "UPDATE vault.secret_versions \
             SET encrypted_data = $1, nonce = $2 \
             WHERE id = $3",
        )
        .bind(&new_encrypted_data)
        .bind(&new_nonce)
        .bind(row.id)
        .execute(&pool)
        .await
        {
            Ok(_) => {
                tracing::info!(
                    key_path = %row.key_path,
                    version = row.version,
                    "再暗号化完了"
                );
                migrated += 1;
            }
            Err(e) => {
                tracing::error!(
                    key_path = %row.key_path,
                    version = row.version,
                    error = %e,
                    "DB 更新に失敗しました"
                );
                errors += 1;
            }
        }
    }

    // RLS を元に戻す
    sqlx::query("SET row_security = on")
        .execute(&pool)
        .await?;

    tracing::info!(
        migrated = migrated,
        already_new_format = already_new_format,
        errors = errors,
        "再暗号化完了"
    );

    if errors > 0 {
        return Err(anyhow::anyhow!(
            "{} 件のエラーが発生しました。ログを確認してください",
            errors
        ));
    }

    println!(
        "完了: {} 件を再暗号化, {} 件はすでに新形式",
        migrated, already_new_format
    );

    Ok(())
}
