// IObjectStorage.cs — k1s0 tier1 Library C# 実装: Object Storage の L3 interface
// 09_ストレージ適合仕様.md §IObjectStorageClient（OSS 中立 L3）に準拠する。
// S3 / GCS / Azure Blob 等 OSS の API を一切露出しない Wire protocol 抽象 interface を宣言する。
// 公開シグネチャに OSS 型（AWSSDK.S3 等）を一切含まない。

// System: 基本型に使用する
using System;
// System.Collections.Generic: IReadOnlyDictionary に使用する
using System.Collections.Generic;
// System.IO: Stream に使用する
using System.IO;
// System.Runtime.CompilerServices: IAsyncEnumerable に使用する
using System.Runtime.CompilerServices;
// System.Threading: CancellationToken に使用する
using System.Threading;
// System.Threading.Tasks: Task / ValueTask に使用する
using System.Threading.Tasks;

// k1s0 tier1 名前空間
namespace K1s0.Tier1;

/// <summary>
/// StorageObjectMeta はオブジェクトのメタデータを宣言する型。
/// OSS の HeadObject レスポンス型を露出せず Library 独自語彙で表現する。
/// </summary>
// StorageObjectMeta レコード定義
public sealed record StorageObjectMeta(
    // Key: オブジェクトキー（バケット内でユニーク）
    string Key,
    // SizeBytes: オブジェクトのバイトサイズ
    long SizeBytes,
    // ContentType: MIME type（"application/octet-stream" 等）
    string ContentType,
    // ETag: オブジェクトの整合性チェックサム
    string ETag,
    // CustomMeta: ユーザー定義メタデータ（tenant_id 等を格納する）
    IReadOnlyDictionary<string, string> CustomMeta,
    // VersionId: バージョニング対応バケットのオブジェクトバージョン識別子
    string? VersionId
);

/// <summary>
/// StoragePutOptions は PutObjectAsync に渡すオプションを宣言する型。
/// </summary>
// StoragePutOptions クラス定義
public sealed class StoragePutOptions
{
    /// <summary>ContentType: アップロードするオブジェクトの MIME type</summary>
    // ContentType プロパティ
    public string? ContentType { get; init; }

    /// <summary>CustomMeta: ユーザー定義メタデータ（tenant_id 等）</summary>
    // CustomMeta プロパティ
    public IReadOnlyDictionary<string, string>? CustomMeta { get; init; }

    /// <summary>ServerSideEncryption: サーバー側暗号化の設定（"AES256" / "aws:kms" 等）</summary>
    // ServerSideEncryption プロパティ
    public string? ServerSideEncryption { get; init; }

    /// <summary>KmsKeyId: KMS を使用するサーバー側暗号化の KMS キー ID</summary>
    // KmsKeyId プロパティ
    public string? KmsKeyId { get; init; }
}

/// <summary>
/// StorageGetOptions は GetObjectAsync に渡すオプションを宣言する型。
/// </summary>
// StorageGetOptions クラス定義
public sealed class StorageGetOptions
{
    /// <summary>VersionId: 特定バージョンを取得する場合に設定する（null = 最新バージョン）</summary>
    // VersionId プロパティ
    public string? VersionId { get; init; }

    /// <summary>RangeStart: Range 取得の開始バイトオフセット（0 = 先頭から）</summary>
    // RangeStart プロパティ
    public long RangeStart { get; init; }

    /// <summary>RangeEnd: Range 取得の終了バイトオフセット（0 = 末尾まで）</summary>
    // RangeEnd プロパティ
    public long RangeEnd { get; init; }
}

/// <summary>
/// StorageListOptions は ListObjectsAsync に渡すオプションを宣言する型。
/// </summary>
// StorageListOptions クラス定義
public sealed class StorageListOptions
{
    /// <summary>Prefix: 列挙するキーのプレフィックスフィルター（null = 全キー）</summary>
    // Prefix プロパティ
    public string? Prefix { get; init; }

    /// <summary>Delimiter: 仮想ディレクトリ区切り文字（"/" 等）</summary>
    // Delimiter プロパティ
    public string? Delimiter { get; init; }

    /// <summary>MaxKeys: 一度に取得するキーの最大数（0 = 実装固有のデフォルト上限を使用する）</summary>
    // MaxKeys プロパティ
    public int MaxKeys { get; init; }

    /// <summary>ContinuationToken: ページネーション継続トークン（null = 最初のページ）</summary>
    // ContinuationToken プロパティ
    public string? ContinuationToken { get; init; }
}

/// <summary>
/// StorageListResult は ListObjectsAsync の結果を宣言する型。
/// </summary>
// StorageListResult レコード定義
public sealed record StorageListResult(
    // Objects: 取得したオブジェクトのメタデータ一覧
    IReadOnlyList<StorageObjectMeta> Objects,
    // CommonPrefixes: 仮想ディレクトリのプレフィックス一覧
    IReadOnlyList<string> CommonPrefixes,
    // NextContinuationToken: 次ページのトークン（null = 最終ページ）
    string? NextContinuationToken,
    // IsTruncated: 結果が切り詰められているかどうか
    bool IsTruncated
);

/// <summary>
/// PresignedUrlOptions は PresignGetUrlAsync / PresignPutUrlAsync に渡すオプションを宣言する型。
/// wall-clock TTL 禁止規約に準拠して HLC ベースの TTL のみを受け付ける。
/// </summary>
// PresignedUrlOptions クラス定義
public sealed class PresignedUrlOptions
{
    /// <summary>Ttl: 署名付き URL の HLC ベース有効期限（必須: 無期限 URL は禁止する）</summary>
    // Ttl プロパティ（必須）
    public required CacheTtl Ttl { get; init; }

    /// <summary>ContentType: PUT 用署名付き URL の Content-Type 制約</summary>
    // ContentType プロパティ
    public string? ContentType { get; init; }
}

/// <summary>
/// IObjectStorageClient は Object Storage の L3 抽象 interface を宣言する。
/// S3 / GCS / Azure Blob / MinIO 等を透過的に切り替え可能にする。
/// OSS 型を引数・戻り値に一切含まない。
/// </summary>
// IObjectStorageClient インターフェース定義
public interface IObjectStorageClient
{
    /// <summary>
    /// PutObjectAsync はオブジェクトをバケットにアップロードする。
    /// stream はオブジェクトデータを提供する Stream（ストリーミングアップロードをサポートする）。
    /// </summary>
    // PutObjectAsync メソッド: オブジェクトをアップロードする
    Task<StorageObjectMeta> PutObjectAsync(
        string bucket,
        string key,
        Stream stream,
        StoragePutOptions? opts = null,
        CancellationToken cancellationToken = default);

    /// <summary>
    /// GetObjectAsync はバケットからオブジェクトをダウンロードする。
    /// 戻り値の Stream は使用後に必ず Dispose する（リソースリーク防止）。
    /// </summary>
    // GetObjectAsync メソッド: オブジェクトをダウンロードする
    Task<(Stream Content, StorageObjectMeta Meta)> GetObjectAsync(
        string bucket,
        string key,
        StorageGetOptions? opts = null,
        CancellationToken cancellationToken = default);

    /// <summary>
    /// HeadObjectAsync はオブジェクトのメタデータのみを取得する（ボディは取得しない）。
    /// オブジェクトが存在しない場合は null を返す（エラーと区別する）。
    /// </summary>
    // HeadObjectAsync メソッド: オブジェクトのメタデータを取得する
    Task<StorageObjectMeta?> HeadObjectAsync(string bucket, string key, CancellationToken cancellationToken = default);

    /// <summary>
    /// DeleteObjectAsync はバケットからオブジェクトを削除する（idempotent 操作）。
    /// </summary>
    // DeleteObjectAsync メソッド: オブジェクトを削除する
    Task DeleteObjectAsync(string bucket, string key, CancellationToken cancellationToken = default);

    /// <summary>
    /// ListObjectsAsync はバケット内のオブジェクトを列挙する。
    /// </summary>
    // ListObjectsAsync メソッド: オブジェクトを列挙する
    Task<StorageListResult> ListObjectsAsync(string bucket, StorageListOptions? opts = null, CancellationToken cancellationToken = default);

    /// <summary>
    /// CopyObjectAsync は同一バケット内または異なるバケット間でオブジェクトをコピーする。
    /// </summary>
    // CopyObjectAsync メソッド: オブジェクトをコピーする
    Task<StorageObjectMeta> CopyObjectAsync(
        string srcBucket,
        string srcKey,
        string dstBucket,
        string dstKey,
        CancellationToken cancellationToken = default);

    /// <summary>
    /// PresignGetUrlAsync は GET 用署名付き URL を生成する。
    /// wall-clock TTL 禁止規約に準拠して opts.Ttl は HLC ベースで指定する。
    /// </summary>
    // PresignGetUrlAsync メソッド: GET 用署名付き URL を生成する
    Task<string> PresignGetUrlAsync(string bucket, string key, PresignedUrlOptions opts, CancellationToken cancellationToken = default);

    /// <summary>
    /// PresignPutUrlAsync は PUT 用署名付き URL を生成する。
    /// wall-clock TTL 禁止規約に準拠して opts.Ttl は HLC ベースで指定する。
    /// </summary>
    // PresignPutUrlAsync メソッド: PUT 用署名付き URL を生成する
    Task<string> PresignPutUrlAsync(string bucket, string key, PresignedUrlOptions opts, CancellationToken cancellationToken = default);
}
