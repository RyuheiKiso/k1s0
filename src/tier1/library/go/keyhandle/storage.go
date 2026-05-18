// storage.go — k1s0 tier1 Library Go 実装: Object Storage の L3 interface
// 09_ストレージ適合仕様.md §ObjectStorageClient（OSS 中立 L3）に準拠する。
// S3 / GCS / Azure Blob 等 OSS の API を一切露出しない Wire protocol 抽象 interface を宣言する。
// 公開シグネチャに OSS 型（s3.Client / storage.Client 等）を一切含まない。

// パッケージ名: keyhandle（tier1 Library の Object Storage API を提供する）
package keyhandle

import (
	// context: context.Context（非同期操作に使用する）
	"context"
	// io: オブジェクト読み書きに io.Reader / io.Writer / io.ReadCloser を使用する
	"io"
)

// StorageObjectMeta はオブジェクトのメタデータを宣言する型。
// OSS の HeadObject レスポンス型を露出せず Library 独自語彙で表現する。
type StorageObjectMeta struct {
	// Key: オブジェクトキー（バケット内でユニーク）
	Key string
	// SizeBytes: オブジェクトのバイトサイズ
	SizeBytes int64
	// ContentType: MIME type（"application/octet-stream" 等）
	ContentType string
	// ETag: オブジェクトの整合性チェックサム（MD5 / SHA256 等）
	ETag string
	// CustomMeta: ユーザー定義メタデータ（tenant_id 等を格納する）
	CustomMeta map[string]string
	// VersionID: バージョニング対応バケットのオブジェクトバージョン識別子
	VersionID string
}

// StoragePutOptions は PutObject / PutObjectMultipart に渡すオプションを宣言する型。
type StoragePutOptions struct {
	// ContentType: アップロードするオブジェクトの MIME type
	ContentType string
	// CustomMeta: ユーザー定義メタデータ（tenant_id 等）
	CustomMeta map[string]string
	// ServerSideEncryption: サーバー側暗号化の設定（"AES256" / "aws:kms" 等）
	ServerSideEncryption string
	// KMSKeyID: KMS を使用するサーバー側暗号化の KMS キー ID
	KMSKeyID string
}

// StorageGetOptions は GetObject に渡すオプションを宣言する型。
type StorageGetOptions struct {
	// VersionID: 特定バージョンを取得する場合に設定する（空文字列 = 最新バージョン）
	VersionID string
	// RangeStart: Range 取得の開始バイトオフセット（0 = 先頭から）
	RangeStart int64
	// RangeEnd: Range 取得の終了バイトオフセット（0 = 末尾まで）
	RangeEnd int64
}

// StorageListOptions は ListObjects に渡すオプションを宣言する型。
type StorageListOptions struct {
	// Prefix: 列挙するキーのプレフィックスフィルター（空文字列 = 全キー）
	Prefix string
	// Delimiter: 仮想ディレクトリ区切り文字（"/" 等）
	Delimiter string
	// MaxKeys: 一度に取得するキーの最大数（0 = 実装固有のデフォルト上限を使用する）
	MaxKeys int
	// ContinuationToken: ページネーション継続トークン（空文字列 = 最初のページ）
	ContinuationToken string
}

// StorageListResult は ListObjects の結果を宣言する型。
type StorageListResult struct {
	// Objects: 取得したオブジェクトのメタデータスライス
	Objects []StorageObjectMeta
	// CommonPrefixes: 仮想ディレクトリのプレフィックススライス（Delimiter 設定時のみ）
	CommonPrefixes []string
	// NextContinuationToken: 次ページのトークン（空文字列 = 最終ページ）
	NextContinuationToken string
	// IsTruncated: 結果が切り詰められているかどうか（true = 続くページが存在する）
	IsTruncated bool
}

// PresignedURLOptions は PresignGetURL / PresignPutURL に渡すオプションを宣言する型。
// wall-clock TTL 禁止規約に準拠して HLC ベースの TTL のみを受け付ける。
type PresignedURLOptions struct {
	// TTL: 署名付き URL の HLC ベース有効期限（必須: 無期限 URL は禁止する）
	TTL CacheTTL
	// ContentType: PUT 用署名付き URL の Content-Type 制約
	ContentType string
}

// ObjectStorageClient は Object Storage の L3 抽象 interface を宣言する。
// S3 / GCS / Azure Blob / MinIO 等を透過的に切り替え可能にする。
// OSS 型（s3.Client / storage.Client 等）を引数・戻り値に一切含まない。
type ObjectStorageClient interface {
	// PutObject はオブジェクトをバケットにアップロードする。
	// bucket は対象バケット名、key はオブジェクトキー。
	// r はオブジェクトデータを提供する io.Reader（ストリーミングアップロードをサポートする）。
	// sizeHint はデータサイズのヒント（-1 = 不明）。
	// 戻り値は保存されたオブジェクトのメタデータ。
	PutObject(ctx context.Context, bucket, key string, r io.Reader, sizeHint int64, opts *StoragePutOptions) (*StorageObjectMeta, error)

	// GetObject はバケットからオブジェクトをダウンロードする。
	// 戻り値の io.ReadCloser は使用後に必ず Close する（goroutine / メモリリーク防止）。
	GetObject(ctx context.Context, bucket, key string, opts *StorageGetOptions) (io.ReadCloser, *StorageObjectMeta, error)

	// HeadObject はオブジェクトのメタデータのみを取得する（ボディは取得しない）。
	// オブジェクトが存在しない場合は nil, nil を返す（エラーと区別する）。
	HeadObject(ctx context.Context, bucket, key string) (*StorageObjectMeta, error)

	// DeleteObject はバケットからオブジェクトを削除する。
	// オブジェクトが存在しない場合はエラーを返さない（idempotent 操作）。
	DeleteObject(ctx context.Context, bucket, key string) error

	// ListObjects はバケット内のオブジェクトを列挙する。
	// opts でプレフィックスフィルター・ページネーションを指定する。
	ListObjects(ctx context.Context, bucket string, opts *StorageListOptions) (*StorageListResult, error)

	// CopyObject は同一バケット内または異なるバケット間でオブジェクトをコピーする。
	// srcBucket / srcKey はコピー元、dstBucket / dstKey はコピー先。
	CopyObject(ctx context.Context, srcBucket, srcKey, dstBucket, dstKey string) (*StorageObjectMeta, error)

	// PresignGetURL は GET 用署名付き URL を生成する（認証なしでのダウンロードを許可する）。
	// wall-clock TTL 禁止規約に準拠して opts.TTL は HLC ベースで指定する。
	PresignGetURL(ctx context.Context, bucket, key string, opts PresignedURLOptions) (string, error)

	// PresignPutURL は PUT 用署名付き URL を生成する（認証なしでのアップロードを許可する）。
	// wall-clock TTL 禁止規約に準拠して opts.TTL は HLC ベースで指定する。
	PresignPutURL(ctx context.Context, bucket, key string, opts PresignedURLOptions) (string, error)
}

// MultipartUploadHandle は Multipart Upload セッションを宣言する型。
// OSS の CreateMultipartUpload レスポンスを露出せず Library 独自語彙で表現する。
type MultipartUploadHandle struct {
	// UploadID: Multipart Upload セッション識別子
	UploadID string
	// Bucket: 対象バケット名
	Bucket string
	// Key: 対象オブジェクトキー
	Key string
}

// MultipartObjectStorageClient は大容量オブジェクトの Multipart Upload を追加サポートする interface。
// ObjectStorageClient の上位 interface として宣言する。
type MultipartObjectStorageClient interface {
	// ObjectStorageClient の全メソッドを継承する
	ObjectStorageClient

	// CreateMultipartUpload は Multipart Upload セッションを開始する。
	// 戻り値の MultipartUploadHandle を使用してパートのアップロードを行う。
	CreateMultipartUpload(ctx context.Context, bucket, key string, opts *StoragePutOptions) (*MultipartUploadHandle, error)

	// UploadPart は Multipart Upload のパートをアップロードする。
	// handle は CreateMultipartUpload で取得したハンドル。
	// partNumber は 1 始まりのパート番号（1 〜 10000）。
	// r はパートデータの io.Reader、sizeHint はパートサイズのヒント。
	// 戻り値の string は ETag（CompleteMultipartUpload に必要）。
	UploadPart(ctx context.Context, handle *MultipartUploadHandle, partNumber int, r io.Reader, sizeHint int64) (string, error)

	// CompleteMultipartUpload は全パートのアップロード完了を宣言してオブジェクトを確定する。
	// partETags はパート番号順の ETag スライス（UploadPart の戻り値を順番に格納する）。
	CompleteMultipartUpload(ctx context.Context, handle *MultipartUploadHandle, partETags []string) (*StorageObjectMeta, error)

	// AbortMultipartUpload は Multipart Upload セッションを中止する（パートをクリーンアップする）。
	AbortMultipartUpload(ctx context.Context, handle *MultipartUploadHandle) error
}
