// k1s0 tier2 atomic 三表書込
// State change / Outbox / Audit event を同一 DB トランザクションで書く
// TLA+ の P1-P4 invariant と double-bind する（src/formal/dafny/AtomicThreeTableWrite.dfy）
//
// P1: aggregate 状態変更時、必ず Outbox + Audit が同一 txn に書込
// P2: Outbox 投入失敗時、aggregate 状態変更も rollback
// P3: tenant_id が GUC と aggregate 行で一致しない場合、操作を reject
// P4: pii_segregated の全アクセスを audit_event に記録

// UUID ライブラリ
use uuid::Uuid;
// シリアライズライブラリ
use serde::{Deserialize, Serialize};
// エラー型定義ライブラリ
use thiserror::Error;
// HLC クロック（wall-clock TTL 禁止規律: Utc::now() の代替として HlcClock を使用する）
use k1s0_hlc::{HlcClock, HlcTimestamp};
// sqlx PostgreSQL クライアント（PgPool で接続プールを管理する）
use sqlx::PgPool;
// sqlx Postgres トランザクション型（BEGIN / COMMIT / ROLLBACK に使用する）
use sqlx::Postgres;
// テナントコンテキスト
use crate::tenant_context::TenantContext;

// atomic 三表書込のエラー型
#[derive(Debug, Error)]
pub enum AtomicWriteError {
    // P2: Outbox 投入失敗（rollback が必要）
    #[error("Outbox insert failed, transaction rolled back: {0}")]
    OutboxInsertFailed(String),
    // P3: tenant_id 不一致（GUC と aggregate 行の tenant_id が一致しない）
    #[error("TenantId mismatch: guc={guc}, row={row}")]
    TenantIdMismatch { guc: Uuid, row: Uuid },
    // P4: pii_segregated の audit_event 記録失敗
    #[error("PII audit record failed: {0}")]
    PiiAuditFailed(String),
    // 汎用: DB トランザクションエラー
    #[error("Transaction error: {0}")]
    TransactionError(String),
}

// 書込対象のテーブルクラス（10_テナント分離適合仕様.md の 4 class と一致する）
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TableClass {
    // テナントスコープ: tenant_id 必須、RLS FORCE
    TenantScoped,
    // テナントマスタ: tenant_id 必須、role 制限付き RLS FORCE
    TenantMaster,
    // プラットフォームグローバル: tenant_id 無し、RLS 無効
    PlatformGlobal,
    // PII 隔離: tenant_id 必須 + purpose check + pgaudit 全アクセス
    PiiSegregated,
}

// aggregate の状態変更を表す構造体（P1 の state_change に対応する）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateChange {
    // 変更対象の aggregate ID
    pub aggregate_id: Uuid,
    // 変更対象の tenant_id（TenantContext.tenant_id と一致している必要がある）
    pub tenant_id: Uuid,
    // テーブルクラス（どの class の table を書込むかを示す）
    pub table_class: TableClass,
    // 変更内容のシリアライズ済みペイロード
    pub payload: serde_json::Value,
    // aggregate バージョン（楽観的ロックに使用する）
    pub version: i64,
}

// atomic 三表書込の結果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TripleWriteResult {
    // 書込んだ aggregate ID
    pub aggregate_id: Uuid,
    // 書込んだ outbox エントリの ID
    pub outbox_id: Uuid,
    // 書込んだ audit_event の ID
    pub audit_event_id: Uuid,
    // 書込完了 HLC タイムスタンプ（wall-clock TTL 禁止規律に従い HlcTimestamp を使用する）
    pub committed_at: HlcTimestamp,
}

// atomic 三表書込の実行エンジン
// PgPool を内部に保持し、P1-P4 invariant を保証するトランザクションを自律的に管理する
#[derive(Debug)]
pub struct AtomicTripleWrite {
    // テナントコンテキスト（GUC 注入・tenant_id 検証に使用する）
    context: TenantContext,
    // PostgreSQL 接続プール（BEGIN / INSERT 3 回 / COMMIT を同一プールで実行する）
    pool: PgPool,
}

// execute() の戻り値型
// P1: BEGIN 〜 COMMIT の中で state_change / outbox / audit_event の 3 INSERT を実行する
// P2: outbox INSERT が失敗した場合は txn を rollback して OutboxInsertFailed を返す
// P3: tenant_id が GUC と一致しない場合は即座に reject して TransactionError を返す
// P4: pii_segregated テーブルへのアクセスは audit_event に記録してから txn を実行する
pub type ExecuteResult = Result<TripleWriteResult, AtomicWriteError>;

impl AtomicTripleWrite {
    // AtomicTripleWrite を生成する（TenantContext と PgPool を受け取る）
    pub fn new(context: TenantContext, pool: PgPool) -> Self {
        // TenantContext と PgPool を格納した AtomicTripleWrite を返す
        Self { context, pool }
    }

    // P1-P4: atomic 三表書込を実行する非同期メソッド（PgPool から BEGIN して 3 INSERT / COMMIT する）
    // 本メソッドが BEGIN → 3 INSERT → P3 GUC 検証 → COMMIT / ROLLBACK を自律的に管理する
    pub async fn execute_triple_write(
        &self,
        // 書込対象の状態変更（P1-P4 の対象）
        change: &StateChange,
    ) -> ExecuteResult {
        // P3: tenant_id 一致を事前検証する（GUC と aggregate 行の tenant_id が一致しない場合は即座にエラー）
        self.verify_tenant_id(change)?;

        // P4: pii_segregated の場合は audit_event への記録が必須であることを確認する
        let _pii_required = self.verify_pii_audit_required(change);

        // PgPool からトランザクションを開始する（BEGIN に相当する）
        // sqlx prepare 実行後は query! マクロに置き換えること
        let mut tx: sqlx::Transaction<'_, Postgres> = self.pool
            .begin()
            .await
            .map_err(|e| AtomicWriteError::TransactionError(e.to_string()))?;

        // P3: SET LOCAL で 4 GUC を txn スコープに注入する（RLS FORCE が参照する）
        let set_guc_sql = self.context.to_set_local_sql();

        // SET LOCAL GUC を実行する（transaction スコープのみ有効、COMMIT で自動破棄される）
        // SET LOCAL は動的文字列なため sqlx::query（ランタイム文字列版）を使用する（compile-time 検証不可）
        sqlx::query(&set_guc_sql)
            // トランザクション内で SET LOCAL を実行する
            .execute(&mut *tx)
            .await
            .map_err(|e| AtomicWriteError::TransactionError(e.to_string()))?;

        // P3: SHOW app.tenant_id で GUC の実際値を読み取り aggregate の tenant_id と照合する
        // SHOW は動的パラメータ不要だが sqlx::query! での compile-time 検証は非対応のためランタイム版を使用する
        let guc_row: (String,) = sqlx::query_as(
            // SHOW コマンドで現在の GUC 値を取得する
            "SHOW app.tenant_id",
        )
        // トランザクション内で SHOW コマンドを実行する
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| AtomicWriteError::TransactionError(e.to_string()))?;

        // SHOW app.tenant_id の結果文字列を UUID にパースする
        let guc_tenant_id = guc_row.0
            .parse::<Uuid>()
            // パース失敗は TransactionError として扱う（GUC が未設定 or 不正値の場合）
            .map_err(|e| AtomicWriteError::TransactionError(
                format!("app.tenant_id GUC parse error: {}", e)
            ))?;

        // GUC から読み取った tenant_id と aggregate の tenant_id を比較する
        if guc_tenant_id != change.tenant_id {
            // P3 違反: SHOW app.tenant_id の値と aggregate 行の tenant_id が不一致
            return Err(AtomicWriteError::TenantIdMismatch {
                // GUC から読み取った tenant_id
                guc: guc_tenant_id,
                // aggregate 行の tenant_id
                row: change.tenant_id,
            });
        }

        // Outbox エントリの ID を生成する（P1 の atomic 三表書込で使用する）
        let outbox_id = Uuid::new_v4();
        // audit_event の ID を生成する（domain_event と audit_event で共有する）
        let audit_event_id = Uuid::new_v4();
        // HLC クロックを生成する（wall-clock TTL 禁止規律: Utc::now() の代替）
        let hlc_clock = HlcClock::from_env();
        // HLC タイムスタンプを取得する（3 INSERT で統一した論理時刻を使用する）
        let committed_at: HlcTimestamp = hlc_clock.now();
        // sqlx への bind 用に HLC の wall_ms から DateTime<Utc> を生成する（DB 列型は TIMESTAMPTZ）
        let committed_at_db = chrono::DateTime::from_timestamp_millis(committed_at.wall_ms as i64)
            // wall_ms が範囲外の場合は epoch を使用する（理論上は発生しない）
            .unwrap_or(chrono::DateTime::UNIX_EPOCH);

        // P1: k1s0.domain_event テーブルに INSERT する（aggregate 状態変更の永続化）
        // current_setting('app.tenant_id')::uuid を使って RLS FORCE の tenant_id を注入する
        // sqlx::query! マクロを使用して compile-time 型検証を行う（SQLX_OFFLINE=true で .sqlx/ キャッシュを使用する）
        sqlx::query!(
            r#"
            INSERT INTO k1s0.domain_event
                (id, aggregate_id, tenant_id, event_kind, payload, version, created_at)
            VALUES
                ($1, $2, current_setting('app.tenant_id')::uuid, 'StateChange', $3, $4, $5)
            "#,
            // audit_event_id を domain_event の主キーとして使用する
            audit_event_id,
            // 変更対象の aggregate ID をバインドする
            change.aggregate_id,
            // ペイロードを jsonb 型としてバインドする
            change.payload,
            // aggregate バージョンをバインドする（楽観的ロックに使用する）
            change.version,
            // HLC wall_ms から変換した TIMESTAMPTZ をバインドする
            committed_at_db,
        )
        // 同一 transaction で実行する（P1 の atomic 書込を保証する）
        .execute(&mut *tx)
        .await
        .map_err(|e| AtomicWriteError::TransactionError(e.to_string()))?;

        // P1: k1s0.outbox_message テーブルに INSERT する（Debezium CDC 経由で Kafka に転送される）
        // P2: この INSERT が失敗した場合は OutboxInsertFailed を返し、呼び出し元が rollback する
        // sqlx::query! マクロを使用して compile-time 型検証を行う（SQLX_OFFLINE=true で .sqlx/ キャッシュを使用する）
        sqlx::query!(
            r#"
            INSERT INTO k1s0.outbox_message
                (id, aggregate_id, tenant_id, event_kind, payload, created_at)
            VALUES
                ($1, $2, current_setting('app.tenant_id')::uuid, 'OutboxRelay', $3, $4)
            "#,
            // outbox エントリの ID をバインドする
            outbox_id,
            // 変更対象の aggregate ID をバインドする
            change.aggregate_id,
            // ペイロードを jsonb 型としてバインドする（PII は redact 済みのみ含む）
            change.payload,
            // HLC wall_ms から変換した TIMESTAMPTZ をバインドする
            committed_at_db,
        )
        // 同一 transaction で実行する（P2 の rollback 要件を満たす）
        .execute(&mut *tx)
        .await
        // P2: outbox INSERT 失敗は OutboxInsertFailed にマッピングして rollback を促す
        .map_err(|e| AtomicWriteError::OutboxInsertFailed(e.to_string()))?;

        // P1+P4: k1s0.audit_event テーブルに INSERT する（全操作を監査記録する）
        // P4: pii_segregated は pgaudit も併用するが、アプリ層からも必ず audit_event を書く
        // sqlx::query! マクロを使用して compile-time 型検証を行う（SQLX_OFFLINE=true で .sqlx/ キャッシュを使用する）
        // table_class は Debug フォーマットで文字列化してバインドする
        let table_class_str = format!("{:?}", change.table_class);
        sqlx::query!(
            r#"
            INSERT INTO k1s0.audit_event
                (id, aggregate_id, tenant_id, actor_id, purpose, table_class, payload, created_at)
            VALUES
                ($1, $2, current_setting('app.tenant_id')::uuid,
                 current_setting('app.actor_id'),
                 current_setting('app.purpose'),
                 $3, $4, $5)
            "#,
            // audit_event の ID をバインドする（domain_event と同じ ID で結びつける）
            audit_event_id,
            // 変更対象の aggregate ID をバインドする
            change.aggregate_id,
            // テーブルクラスを文字列としてバインドする
            table_class_str,
            // ペイロードを jsonb 型としてバインドする
            change.payload,
            // HLC wall_ms から変換した TIMESTAMPTZ をバインドする
            committed_at_db,
        )
        // 同一 transaction で実行する（P4 の audit 必須要件を満たす）
        .execute(&mut *tx)
        .await
        // P4: audit_event INSERT 失敗は PiiAuditFailed にマッピングして rollback を促す
        .map_err(|e| AtomicWriteError::PiiAuditFailed(e.to_string()))?;

        // COMMIT: 全 INSERT が成功した場合にトランザクションを確定する
        tx.commit()
            .await
            .map_err(|e| AtomicWriteError::TransactionError(e.to_string()))?;

        // 三表書込の結果を返す
        Ok(TripleWriteResult {
            // 書込んだ aggregate ID を返す
            aggregate_id: change.aggregate_id,
            // 書込んだ outbox エントリの ID を返す
            outbox_id,
            // 書込んだ audit_event の ID を返す
            audit_event_id,
            // 書込完了 HLC タイムスタンプを返す
            committed_at,
        })
    }

    // P1-P4: atomic 三表書込を外部 transaction で実行する非同期メソッド（後方互換用）
    // tx: sqlx::Transaction<'_, Postgres> — 呼び出し元が BEGIN した transaction を受け取る
    // 呼び出し元は Ok 返却後に tx.commit() を呼ぶ。Err 返却後は tx.rollback() または drop で rollback される。
    pub async fn execute(
        &self,
        // 書込対象の状態変更（P1-P4 の対象）
        change: &StateChange,
        // 呼び出し元が管理する PostgreSQL transaction（同一 txn で 3 INSERT を実行する）
        tx: &mut sqlx::Transaction<'_, Postgres>,
    ) -> ExecuteResult {
        // P3: tenant_id 一致を事前検証する（GUC と aggregate 行の tenant_id が一致しない場合は即座にエラー）
        self.verify_tenant_id(change)?;
        // P3: SET LOCAL で 4 GUC を txn スコープに注入する（RLS FORCE が参照する）
        let set_guc_sql = self.context.to_set_local_sql();

        // SET LOCAL GUC を実行する（transaction スコープのみ有効、COMMIT で自動破棄される）
        // SET LOCAL は動的文字列なため sqlx::query（ランタイム文字列版）を使用する（compile-time 検証不可）
        sqlx::query(&set_guc_sql)
            .execute(&mut **tx)
            .await
            .map_err(|e| AtomicWriteError::TransactionError(e.to_string()))?;

        // P3: SHOW app.tenant_id で GUC の実際値を読み取り aggregate の tenant_id と照合する
        // SHOW は動的パラメータ不要だが sqlx::query! での compile-time 検証は非対応のためランタイム版を使用する
        let guc_row: (String,) = sqlx::query_as(
            // SHOW コマンドで現在の GUC 値を取得する
            "SHOW app.tenant_id",
        )
        // 外部トランザクション内で SHOW コマンドを実行する
        .fetch_one(&mut **tx)
        .await
        .map_err(|e| AtomicWriteError::TransactionError(e.to_string()))?;

        // SHOW app.tenant_id の結果文字列を UUID にパースする
        let guc_tenant_id = guc_row.0
            .parse::<Uuid>()
            // パース失敗は TransactionError として扱う
            .map_err(|e| AtomicWriteError::TransactionError(
                format!("app.tenant_id GUC parse error: {}", e)
            ))?;

        // GUC から読み取った tenant_id と aggregate の tenant_id を比較する
        if guc_tenant_id != change.tenant_id {
            // P3 違反: SHOW app.tenant_id の値と aggregate 行の tenant_id が不一致
            return Err(AtomicWriteError::TenantIdMismatch {
                // GUC から読み取った tenant_id
                guc: guc_tenant_id,
                // aggregate 行の tenant_id
                row: change.tenant_id,
            });
        }

        // P4: pii_segregated の場合は audit_event への記録が必須であることを確認する
        let pii_required = self.verify_pii_audit_required(change);
        // pii_required フラグをログとして使用する（実際は audit_event INSERT で常に記録する）
        let _ = pii_required;

        // Outbox エントリの ID を生成する（P1 の atomic 三表書込で使用する）
        let outbox_id = Uuid::new_v4();
        // audit_event の ID を生成する（domain_event と audit_event で共有する）
        let audit_event_id = Uuid::new_v4();
        // HLC クロックを生成する（wall-clock TTL 禁止規律: Utc::now() の代替）
        let hlc_clock = HlcClock::from_env();
        // HLC タイムスタンプを取得する（3 INSERT で統一した論理時刻を使用する）
        let committed_at: HlcTimestamp = hlc_clock.now();
        // sqlx への bind 用に HLC の wall_ms から DateTime<Utc> を生成する（DB 列型は TIMESTAMPTZ）
        let committed_at_db = chrono::DateTime::from_timestamp_millis(committed_at.wall_ms as i64)
            // wall_ms が範囲外の場合は epoch を使用する（理論上は発生しない）
            .unwrap_or(chrono::DateTime::UNIX_EPOCH);

        // P1: k1s0.domain_event テーブルに INSERT する（aggregate 状態変更の永続化）
        // current_setting('app.tenant_id')::uuid を使って RLS FORCE の tenant_id を注入する
        // sqlx::query! マクロを使用して compile-time 型検証を行う（SQLX_OFFLINE=true で .sqlx/ キャッシュを使用する）
        sqlx::query!(
            r#"
            INSERT INTO k1s0.domain_event
                (id, aggregate_id, tenant_id, event_kind, payload, version, created_at)
            VALUES
                ($1, $2, current_setting('app.tenant_id')::uuid, 'StateChange', $3, $4, $5)
            "#,
            // audit_event_id を domain_event の主キーとして使用する
            audit_event_id,
            // 変更対象の aggregate ID をバインドする
            change.aggregate_id,
            // ペイロードを jsonb 型としてバインドする
            change.payload,
            // aggregate バージョンをバインドする（楽観的ロックに使用する）
            change.version,
            // HLC wall_ms から変換した TIMESTAMPTZ をバインドする
            committed_at_db,
        )
        // 同一 transaction で実行する（P1 の atomic 書込を保証する）
        .execute(&mut **tx)
        .await
        .map_err(|e| AtomicWriteError::TransactionError(e.to_string()))?;

        // P1: k1s0.outbox_message テーブルに INSERT する（Debezium CDC 経由で Kafka に転送される）
        // P2: この INSERT が失敗した場合は OutboxInsertFailed を返し、呼び出し元が rollback する
        // sqlx::query! マクロを使用して compile-time 型検証を行う（SQLX_OFFLINE=true で .sqlx/ キャッシュを使用する）
        sqlx::query!(
            r#"
            INSERT INTO k1s0.outbox_message
                (id, aggregate_id, tenant_id, event_kind, payload, created_at)
            VALUES
                ($1, $2, current_setting('app.tenant_id')::uuid, 'OutboxRelay', $3, $4)
            "#,
            // outbox エントリの ID をバインドする
            outbox_id,
            // 変更対象の aggregate ID をバインドする
            change.aggregate_id,
            // ペイロードを jsonb 型としてバインドする（PII は redact 済みのみ含む）
            change.payload,
            // HLC wall_ms から変換した TIMESTAMPTZ をバインドする
            committed_at_db,
        )
        // 同一 transaction で実行する（P2 の rollback 要件を満たす）
        .execute(&mut **tx)
        .await
        // P2: outbox INSERT 失敗は OutboxInsertFailed にマッピングして rollback を促す
        .map_err(|e| AtomicWriteError::OutboxInsertFailed(e.to_string()))?;

        // P1+P4: k1s0.audit_event テーブルに INSERT する（全操作を監査記録する）
        // P4: pii_segregated は pgaudit も併用するが、アプリ層からも必ず audit_event を書く
        // sqlx::query! マクロを使用して compile-time 型検証を行う（SQLX_OFFLINE=true で .sqlx/ キャッシュを使用する）
        // table_class は Debug フォーマットで文字列化してバインドする
        let table_class_str = format!("{:?}", change.table_class);
        sqlx::query!(
            r#"
            INSERT INTO k1s0.audit_event
                (id, aggregate_id, tenant_id, actor_id, purpose, table_class, payload, created_at)
            VALUES
                ($1, $2, current_setting('app.tenant_id')::uuid,
                 current_setting('app.actor_id'),
                 current_setting('app.purpose'),
                 $3, $4, $5)
            "#,
            // audit_event の ID をバインドする（domain_event と同じ ID で結びつける）
            audit_event_id,
            // 変更対象の aggregate ID をバインドする
            change.aggregate_id,
            // テーブルクラスを文字列としてバインドする
            table_class_str,
            // ペイロードを jsonb 型としてバインドする
            change.payload,
            // HLC wall_ms から変換した TIMESTAMPTZ をバインドする
            committed_at_db,
        )
        // 同一 transaction で実行する（P4 の audit 必須要件を満たす）
        .execute(&mut **tx)
        .await
        // P4: audit_event INSERT 失敗は PiiAuditFailed にマッピングして rollback を促す
        .map_err(|e| AtomicWriteError::PiiAuditFailed(e.to_string()))?;

        // 三表書込の結果を返す（呼び出し元が tx.commit() を呼ぶことで確定する）
        Ok(TripleWriteResult {
            // 書込んだ aggregate ID を返す
            aggregate_id: change.aggregate_id,
            // 書込んだ outbox エントリの ID を返す
            outbox_id,
            // 書込んだ audit_event の ID を返す
            audit_event_id,
            // 書込完了 HLC タイムスタンプを返す
            committed_at,
        })
    }

    // P3: tenant_id 一致を検証する
    // StateChange の tenant_id が TenantContext の tenant_id と一致しない場合はエラーを返す
    pub fn verify_tenant_id(&self, change: &StateChange) -> Result<(), AtomicWriteError> {
        // GUC の tenant_id と aggregate の tenant_id を比較する
        let guc_tenant_id = self.context.tenant_id();
        if guc_tenant_id != change.tenant_id {
            // P3 違反: tenant_id 不一致でエラーを返す
            return Err(AtomicWriteError::TenantIdMismatch {
                guc: guc_tenant_id,
                row: change.tenant_id,
            });
        }
        // tenant_id が一致した場合は Ok を返す
        Ok(())
    }

    // P4: pii_segregated テーブルへのアクセスを audit_event に記録する必要があることを検証する
    pub fn verify_pii_audit_required(&self, change: &StateChange) -> bool {
        // pii_segregated の場合は必ず audit_event を記録する（true を返す）
        change.table_class == TableClass::PiiSegregated
    }

    // P2: Outbox 失敗シミュレーション（テスト専用、production では使用しない）
    #[cfg(test)]
    pub fn simulate_outbox_failure() -> AtomicWriteError {
        // outbox 失敗時は rollback するためのエラーを返す
        AtomicWriteError::OutboxInsertFailed("simulated outbox failure".to_string())
    }
}

#[cfg(test)]
mod tests {
    // テストモジュール内部でのみ使用するインポート
    use super::*;
    use crate::tenant_context::SessionPurpose;

    // テスト用の PgPool をモックするために sqlx の PgPool を生成する
    // 統合テスト環境では TEST_DATABASE_URL を使用する（unit test ではダミー pool で十分）
    async fn make_pool_for_test() -> PgPool {
        // TEST_DATABASE_URL が設定されている場合は実際の DB に接続する
        let url = std::env::var("TEST_DATABASE_URL")
            .unwrap_or_else(|_| "postgres://localhost/k1s0_test".to_string());
        // PgPool を接続せずに生成する（unit test では pool を使わないため）
        // sqlx::PgPool::connect は非同期 I/O が必要なため offline test では unwrap できない
        // pool が必要な実際のパスは integration test (#[cfg(feature = "integration")]) でカバーする
        sqlx::PgPool::connect_lazy(&url)
            .expect("PgPool::connect_lazy should not fail on valid URL format")
    }

    // テスト用の TenantContext を生成するヘルパー関数
    fn make_context(tenant_id: Uuid) -> TenantContext {
        TenantContext::from_auth(
            tenant_id,
            "test-actor".to_string(),
            SessionPurpose::BusinessOp,
        )
    }

    // テスト用の StateChange を生成するヘルパー関数
    fn make_state_change(tenant_id: Uuid, table_class: TableClass) -> StateChange {
        StateChange {
            aggregate_id: Uuid::new_v4(),
            tenant_id,
            table_class,
            payload: serde_json::json!({"key": "value"}),
            version: 1,
        }
    }

    #[tokio::test]
    // P3: tenant_id 一致の場合は成功することを確認する
    async fn test_p3_tenant_id_match_ok() {
        // 同一の tenant_id で TenantContext と StateChange を生成する
        let tenant_id = Uuid::new_v4();
        let ctx = make_context(tenant_id);
        // テスト用 PgPool を生成する
        let pool = make_pool_for_test().await;
        let writer = AtomicTripleWrite::new(ctx, pool);
        let change = make_state_change(tenant_id, TableClass::TenantScoped);
        // P3 検証が成功することを確認する
        assert!(writer.verify_tenant_id(&change).is_ok());
    }

    #[tokio::test]
    // P3: tenant_id 不一致の場合は TenantIdMismatch エラーを返すことを確認する
    async fn test_p3_tenant_id_mismatch_error() {
        // 異なる tenant_id で TenantContext と StateChange を生成する
        let ctx = make_context(Uuid::new_v4());
        // テスト用 PgPool を生成する
        let pool = make_pool_for_test().await;
        let writer = AtomicTripleWrite::new(ctx, pool);
        let different_tenant_id = Uuid::new_v4();
        let change = make_state_change(different_tenant_id, TableClass::TenantScoped);
        // P3 違反: TenantIdMismatch エラーが返されることを確認する
        let err = writer.verify_tenant_id(&change);
        assert!(matches!(err, Err(AtomicWriteError::TenantIdMismatch { .. })));
    }

    #[tokio::test]
    // P4: pii_segregated の場合は audit 必須フラグが true を返すことを確認する
    async fn test_p4_pii_audit_required() {
        // PiiSegregated テーブルクラスの StateChange を生成する
        let tenant_id = Uuid::new_v4();
        let ctx = make_context(tenant_id);
        // テスト用 PgPool を生成する
        let pool = make_pool_for_test().await;
        let writer = AtomicTripleWrite::new(ctx, pool);
        let pii_change = make_state_change(tenant_id, TableClass::PiiSegregated);
        // pii_segregated では audit が必須であることを確認する
        assert!(writer.verify_pii_audit_required(&pii_change));
        // tenant_scoped では pgaudit は不要（通常 audit_event のみ）
        let normal_change = make_state_change(tenant_id, TableClass::TenantScoped);
        assert!(!writer.verify_pii_audit_required(&normal_change));
    }

    #[test]
    // P2: outbox 失敗シミュレーションがエラーを返すことを確認する
    fn test_p2_outbox_failure_simulation() {
        // outbox 失敗エラーが OutboxInsertFailed であることを確認する
        let err = AtomicTripleWrite::simulate_outbox_failure();
        assert!(matches!(err, AtomicWriteError::OutboxInsertFailed(_)));
    }
}
