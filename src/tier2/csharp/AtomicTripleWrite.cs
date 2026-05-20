// k1s0 tier2 atomic 三表書込 C# (.NET 8+) 実装
// Rust 実装（atomic_triple_write.rs）と semantic 等価な C# 版
// State change / Outbox / Audit event を同一 DB トランザクションで書く
// TLA+ の P1-P4 invariant と double-bind する（src/formal/dafny/AtomicThreeTableWrite.dfy）
//
// P1: aggregate 状態変更時、必ず Outbox + Audit が同一 txn に書込
// P2: Outbox 投入失敗時、aggregate 状態変更も rollback
// P3: tenant_id が GUC と aggregate 行で一致しない場合、操作を reject
// P4: pii_segregated の全アクセスを audit_event に記録

// Guid / Exception / InvalidOperationException 等の基本型
using System;
// 非同期処理に使用する
using System.Threading;
using System.Threading.Tasks;
// Npgsql: PostgreSQL クライアント（NpgsqlTransaction による実 transaction に使用する）
using Npgsql;
// HLC wrapper: wall-clock TTL 禁止規約 (src/CLAUDE.md §wall-clock TTL 禁止) に従い HLC を使用する
using K1s0.HlcLib;

// k1s0 tier2 名前空間
namespace K1s0.Tier2;

/// <summary>
/// 書込対象テーブルクラス（10_テナント分離適合仕様.md の 4 class と一致する）
/// </summary>
public enum TableClass
{
    /// <summary>tenant_scoped: tenant_id 必須、RLS FORCE</summary>
    TenantScoped,
    /// <summary>tenant_master: tenant_id 必須、role 制限付き RLS FORCE</summary>
    TenantMaster,
    /// <summary>platform_global: tenant_id 無し、RLS 無効</summary>
    PlatformGlobal,
    /// <summary>pii_segregated: tenant_id 必須 + purpose check + pgaudit 全アクセス</summary>
    PiiSegregated,
}

/// <summary>
/// aggregate の状態変更を表すデータクラス（P1 の state_change に対応する）
/// </summary>
public sealed record StateChange(
    // 変更対象の aggregate ID
    Guid AggregateId,
    // 変更対象の tenant_id（TenantContext.TenantId と一致している必要がある）
    Guid TenantId,
    // テーブルクラス（どの class の table を書込むかを示す）
    TableClass TableClass,
    // 変更内容のシリアライズ済みペイロード（JSON 文字列）
    string Payload,
    // aggregate バージョン（楽観的ロックに使用する）
    long Version
);

/// <summary>
/// atomic 三表書込の結果
/// </summary>
public sealed record TripleWriteResult(
    // 書込んだ aggregate ID
    Guid AggregateId,
    // 書込んだ outbox エントリの ID
    Guid OutboxId,
    // 書込んだ audit_event の ID
    Guid AuditEventId,
    // 書込完了日時（UTC）
    DateTimeOffset CommittedAt
);

/// <summary>
/// atomic 三表書込のインターフェース
/// 4 言語等価強度を保証するために interface を定義する
/// </summary>
public interface IAtomicTripleWrite
{
    /// <summary>P3: tenant_id 一致を検証する</summary>
    void VerifyTenantId(StateChange change);

    /// <summary>P4: pii_segregated アクセスが audit_event 必須かを返す</summary>
    bool VerifyPiiAuditRequired(StateChange change);

    /// <summary>P1-P4: atomic 三表書込を実行する非同期メソッド（NpgsqlTransaction を使用する）</summary>
    Task<TripleWriteResult> ExecuteAsync(
        StateChange change,
        NpgsqlTransaction transaction,
        CancellationToken cancellationToken = default);
}

/// <summary>
/// atomic 三表書込の実行エンジン
/// Npgsql 8 の NpgsqlTransaction を受け取る ExecuteAsync を持つ
/// TenantContext を使って GUC 注入と tenant_id 検証を行う
/// </summary>
public sealed class AtomicTripleWrite : IAtomicTripleWrite
{
    // テナントコンテキスト（GUC 注入・tenant_id 検証に使用する）
    private readonly TenantContext _context;

    /// <summary>
    /// AtomicTripleWrite を生成する（TenantContext を受け取る）
    /// </summary>
    public AtomicTripleWrite(TenantContext context)
    {
        // TenantContext を格納する（null は許容しない）
        _context = context ?? throw new ArgumentNullException(nameof(context));
    }

    /// <summary>
    /// P3: tenant_id 一致を検証する
    /// StateChange の TenantId が TenantContext の TenantId と一致しない場合は例外をスローする
    /// </summary>
    public void VerifyTenantId(StateChange change)
    {
        // null チェック
        ArgumentNullException.ThrowIfNull(change);
        // GUC の tenant_id と aggregate の tenant_id を比較する
        var gucTenantId = _context.TenantId;
        if (gucTenantId != change.TenantId)
        {
            // P3 違反: tenant_id 不一致で例外をスローする
            throw new InvalidOperationException(
                $"P3 TenantId mismatch: guc={gucTenantId:D}, row={change.TenantId:D}");
        }
    }

    /// <summary>
    /// P4: pii_segregated アクセスが audit_event 必須かを返す
    /// </summary>
    public bool VerifyPiiAuditRequired(StateChange change)
    {
        // null チェック
        ArgumentNullException.ThrowIfNull(change);
        // PiiSegregated の場合は必ず audit_event を記録する（true を返す）
        return change.TableClass == TableClass.PiiSegregated;
    }

    /// <summary>
    /// P1-P4: atomic 三表書込を実行する非同期メソッド（NpgsqlTransaction を使用する）
    /// transaction: 呼び出し元が BEGIN した NpgsqlTransaction を受け取る
    /// 呼び出し元は Ok 返却後に transaction.CommitAsync() を呼ぶ。例外時は RollbackAsync() を呼ぶ。
    /// P1: state_change / outbox / audit_event の 3 INSERT を同一 txn で実行する
    /// P2: outbox INSERT が失敗した場合は例外をスローし、呼び出し元が RollbackAsync する
    /// P3: tenant_id が GUC と一致しない場合は即座に reject して InvalidOperationException をスローする
    /// P4: pii_segregated テーブルへのアクセスは audit_event に記録してから txn を実行する
    /// </summary>
    public async Task<TripleWriteResult> ExecuteAsync(
        // 書込対象の状態変更（P1-P4 の対象）
        StateChange change,
        // 呼び出し元が管理する NpgsqlTransaction（同一 txn で 3 INSERT を実行する）
        NpgsqlTransaction transaction,
        // キャンセレーショントークン
        CancellationToken cancellationToken = default)
    {
        // null チェック（change と transaction は必須）
        ArgumentNullException.ThrowIfNull(change);
        ArgumentNullException.ThrowIfNull(transaction);
        // cancellationToken の cancel を確認する
        cancellationToken.ThrowIfCancellationRequested();
        // P3: tenant_id 一致を事前検証する（GUC と aggregate 行の tenant_id が一致しない場合は即座に例外）
        VerifyTenantId(change);
        // P4: pii_segregated の場合は audit_event への記録が必須であることを確認する
        _ = VerifyPiiAuditRequired(change);

        // P3: SET LOCAL で 4 GUC を txn スコープに注入する（RLS FORCE が参照する）
        var setGucSql = _context.ToSetLocalSql();
        // NpgsqlCommand を使って SET LOCAL GUC を transaction 内で実行する
        await using var setGucCmd = new NpgsqlCommand(setGucSql, transaction.Connection, transaction);
        // SET LOCAL GUC を非同期実行する（transaction スコープのみ有効、COMMIT で自動破棄される）
        await setGucCmd.ExecuteNonQueryAsync(cancellationToken).ConfigureAwait(false);

        // outbox エントリの ID を生成する（P1 の atomic 三表書込で使用する）
        var outboxId = Guid.NewGuid();
        // audit_event の ID を生成する（domain_event と audit_event で共有する）
        var auditEventId = Guid.NewGuid();
        // HLC タイムスタンプを取得する（wall-clock TTL 禁止規約 src/CLAUDE.md §wall-clock TTL 禁止 に従う）
        var hlcNow = HlcClock.FromEnv().Now();
        // HLC の WallMs（UNIX ミリ秒）を DateTimeOffset に変換する（3 INSERT で統一した timestamp を使用する）
        var committedAt = DateTimeOffset.FromUnixTimeMilliseconds((long)hlcNow.WallMs);

        // P1: k1s0.domain_event テーブルに INSERT する（aggregate 状態変更の永続化）
        // current_setting('app.tenant_id')::uuid を使って RLS FORCE の tenant_id を注入する
        const string domainEventSql = @"
            INSERT INTO k1s0.domain_event
                (id, aggregate_id, tenant_id, event_kind, payload, version, created_at)
            VALUES
                (@id, @agg_id, current_setting('app.tenant_id')::uuid, 'StateChange', @payload::jsonb, @version, @created_at)
        ";
        // domain_event INSERT コマンドを生成する
        await using var domainEventCmd = new NpgsqlCommand(domainEventSql, transaction.Connection, transaction);
        // audit_event_id を domain_event の主キーとしてバインドする
        domainEventCmd.Parameters.AddWithValue("@id", auditEventId);
        // 変更対象の aggregate ID をバインドする
        domainEventCmd.Parameters.AddWithValue("@agg_id", change.AggregateId);
        // ペイロードを jsonb 文字列としてバインドする
        domainEventCmd.Parameters.AddWithValue("@payload", change.Payload);
        // aggregate バージョンをバインドする（楽観的ロックに使用する）
        domainEventCmd.Parameters.AddWithValue("@version", change.Version);
        // 書込完了日時をバインドする
        domainEventCmd.Parameters.AddWithValue("@created_at", committedAt);
        // P1 の domain_event INSERT を非同期実行する（同一 txn で実行する）
        await domainEventCmd.ExecuteNonQueryAsync(cancellationToken).ConfigureAwait(false);

        // P1: k1s0.outbox_message テーブルに INSERT する（Debezium CDC 経由で Kafka に転送される）
        // P2: この INSERT が失敗した場合は例外をスローし、呼び出し元が RollbackAsync を呼ぶ
        // migration SoT: 0001_initial_schema.sql が CREATE TABLE k1s0.outbox_message を発行している
        const string outboxSql = @"
            INSERT INTO k1s0.outbox_message
                (id, aggregate_id, tenant_id, event_kind, payload, created_at)
            VALUES
                (@id, @agg_id, current_setting('app.tenant_id')::uuid, 'OutboxRelay', @payload::jsonb, @created_at)
        ";
        // outbox INSERT コマンドを生成する
        await using var outboxCmd = new NpgsqlCommand(outboxSql, transaction.Connection, transaction);
        // outbox エントリの ID をバインドする
        outboxCmd.Parameters.AddWithValue("@id", outboxId);
        // 変更対象の aggregate ID をバインドする
        outboxCmd.Parameters.AddWithValue("@agg_id", change.AggregateId);
        // ペイロードを jsonb 文字列としてバインドする（PII は redact 済みのみ含む）
        outboxCmd.Parameters.AddWithValue("@payload", change.Payload);
        // 書込完了日時をバインドする
        outboxCmd.Parameters.AddWithValue("@created_at", committedAt);
        // P2 の outbox INSERT を非同期実行する（失敗時は呼び出し元が Rollback する）
        await outboxCmd.ExecuteNonQueryAsync(cancellationToken).ConfigureAwait(false);

        // P1+P4: k1s0.audit_event テーブルに INSERT する（全操作を監査記録する）
        // P4: pii_segregated は pgaudit も併用するが、アプリ層からも必ず audit_event を書く
        const string auditEventSql = @"
            INSERT INTO k1s0.audit_event
                (id, aggregate_id, tenant_id, actor_id, purpose, table_class, payload, created_at)
            VALUES
                (@id, @agg_id, current_setting('app.tenant_id')::uuid,
                 current_setting('app.actor_id'),
                 current_setting('app.purpose'),
                 @table_class, @payload::jsonb, @created_at)
        ";
        // audit_event INSERT コマンドを生成する
        await using var auditEventCmd = new NpgsqlCommand(auditEventSql, transaction.Connection, transaction);
        // audit_event の ID をバインドする（domain_event と同じ ID で結びつける）
        auditEventCmd.Parameters.AddWithValue("@id", auditEventId);
        // 変更対象の aggregate ID をバインドする
        auditEventCmd.Parameters.AddWithValue("@agg_id", change.AggregateId);
        // テーブルクラスを文字列としてバインドする
        auditEventCmd.Parameters.AddWithValue("@table_class", change.TableClass.ToString());
        // ペイロードを jsonb 文字列としてバインドする
        auditEventCmd.Parameters.AddWithValue("@payload", change.Payload);
        // 書込完了日時をバインドする
        auditEventCmd.Parameters.AddWithValue("@created_at", committedAt);
        // P4 の audit_event INSERT を非同期実行する（P4 の audit 必須要件を満たす）
        await auditEventCmd.ExecuteNonQueryAsync(cancellationToken).ConfigureAwait(false);

        // 三表書込の結果を返す（呼び出し元が transaction.CommitAsync() を呼ぶことで確定する）
        return new TripleWriteResult(
            // 書込んだ aggregate ID を返す
            AggregateId: change.AggregateId,
            // 書込んだ outbox エントリの ID を返す
            OutboxId: outboxId,
            // 書込んだ audit_event の ID を返す
            AuditEventId: auditEventId,
            // 書込完了日時を返す
            CommittedAt: committedAt
        );
    }
}
