// k1s0 tier2 attachment C# (.NET 8+) インターフェース定義
// Rust 実装（attachment/src/attachment_store.rs）と 4 言語等価強度を持つ C# 版
// テナント分離されたオブジェクトストレージへの添付ファイル操作を抽象化する（設計方針 28）

// System.Guid などの基本型
using System;
// System.Threading: キャンセルトークン型
using System.Threading;
// System.Threading.Tasks: 非同期処理に使用する
using System.Threading.Tasks;

// k1s0 tier2 attachment 名前空間
namespace K1s0.Tier2.Attachment;

/// <summary>
/// AttachmentMetadata: アップロード完了後に返すメタデータ構造体
/// Rust の AttachmentMetadata 構造体に対応する
/// </summary>
public sealed record AttachmentMetadata
{
    // 添付ファイルの一意識別子（UUID v4）
    public required Guid AttachmentId { get; init; }
    // アップロード先テナント識別子（RLS で自動フィルタリングされる）
    public required Guid TenantId { get; init; }
    // ファイルの MIME タイプ（例: application/pdf / image/png）
    public required string ContentType { get; init; }
    // アップロードされたファイルのバイトサイズ
    public required ulong SizeBytes { get; init; }
    // ストレージ内のオブジェクトキー（テナント分離プレフィックス付き）
    public required string ObjectKey { get; init; }
    // HLC タイムスタンプ（wall clock TTL 禁止規約により HLC を使用する）
    public required ulong HlcTimestamp { get; init; }
    // エンベロープ暗号化で使用された DEK ハンドル（OpenBao Transit で管理する）
    public required string DekHandle { get; init; }
}

/// <summary>
/// IAttachmentStore: テナント分離されたオブジェクトストレージへの操作インターフェース
/// Rust の AttachmentStore トレイトに対応する（設計方針 28）
/// 実装クラスは MinIO または S3 互換ストレージと連携する
/// tenant_id は AuthContext から取得するため API 引数で受け取らない（直接渡し禁止）
/// </summary>
public interface IAttachmentStore
{
    /// <summary>
    /// テナント分離されたオブジェクトストレージに添付ファイルをアップロードする
    /// tenantId: アップロード先テナント識別子（AuthContext から注入する / API 引数で受け取り禁止）
    /// data: アップロードするファイルのバイト配列
    /// contentType: ファイルの MIME タイプ
    /// アップロード完了後に AttachmentMetadata を返す
    /// </summary>
    // UploadAsync メソッド（テナント分離ファイルアップロード操作）
    Task<AttachmentMetadata> UploadAsync(
        Guid tenantId,
        ReadOnlyMemory<byte> data,
        string contentType,
        CancellationToken cancellationToken = default);

    /// <summary>
    /// 添付ファイルを取得する
    /// tenantId: 取得対象テナント識別子（RLS で自動フィルタリングされる）
    /// attachmentId: 取得対象添付ファイル識別子
    /// ファイルのバイト配列を返す
    /// </summary>
    // FetchAsync メソッド（添付ファイル取得操作）
    Task<byte[]> FetchAsync(
        Guid tenantId,
        Guid attachmentId,
        CancellationToken cancellationToken = default);

    /// <summary>
    /// 添付ファイルを削除する（PII データ削除フローから呼び出される）
    /// tenantId: 削除対象テナント識別子
    /// attachmentId: 削除対象添付ファイル識別子
    /// </summary>
    // DeleteAsync メソッド（添付ファイル削除操作）
    Task DeleteAsync(
        Guid tenantId,
        Guid attachmentId,
        CancellationToken cancellationToken = default);
}
