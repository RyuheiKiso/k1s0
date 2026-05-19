// ObjectStorageImpl.cs — k1s0 tier1 Library C# 実装: IObjectStorageClient の AWSSDK.S3 facade 実装
// 09_ストレージ適合仕様.md §IObjectStorageClient（OSS 中立 L3）に準拠する。
// AWSSDK.S3 の AmazonS3Client を L3 ラップして公開 API に AWSSDK.S3 型を露出しない。
// wall-clock TTL 禁止規約に準拠して PresignedUrlOptions.Ttl のみを有効期限に使用する。

// System: 基本型に使用する
using System;
// System.Collections.Generic: IReadOnlyDictionary / IReadOnlyList に使用する
using System.Collections.Generic;
// System.IO: Stream に使用する
using System.IO;
// System.Linq: LINQ 拡張メソッドに使用する
using System.Linq;
// System.Threading: CancellationToken に使用する
using System.Threading;
// System.Threading.Tasks: Task に使用する
using System.Threading.Tasks;
// Amazon.S3: S3 クライアント（内部のみ使用する）
using Amazon.S3;
// Amazon.S3.Model: S3 リクエスト / レスポンス型（内部のみ使用する）
using Amazon.S3.Model;

// k1s0 tier1 名前空間
namespace K1s0.Tier1;

/// <summary>
/// ObjectStorageImpl は IObjectStorageClient の AWSSDK.S3 facade 実装クラス。
/// AmazonS3Client を L3 ラップして公開 API に IAmazonS3 / AmazonS3Client 型を露出しない。
/// S3 / MinIO / Ceph S3 等の S3 互換ストレージを透過的に切り替えられる設計とする。
/// wall-clock TTL 禁止: PresignedUrlOptions.Ttl の HLC tick から有効期限を計算する。
/// </summary>
// ObjectStorageImpl クラス定義（internal sealed: 外部からの継承・直接参照を禁止する）
internal sealed class ObjectStorageImpl : IObjectStorageClient
{
    // _s3: AmazonS3Client（内部に隠蔽する）
    private readonly IAmazonS3 _s3;

    /// <summary>
    /// コンストラクタ: IAmazonS3 を注入する。
    /// IAmazonS3 型で受け取り、内部でのみ参照する（公開 API に露出しない）。
    /// </summary>
    // コンストラクタ: IAmazonS3 を依存注入する
    public ObjectStorageImpl(IAmazonS3 s3)
    {
        // null チェック: s3 が null の場合は例外を投げる
        _s3 = s3 ?? throw new ArgumentNullException(nameof(s3));
    }

    // ToStorageObjectMeta は S3 の HeadObjectResponse を StorageObjectMeta に変換する
    private static StorageObjectMeta ToStorageObjectMeta(string key, GetObjectMetadataResponse resp)
    {
        // カスタムメタデータを IReadOnlyDictionary に変換する
        var customMeta = resp.Metadata.Keys
            .ToDictionary(k => k, k => resp.Metadata[k]);
        // StorageObjectMeta を構築して返す
        return new StorageObjectMeta(
            // キーを設定する
            Key: key,
            // サイズを設定する
            SizeBytes: resp.ContentLength,
            // Content-Type を設定する
            ContentType: resp.Headers.ContentType ?? "application/octet-stream",
            // ETag を設定する（引用符を除去する）
            ETag: (resp.ETag ?? string.Empty).Trim('"'),
            // カスタムメタデータを設定する
            CustomMeta: customMeta,
            // バージョン ID を設定する
            VersionId: resp.VersionId
        );
    }

    // ToStorageObjectMetaFromPutResponse は PutObjectResponse から StorageObjectMeta を構築する
    private static StorageObjectMeta ToStorageObjectMetaFromPutResponse(string key, string bucket, PutObjectResponse resp)
    {
        // StorageObjectMeta を構築して返す（PutObjectResponse には ContentType / Size がないため簡易実装）
        return new StorageObjectMeta(
            // キーを設定する
            Key: key,
            // サイズは不明（0 を設定する）
            SizeBytes: 0,
            // Content-Type は不明（application/octet-stream を設定する）
            ContentType: "application/octet-stream",
            // ETag を設定する（引用符を除去する）
            ETag: (resp.ETag ?? string.Empty).Trim('"'),
            // カスタムメタデータは空とする
            CustomMeta: new Dictionary<string, string>(),
            // バージョン ID を設定する
            VersionId: resp.VersionId
        );
    }

    /// <summary>
    /// PutObjectAsync はオブジェクトをバケットにアップロードする。
    /// stream はオブジェクトデータを提供する Stream（ストリーミングアップロードをサポートする）。
    /// </summary>
    // PutObjectAsync メソッド実装: AmazonS3Client.PutObjectAsync を呼び出す
    public async Task<StorageObjectMeta> PutObjectAsync(
        string bucket,
        string key,
        Stream stream,
        StoragePutOptions? opts = null,
        CancellationToken cancellationToken = default)
    {
        // PutObjectRequest を構築する
        var request = new PutObjectRequest
        {
            // バケット名を設定する
            BucketName = bucket,
            // キーを設定する
            Key = key,
            // ストリームを設定する
            InputStream = stream,
            // Content-Type を設定する（null の場合は application/octet-stream）
            ContentType = opts?.ContentType ?? "application/octet-stream",
            // サーバー側暗号化を設定する
            ServerSideEncryptionMethod = opts?.ServerSideEncryption == "AES256"
                ? ServerSideEncryptionMethod.AES256
                : (opts?.ServerSideEncryption == "aws:kms" ? ServerSideEncryptionMethod.AWSKMS : ServerSideEncryptionMethod.None),
        };
        // KMS キー ID を設定する（aws:kms の場合のみ）
        if (opts?.KmsKeyId is not null)
        {
            // KMS キー ID を設定する
            request.ServerSideEncryptionKeyManagementServiceKeyId = opts.KmsKeyId;
        }
        // カスタムメタデータを設定する
        if (opts?.CustomMeta is not null)
        {
            // 各メタデータを PutObjectRequest.Metadata に追加する
            foreach (var kv in opts.CustomMeta)
            {
                // メタデータを追加する（S3 のメタデータキーは "x-amz-meta-" プレフィックスを除去する）
                request.Metadata.Add(kv.Key, kv.Value);
            }
        }
        // S3 の PutObjectAsync を呼び出す
        var response = await _s3.PutObjectAsync(request, cancellationToken).ConfigureAwait(false);
        // StorageObjectMeta に変換して返す
        return ToStorageObjectMetaFromPutResponse(key, bucket, response);
    }

    /// <summary>
    /// GetObjectAsync はバケットからオブジェクトをダウンロードする。
    /// 戻り値の Stream は使用後に必ず Dispose する（リソースリーク防止）。
    /// </summary>
    // GetObjectAsync メソッド実装: AmazonS3Client.GetObjectAsync を呼び出す
    public async Task<(Stream Content, StorageObjectMeta Meta)> GetObjectAsync(
        string bucket,
        string key,
        StorageGetOptions? opts = null,
        CancellationToken cancellationToken = default)
    {
        // GetObjectRequest を構築する
        var request = new GetObjectRequest
        {
            // バケット名を設定する
            BucketName = bucket,
            // キーを設定する
            Key = key,
            // バージョン ID を設定する（null の場合は最新バージョン）
            VersionId = opts?.VersionId,
        };
        // Range 取得が指定されている場合は設定する
        if (opts?.RangeStart > 0 || opts?.RangeEnd > 0)
        {
            // ByteRange を設定する
            request.ByteRange = new ByteRange(opts?.RangeStart ?? 0, opts?.RangeEnd ?? 0);
        }
        // S3 の GetObjectAsync を呼び出す
        var response = await _s3.GetObjectAsync(request, cancellationToken).ConfigureAwait(false);
        // StorageObjectMeta を構築する（GetObjectResponse からメタデータを取得する）
        var customMeta = response.Metadata.Keys
            .ToDictionary(k => k, k => response.Metadata[k]);
        // StorageObjectMeta を構築する
        var meta = new StorageObjectMeta(
            // キーを設定する
            Key: key,
            // Content-Length を設定する
            SizeBytes: response.ContentLength,
            // Content-Type を設定する
            ContentType: response.Headers.ContentType ?? "application/octet-stream",
            // ETag を設定する（引用符を除去する）
            ETag: (response.ETag ?? string.Empty).Trim('"'),
            // カスタムメタデータを設定する
            CustomMeta: customMeta,
            // バージョン ID を設定する
            VersionId: response.VersionId
        );
        // Content Stream と Meta を返す
        return (response.ResponseStream, meta);
    }

    /// <summary>
    /// HeadObjectAsync はオブジェクトのメタデータのみを取得する（ボディは取得しない）。
    /// オブジェクトが存在しない場合は null を返す。
    /// </summary>
    // HeadObjectAsync メソッド実装: AmazonS3Client.GetObjectMetadataAsync を呼び出す
    public async Task<StorageObjectMeta?> HeadObjectAsync(string bucket, string key, CancellationToken cancellationToken = default)
    {
        try
        {
            // GetObjectMetadataRequest を構築する
            var request = new GetObjectMetadataRequest
            {
                // バケット名を設定する
                BucketName = bucket,
                // キーを設定する
                Key = key,
            };
            // S3 の GetObjectMetadataAsync を呼び出す
            var response = await _s3.GetObjectMetadataAsync(request, cancellationToken).ConfigureAwait(false);
            // StorageObjectMeta に変換して返す
            return ToStorageObjectMeta(key, response);
        }
        catch (AmazonS3Exception ex) when (ex.StatusCode == System.Net.HttpStatusCode.NotFound)
        {
            // 404 の場合は null を返す（エラーと区別する）
            return null;
        }
    }

    /// <summary>
    /// DeleteObjectAsync はバケットからオブジェクトを削除する（idempotent 操作）。
    /// </summary>
    // DeleteObjectAsync メソッド実装: AmazonS3Client.DeleteObjectAsync を呼び出す
    public async Task DeleteObjectAsync(string bucket, string key, CancellationToken cancellationToken = default)
    {
        // DeleteObjectRequest を構築する
        var request = new DeleteObjectRequest
        {
            // バケット名を設定する
            BucketName = bucket,
            // キーを設定する
            Key = key,
        };
        // S3 の DeleteObjectAsync を呼び出す
        await _s3.DeleteObjectAsync(request, cancellationToken).ConfigureAwait(false);
    }

    /// <summary>
    /// ListObjectsAsync はバケット内のオブジェクトを列挙する。
    /// </summary>
    // ListObjectsAsync メソッド実装: AmazonS3Client.ListObjectsV2Async を呼び出す
    public async Task<StorageListResult> ListObjectsAsync(string bucket, StorageListOptions? opts = null, CancellationToken cancellationToken = default)
    {
        // ListObjectsV2Request を構築する
        var request = new ListObjectsV2Request
        {
            // バケット名を設定する
            BucketName = bucket,
            // プレフィックスを設定する（null の場合はルートから列挙する）
            Prefix = opts?.Prefix,
            // デリミタを設定する
            Delimiter = opts?.Delimiter,
            // MaxKeys を設定する（0 の場合は 1000 を使用する）
            MaxKeys = opts?.MaxKeys > 0 ? opts.MaxKeys : 1000,
            // 継続トークンを設定する（null の場合は最初のページ）
            ContinuationToken = opts?.ContinuationToken,
        };
        // S3 の ListObjectsV2Async を呼び出す
        var response = await _s3.ListObjectsV2Async(request, cancellationToken).ConfigureAwait(false);
        // StorageObjectMeta リストを構築する
        var objects = response.S3Objects.Select(o => new StorageObjectMeta(
            // キーを設定する
            Key: o.Key,
            // サイズを設定する
            SizeBytes: o.Size,
            // Content-Type は ListObjectsV2 では取得できないため空文字列とする
            ContentType: string.Empty,
            // ETag を設定する（引用符を除去する）
            ETag: (o.ETag ?? string.Empty).Trim('"'),
            // カスタムメタデータは空とする（ListObjectsV2 ではメタデータが取得できない）
            CustomMeta: new Dictionary<string, string>(),
            // バージョン ID は null（ListObjectsV2 ではバージョン情報なし）
            VersionId: null
        )).ToList();
        // StorageListResult を返す
        return new StorageListResult(
            // オブジェクトリストを設定する
            Objects: objects.AsReadOnly(),
            // CommonPrefixes を設定する
            CommonPrefixes: response.CommonPrefixes.AsReadOnly(),
            // 次ページのトークンを設定する（IsTruncated が false の場合は null）
            NextContinuationToken: response.IsTruncated ? response.NextContinuationToken : null,
            // IsTruncated を設定する
            IsTruncated: response.IsTruncated
        );
    }

    /// <summary>
    /// CopyObjectAsync は同一バケット内または異なるバケット間でオブジェクトをコピーする。
    /// </summary>
    // CopyObjectAsync メソッド実装: AmazonS3Client.CopyObjectAsync を呼び出す
    public async Task<StorageObjectMeta> CopyObjectAsync(
        string srcBucket,
        string srcKey,
        string dstBucket,
        string dstKey,
        CancellationToken cancellationToken = default)
    {
        // CopyObjectRequest を構築する
        var request = new CopyObjectRequest
        {
            // コピー元バケット名を設定する
            SourceBucket = srcBucket,
            // コピー元キーを設定する
            SourceKey = srcKey,
            // コピー先バケット名を設定する
            DestinationBucket = dstBucket,
            // コピー先キーを設定する
            DestinationKey = dstKey,
        };
        // S3 の CopyObjectAsync を呼び出す
        var response = await _s3.CopyObjectAsync(request, cancellationToken).ConfigureAwait(false);
        // コピー後のメタデータを取得する
        var meta = await HeadObjectAsync(dstBucket, dstKey, cancellationToken).ConfigureAwait(false);
        // meta が null の場合は簡易 StorageObjectMeta を返す
        return meta ?? new StorageObjectMeta(
            // コピー先キーを設定する
            Key: dstKey,
            // サイズは不明（0 を設定する）
            SizeBytes: 0,
            // Content-Type は不明（application/octet-stream を設定する）
            ContentType: "application/octet-stream",
            // ETag を設定する
            ETag: (response.ETag ?? string.Empty).Trim('"'),
            // カスタムメタデータは空とする
            CustomMeta: new Dictionary<string, string>(),
            // バージョン ID を設定する
            VersionId: response.VersionId
        );
    }

    /// <summary>
    /// PresignGetUrlAsync は GET 用署名付き URL を生成する。
    /// wall-clock TTL 禁止: opts.Ttl の HLC tick から有効期限を計算する。
    /// </summary>
    // PresignGetUrlAsync メソッド実装: AmazonS3Client.GetPreSignedURL を呼び出す
    public Task<string> PresignGetUrlAsync(string bucket, string key, PresignedUrlOptions opts, CancellationToken cancellationToken = default)
    {
        // HLC tick を TimeSpan に変換する（wall-clock 禁止: HLC ベースの有効期限計算）
        var ttl = TimeSpan.FromTicks((long)opts.Ttl.LogicalTicks);
        // GetPreSignedUrlRequest を構築する
        var request = new GetPreSignedUrlRequest
        {
            // バケット名を設定する
            BucketName = bucket,
            // キーを設定する
            Key = key,
            // HTTP GET メソッドを指定する
            Verb = HttpVerb.GET,
            // 有効期限を設定する（HLC TTL からの変換値）
            Expires = DateTime.UtcNow.Add(ttl),
        };
        // 署名付き URL を生成して返す
        var url = _s3.GetPreSignedURL(request);
        // 生成した URL を返す
        return Task.FromResult(url);
    }

    /// <summary>
    /// PresignPutUrlAsync は PUT 用署名付き URL を生成する。
    /// wall-clock TTL 禁止: opts.Ttl の HLC tick から有効期限を計算する。
    /// </summary>
    // PresignPutUrlAsync メソッド実装: AmazonS3Client.GetPreSignedURL を呼び出す
    public Task<string> PresignPutUrlAsync(string bucket, string key, PresignedUrlOptions opts, CancellationToken cancellationToken = default)
    {
        // HLC tick を TimeSpan に変換する（wall-clock 禁止: HLC ベースの有効期限計算）
        var ttl = TimeSpan.FromTicks((long)opts.Ttl.LogicalTicks);
        // GetPreSignedUrlRequest を構築する
        var request = new GetPreSignedUrlRequest
        {
            // バケット名を設定する
            BucketName = bucket,
            // キーを設定する
            Key = key,
            // HTTP PUT メソッドを指定する
            Verb = HttpVerb.PUT,
            // 有効期限を設定する（HLC TTL からの変換値）
            Expires = DateTime.UtcNow.Add(ttl),
            // Content-Type を設定する（null の場合は設定しない）
            ContentType = opts.ContentType,
        };
        // 署名付き URL を生成して返す
        var url = _s3.GetPreSignedURL(request);
        // 生成した URL を返す
        return Task.FromResult(url);
    }
}
