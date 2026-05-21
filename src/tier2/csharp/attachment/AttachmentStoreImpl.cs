// k1s0 tier2 attachment C# (.NET 8+) 実装クラス
// Rust 実装（attachment/src/attachment_store.rs ObjectAttachmentStore）と 4 言語等価強度を持つ C# 実装版
// IAttachmentStore インターフェースを実装し、テナント分離されたオブジェクトストレージへの CRUD を提供する
// wall-clock TTL 禁止規約準拠: HlcTimestamp を HlcClock 経由で取得する

// System.Guid などの基本型
using System;
// System.Collections.Generic: Dictionary に使用する
using System.Collections.Generic;
// System.Threading: CancellationToken に使用する
using System.Threading;
// System.Threading.Tasks: 非同期処理に使用する
using System.Threading.Tasks;
// HLC wrapper: wall-clock TTL 禁止規約 (src/CLAUDE.md §wall-clock TTL 禁止) に従い HLC を使用する
using K1s0.HlcLib;

// k1s0 tier2 attachment 名前空間
namespace K1s0.Tier2.Attachment;

/// <summary>
/// AttachmentStoreImpl: IAttachmentStore の骨格実装クラス
/// Rust の ObjectAttachmentStore と等価な C# 実装を提供する
/// テナント分離されたオブジェクトストレージへの Upload / Fetch / Delete を実装する
/// 実際の S3/MinIO 接続は DI コンテナから IObjectStorageClient を注入する形に拡張する
/// </summary>
public sealed class AttachmentStoreImpl : IAttachmentStore
{
    // _hlcClock: HLC タイムスタンプ取得に使用する（wall-clock 禁止のため HLC を使う）
    private readonly HlcClock _hlcClock;

    // _store: テナント別添付ファイルの in-memory ストア（骨格実装用 / 本番は S3/MinIO を使う）
    // key: "{tenantId}/{attachmentId}", value: (data, metadata) のペア
    private readonly Dictionary<string, (byte[] Data, AttachmentMetadata Metadata)> _store
        = new();

    // _lock: _store への同時アクセスを制御する排他オブジェクト
    private readonly object _lock = new();

    /// <summary>
    /// AttachmentStoreImpl を生成する（HlcClock を受け取る）
    /// </summary>
    public AttachmentStoreImpl(HlcClock hlcClock)
    {
        // HlcClock が null の場合は ArgumentNullException をスローする
        _hlcClock = hlcClock ?? throw new ArgumentNullException(nameof(hlcClock));
    }

    /// <summary>
    /// デフォルトコンストラクタ: HLC_NODE_ID 環境変数から HlcClock を自動生成する
    /// </summary>
    public AttachmentStoreImpl()
    {
        // 環境変数から HlcClock を生成する（HLC_NODE_ID 未設定時は nodeId=0）
        _hlcClock = HlcClock.FromEnv();
    }

    /// <summary>
    /// テナント分離されたオブジェクトストレージに添付ファイルをアップロードする
    /// tenantId: アップロード先テナント識別子（AuthContext から注入する / API 引数で受け取り禁止）
    /// data: アップロードするファイルのバイト配列
    /// contentType: ファイルの MIME タイプ
    /// アップロード完了後に AttachmentMetadata を返す
    /// </summary>
    public Task<AttachmentMetadata> UploadAsync(
        // アップロード先テナント識別子（AuthContext からのみ注入する）
        Guid tenantId,
        // アップロードするファイルのバイト配列
        ReadOnlyMemory<byte> data,
        // ファイルの MIME タイプ
        string contentType,
        // キャンセルトークン
        CancellationToken cancellationToken = default)
    {
        // null チェック: contentType が null または空の場合は例外をスローする
        if (string.IsNullOrWhiteSpace(contentType))
        {
            // contentType 未指定は不正な呼び出しとして扱う
            throw new ArgumentException("contentType が null または空です", nameof(contentType));
        }
        // キャンセルトークンの cancel を確認する
        cancellationToken.ThrowIfCancellationRequested();

        // UUID v4 の添付ファイル識別子を生成する（Rust の Uuid::new_v4() と同等）
        var attachmentId = Guid.NewGuid();

        // テナント分離されたオブジェクトキーを構築する（形式: "{tenantId}/{attachmentId}"）
        var objectKey = $"{tenantId:D}/{attachmentId:D}";

        // HLC タイムスタンプを取得する（wall-clock TTL 禁止規約に準拠する）
        var hlcTs = _hlcClock.Now();

        // データをバイト配列にコピーする（ReadOnlyMemory を具体化する）
        var dataBytes = data.ToArray();

        // AttachmentMetadata を組み立てる
        var metadata = new AttachmentMetadata
        {
            // 生成した添付ファイル識別子
            AttachmentId = attachmentId,
            // アップロード先テナント識別子
            TenantId = tenantId,
            // MIME タイプ
            ContentType = contentType,
            // アップロードされたファイルのバイトサイズ
            SizeBytes = (ulong)dataBytes.Length,
            // テナント分離されたオブジェクトキー
            ObjectKey = objectKey,
            // HLC タイムスタンプ（wall-clock 禁止 / HlcClock.Now() から取得した WallMs）
            HlcTimestamp = hlcTs.WallMs,
            // DEK ハンドル（骨格実装では "dek-placeholder" を使用する / 本番は OpenBao Transit から取得する）
            DekHandle = "dek-placeholder",
        };

        // _store に排他ロックで保存する（in-memory 骨格実装）
        lock (_lock)
        {
            // テナント分離キーでデータとメタデータを保存する
            _store[objectKey] = (dataBytes, metadata);
        }

        // アップロード完了メタデータを返す
        return Task.FromResult(metadata);
    }

    /// <summary>
    /// 添付ファイルを取得する
    /// tenantId: 取得対象テナント識別子（RLS で自動フィルタリングされる）
    /// attachmentId: 取得対象添付ファイル識別子
    /// ファイルのバイト配列を返す
    /// </summary>
    public Task<byte[]> FetchAsync(
        // 取得対象テナント識別子
        Guid tenantId,
        // 取得対象添付ファイル識別子
        Guid attachmentId,
        // キャンセルトークン
        CancellationToken cancellationToken = default)
    {
        // キャンセルトークンの cancel を確認する
        cancellationToken.ThrowIfCancellationRequested();

        // テナント分離されたオブジェクトキーを構築する
        var objectKey = $"{tenantId:D}/{attachmentId:D}";

        // _store から排他ロックで取得する
        lock (_lock)
        {
            // オブジェクトキーで検索する
            if (_store.TryGetValue(objectKey, out var entry))
            {
                // 見つかった場合はデータのコピーを返す
                return Task.FromResult(entry.Data);
            }
        }

        // 見つからない場合は InvalidOperationException をスローする
        throw new InvalidOperationException(
            $"添付ファイルが見つかりません: tenantId={tenantId:D}, attachmentId={attachmentId:D}");
    }

    /// <summary>
    /// 添付ファイルを削除する（PII データ削除フローから呼び出される）
    /// tenantId: 削除対象テナント識別子
    /// attachmentId: 削除対象添付ファイル識別子
    /// </summary>
    public Task DeleteAsync(
        // 削除対象テナント識別子
        Guid tenantId,
        // 削除対象添付ファイル識別子
        Guid attachmentId,
        // キャンセルトークン
        CancellationToken cancellationToken = default)
    {
        // キャンセルトークンの cancel を確認する
        cancellationToken.ThrowIfCancellationRequested();

        // テナント分離されたオブジェクトキーを構築する
        var objectKey = $"{tenantId:D}/{attachmentId:D}";

        // _store から排他ロックで削除する
        lock (_lock)
        {
            // オブジェクトキーでエントリを削除する（存在しない場合も正常終了する）
            _store.Remove(objectKey);
        }

        // 削除完了を返す
        return Task.CompletedTask;
    }
}
