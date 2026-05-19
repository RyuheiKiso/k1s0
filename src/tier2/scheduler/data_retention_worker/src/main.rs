// k1s0 tier2 data-retention-worker: パーティションテーブル保持期限管理バイナリ
// RETENTION_DAYS 日より古い audit_events_YYYY_MM パーティションを DROP TABLE IF EXISTS で削除する
// 実行成功時は exit code 0、エラー時は exit code 1 で終了する

// tokio: 非同期ランタイムのエントリポイントマクロに使用する
use tokio;
// sqlx: PostgreSQL 接続プールおよびクエリ実行に使用する
use sqlx::postgres::PgPoolOptions;
// tracing: 構造化ロギング（info! / warn! / error! マクロ）に使用する
use tracing::{info, warn, error};
// tracing_subscriber: stdout へのログ出力バックエンドの初期化に使用する
use tracing_subscriber;
// anyhow: エラーハンドリング（Result 型の統合 error context）に使用する
use anyhow::{Context, Result};
// chrono: 現在年月および保持期限の月計算に使用する
use chrono::{Utc, Datelike, Duration};
// std::env: 環境変数 DATABASE_URL / RETENTION_DAYS の読み取りに使用する
use std::env;
// std::process: exit code を明示的に設定して終了するために使用する
use std::process;

// main 関数: tokio 非同期エントリポイント（#[tokio::main] を使用する）
#[tokio::main]
async fn main() {
    // tracing_subscriber を初期化して構造化ロギングを有効化する
    tracing_subscriber::fmt()
        // タイムスタンプ付きで stdout にテキスト出力する
        .with_target(false)
        // 初期化を実行する
        .init();

    // run() を呼び出して結果に応じて exit code を設定する
    match run().await {
        // 正常終了: exit code 0 で終了する
        Ok(()) => {
            // 正常完了のログを出力する
            info!("data-retention-worker completed successfully");
            // 明示的に exit code 0 で終了する
            process::exit(0);
        }
        // エラー終了: exit code 1 で終了する
        Err(e) => {
            // エラー内容を error レベルでログ出力する
            error!("data-retention-worker failed: {:?}", e);
            // 明示的に exit code 1 で終了する
            process::exit(1);
        }
    }
}

// run: 保持期限超過パーティションの削除ロジック（非同期 / anyhow::Result を返す）
async fn run() -> Result<()> {
    // DATABASE_URL 環境変数を読み取る（未設定時はエラーで終了する）
    let database_url = env::var("DATABASE_URL")
        .context("DATABASE_URL environment variable is not set")?;

    // RETENTION_DAYS 環境変数を読み取る（未設定時はエラーで終了する）
    let retention_days_str = env::var("RETENTION_DAYS")
        .context("RETENTION_DAYS environment variable is not set")?;

    // RETENTION_DAYS を u32 にパースする（数値以外の値が設定された場合はエラーで終了する）
    let retention_days: u64 = retention_days_str
        .parse::<u64>()
        .context("RETENTION_DAYS must be a non-negative integer")?;

    // 設定値をログ出力して動作確認しやすくする
    info!(retention_days = retention_days, "starting data retention worker");

    // PostgreSQL 接続プールを生成する（最大接続数 2 — バッチ処理のため最小限に抑える）
    let pool = PgPoolOptions::new()
        // 最大接続数を 2 に制限する（バッチワーカーは高頻度接続が不要なため）
        .max_connections(2)
        // DATABASE_URL に接続する
        .connect(&database_url)
        .await
        .context("failed to connect to PostgreSQL")?;

    // 接続成功ログを出力する
    info!("connected to PostgreSQL");

    // 現在時刻（UTC）を取得して削除対象の月境界を計算する
    let now_utc = Utc::now();

    // 保持期限を日数で Duration に変換する
    let retention_duration = Duration::days(retention_days as i64);

    // 現在時刻から保持期限分だけ過去に遡った境界日時を計算する
    let cutoff_date = now_utc - retention_duration;

    // 削除対象の年月ペアを収集するためのベクターを初期化する
    // 最大 5 年分（60 ヶ月）を上限として古い月から新しい月に向かって列挙する
    let mut partitions_to_drop: Vec<(i32, u32)> = Vec::new();

    // 調査開始月: cutoff_date の 5 年前から境界月まで走査する
    // 実運用では最大で数十ヶ月分のパーティションが存在する想定
    let scan_start = cutoff_date - Duration::days(5 * 365);

    // 走査開始年月を初期化する
    let mut scan_year = scan_start.year();
    // 走査開始月を初期化する
    let mut scan_month = scan_start.month();

    // 境界月（cutoff_date の年月）まで走査する
    loop {
        // 現在の走査年月が境界日時の年月以前であることを確認する
        let is_before_cutoff =
            // 年が境界年より小さい場合は確実に対象
            scan_year < cutoff_date.year()
            // 同一年の場合は月が境界月以前の場合を対象とする
            || (scan_year == cutoff_date.year() && scan_month <= cutoff_date.month());

        // 境界日時より古い月のパーティションを削除対象リストに追加する
        if is_before_cutoff {
            // 削除対象の年月ペアをベクターに追加する
            partitions_to_drop.push((scan_year, scan_month));
        } else {
            // 境界月を超えたらループを終了する
            break;
        }

        // 次の月に進める
        if scan_month == 12 {
            // 12 月の場合は翌年の 1 月に繰り上げる
            scan_year += 1;
            // 月を 1 月にリセットする
            scan_month = 1;
        } else {
            // それ以外は月を 1 進める
            scan_month += 1;
        }

        // 無限ループを防ぐため 60 ヶ月を上限として走査を打ち切る
        if partitions_to_drop.len() >= 60 {
            // 上限に達したことを警告ログとして記録する
            warn!("scan limit reached (60 months), stopping scan");
            // ループを抜ける
            break;
        }
    }

    // 削除対象パーティションが存在しない場合はスキップして正常終了する
    if partitions_to_drop.is_empty() {
        // 削除対象なしのログを出力する
        info!("no partitions to drop");
        // 正常終了する
        return Ok(());
    }

    // 削除対象パーティション一覧をログ出力する
    info!(count = partitions_to_drop.len(), "partitions eligible for drop");

    // 削除成功・スキップカウンターを初期化する
    let mut dropped_count = 0u32;
    // エラーカウンターを初期化する
    let mut error_count = 0u32;

    // 各パーティションに対して DROP TABLE IF EXISTS を実行する
    for (year, month) in &partitions_to_drop {
        // パーティションテーブル名を audit_events_YYYY_MM 形式で生成する
        let table_name = format!("audit_events_{:04}_{:02}", year, month);

        // DROP TABLE IF EXISTS を実行する（テーブルが存在しない場合は無視する）
        let drop_sql = format!("DROP TABLE IF EXISTS {}", table_name);

        // DROP TABLE IF EXISTS クエリを実行する
        match sqlx::query(&drop_sql)
            // 接続プールを使用してクエリを実行する
            .execute(&pool)
            .await
        {
            // 実行成功: ログを出力して成功カウンターを増やす
            Ok(_) => {
                // 削除成功ログを出力する
                info!(table = %table_name, "dropped partition table");
                // 成功カウンターをインクリメントする
                dropped_count += 1;
            }
            // 実行失敗: エラーログを出力してエラーカウンターを増やす（他パーティションは継続する）
            Err(e) => {
                // エラーログを出力する
                error!(table = %table_name, error = %e, "failed to drop partition table");
                // エラーカウンターをインクリメントする
                error_count += 1;
            }
        }
    }

    // 実行結果のサマリーをログ出力する
    info!(
        dropped = dropped_count,
        errors = error_count,
        "data retention worker finished"
    );

    // エラーが 1 件以上あった場合は anyhow::bail! でエラーを返す
    if error_count > 0 {
        // エラー件数を含むエラーメッセージを返す
        anyhow::bail!(
            "{} partition(s) failed to drop; check logs for details",
            error_count
        );
    }

    // 全パーティションの削除が成功した場合は Ok を返す
    Ok(())
}
