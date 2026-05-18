// k1s0 tier3 C# 添付ファイルストア interface（TypeScript 等価強度実装）
// チャンクアップロード / MIME 検査 / ハッシュチェーン整合性を C# interface として定義する
// .NET 8+ Task<T> を使用した非同期 interface

// System 名前空間: Task<T> / IReadOnlySet に使用する
using System.Collections.Frozen;

// k1s0 tier3 State 名前空間に添付ファイル関連クラスを配置する
namespace K1s0.Tier3.State;

// --------- 定数 ---------

// 添付ファイル関連の定数クラス
public static class AttachmentConstants
{
    // デフォルトのチャンクサイズ（4 MiB）
    public const int DefaultChunkSizeBytes = 4 * 1024 * 1024;

    // 許可する MIME タイプ一覧（FrozenSet で不変集合として管理する）
    public static readonly FrozenSet<string> AllowedMimeTypes = new HashSet<string>
    {
        // PDF 文書
        "application/pdf",
        // Microsoft Excel（新形式）
        "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        // Microsoft Word（新形式）
        "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
        // CSV テキスト
        "text/csv",
        // プレーンテキスト
        "text/plain",
        // JPEG 画像
        "image/jpeg",
        // PNG 画像
        "image/png",
    }.ToFrozenSet(StringComparer.OrdinalIgnoreCase);
}

// --------- 型定義 ---------

// 添付ファイルのメタデータ record
public sealed record AttachmentMeta(
    // ファイル名
    string FileName,
    // MIME タイプ
    string MimeType,
    // ファイルサイズ（バイト）
    long SizeBytes,
    // ファイル全体の SHA-256 ハッシュ（hex 文字列）
    string Sha256Hex
);

// チャンクデータ record
public sealed record AttachmentChunk(
    // チャンク番号（0 始まり）
    int ChunkIndex,
    // チャンクのバイナリデータ
    ReadOnlyMemory<byte> Data,
    // このチャンクの SHA-256 ハッシュ（hex 文字列）
    string ChunkSha256Hex
);

// アップロード結果 record
public sealed record AttachmentUploadResult(
    // 付与された添付ファイル ID（サーバー生成 UUID）
    string AttachmentId,
    // アップロード完了時刻（ISO 8601）
    string CompletedAt,
    // サーバー側で計算されたファイル全体ハッシュ
    string ServerSha256Hex
);

// --------- MIME 検査ヘルパー ---------

// MIME タイプ検査のヘルパークラス
public static class MimeTypeChecker
{
    // MIME タイプが許可リストに含まれているか検査する
    public static bool IsAllowed(string mimeType) =>
        // FrozenSet の Contains で O(1) 検索を行う
        AttachmentConstants.AllowedMimeTypes.Contains(mimeType);
}

// --------- IAttachmentStore interface ---------

// IAttachmentStore: 添付ファイルの保存・取得・削除を抽象化する interface
// .NET 8 Task<T> を使用した非同期 interface
public interface IAttachmentStore
{
    // アップロードセッションを開始する（multipart upload init 相当）
    // meta: ファイルのメタデータ
    // 戻り値: アップロード ID（string）
    Task<string> InitUploadAsync(AttachmentMeta meta, CancellationToken ct = default);

    // チャンクをアップロードする
    // uploadId: InitUploadAsync で取得したアップロード ID
    // chunk: アップロードするチャンクデータ
    Task UploadChunkAsync(string uploadId, AttachmentChunk chunk, CancellationToken ct = default);

    // 全チャンクのアップロード完了を通知してアップロード結果を返す
    // uploadId: 完了するアップロード ID
    Task<AttachmentUploadResult> CompleteUploadAsync(string uploadId, CancellationToken ct = default);

    // 指定した添付ファイル ID のメタデータを取得する
    // attachmentId: 取得する添付ファイルの ID
    Task<AttachmentMeta> GetMetaAsync(string attachmentId, CancellationToken ct = default);

    // 指定した添付ファイル ID を論理削除する
    // attachmentId: 削除する添付ファイルの ID
    Task DeleteAsync(string attachmentId, CancellationToken ct = default);
}
