/**
 * storage.ts — k1s0 tier1 Library TypeScript 実装: Object Storage の L3 interface
 * 09_ストレージ適合仕様.md §ObjectStorageClient（OSS 中立 L3）に準拠する。
 * S3 / GCS / Azure Blob 等 OSS の API を一切露出しない Wire protocol 抽象 interface を宣言する。
 * 公開シグネチャに OSS 型を一切含まない。
 */

// CacheTtl は HLC ベース TTL のために再利用する（wall-clock TTL 禁止規約）
import type { CacheTtl } from "./cache.js";

/**
 * StorageObjectMeta はオブジェクトのメタデータを宣言する型。
 * OSS の HeadObject レスポンス型を露出せず Library 独自語彙で表現する。
 */
// StorageObjectMeta 型定義
export interface StorageObjectMeta {
  // key: オブジェクトキー（バケット内でユニーク）
  readonly key: string;
  // sizeBytes: オブジェクトのバイトサイズ
  readonly sizeBytes: number;
  // contentType: MIME type（"application/octet-stream" 等）
  readonly contentType: string;
  // etag: オブジェクトの整合性チェックサム（MD5 / SHA256 等）
  readonly etag: string;
  // customMeta: ユーザー定義メタデータ（tenant_id 等を格納する）
  readonly customMeta: Readonly<Record<string, string>>;
  // versionId: バージョニング対応バケットのオブジェクトバージョン識別子
  readonly versionId?: string | undefined;
}

/**
 * StoragePutOptions は putObject に渡すオプションを宣言する型。
 */
// StoragePutOptions 型定義
export interface StoragePutOptions {
  // contentType: アップロードするオブジェクトの MIME type
  readonly contentType?: string | undefined;
  // customMeta: ユーザー定義メタデータ（tenant_id 等）
  readonly customMeta?: Readonly<Record<string, string>> | undefined;
  // serverSideEncryption: サーバー側暗号化の設定（"AES256" / "aws:kms" 等）
  readonly serverSideEncryption?: string | undefined;
  // kmsKeyId: KMS を使用するサーバー側暗号化の KMS キー ID
  readonly kmsKeyId?: string | undefined;
}

/**
 * StorageGetOptions は getObject に渡すオプションを宣言する型。
 */
// StorageGetOptions 型定義
export interface StorageGetOptions {
  // versionId: 特定バージョンを取得する場合に設定する（未指定 = 最新バージョン）
  readonly versionId?: string | undefined;
  // rangeStart: Range 取得の開始バイトオフセット（0 = 先頭から）
  readonly rangeStart?: number | undefined;
  // rangeEnd: Range 取得の終了バイトオフセット（0 = 末尾まで）
  readonly rangeEnd?: number | undefined;
}

/**
 * StorageListOptions は listObjects に渡すオプションを宣言する型。
 */
// StorageListOptions 型定義
export interface StorageListOptions {
  // prefix: 列挙するキーのプレフィックスフィルター（未指定 = 全キー）
  readonly prefix?: string | undefined;
  // delimiter: 仮想ディレクトリ区切り文字（"/" 等）
  readonly delimiter?: string | undefined;
  // maxKeys: 一度に取得するキーの最大数（0 = 実装固有のデフォルト上限を使用する）
  readonly maxKeys?: number | undefined;
  // continuationToken: ページネーション継続トークン（未指定 = 最初のページ）
  readonly continuationToken?: string | undefined;
}

/**
 * StorageListResult は listObjects の結果を宣言する型。
 */
// StorageListResult 型定義
export interface StorageListResult {
  // objects: 取得したオブジェクトのメタデータ配列
  readonly objects: readonly StorageObjectMeta[];
  // commonPrefixes: 仮想ディレクトリのプレフィックス配列（delimiter 設定時のみ）
  readonly commonPrefixes: readonly string[];
  // nextContinuationToken: 次ページのトークン（未定義 = 最終ページ）
  readonly nextContinuationToken?: string | undefined;
  // isTruncated: 結果が切り詰められているかどうか
  readonly isTruncated: boolean;
}

/**
 * PresignedUrlOptions は presignGetUrl / presignPutUrl に渡すオプションを宣言する型。
 * wall-clock TTL 禁止規約に準拠して HLC ベースの TTL のみを受け付ける。
 */
// PresignedUrlOptions 型定義
export interface PresignedUrlOptions {
  // ttl: 署名付き URL の HLC ベース有効期限（必須: 無期限 URL は禁止する）
  readonly ttl: CacheTtl;
  // contentType: PUT 用署名付き URL の Content-Type 制約
  readonly contentType?: string | undefined;
}

/**
 * ObjectStorageClient は Object Storage の L3 抽象 interface を宣言する。
 * S3 / GCS / Azure Blob / MinIO 等を透過的に切り替え可能にする。
 * OSS 型を引数・戻り値に一切含まない。
 */
// ObjectStorageClient インターフェース定義
export interface ObjectStorageClient {
  /**
   * putObject はオブジェクトをバケットにアップロードする。
   * data は Uint8Array または ReadableStream（ストリーミングアップロードをサポートする）。
   * 戻り値は保存されたオブジェクトのメタデータ。
   */
  // putObject メソッド: オブジェクトをアップロードする
  putObject(
    bucket: string,
    key: string,
    data: Uint8Array | ReadableStream<Uint8Array>,
    opts?: StoragePutOptions,
  ): Promise<StorageObjectMeta>;

  /**
   * getObject はバケットからオブジェクトをダウンロードする。
   * 戻り値は ReadableStream（大容量ファイルのストリーミング対応）。
   */
  // getObject メソッド: オブジェクトをダウンロードする
  getObject(
    bucket: string,
    key: string,
    opts?: StorageGetOptions,
  ): Promise<[ReadableStream<Uint8Array>, StorageObjectMeta]>;

  /**
   * headObject はオブジェクトのメタデータのみを取得する（ボディは取得しない）。
   * オブジェクトが存在しない場合は null を返す（エラーと区別する）。
   */
  // headObject メソッド: オブジェクトのメタデータを取得する
  headObject(bucket: string, key: string): Promise<StorageObjectMeta | null>;

  /**
   * deleteObject はバケットからオブジェクトを削除する。
   * オブジェクトが存在しない場合はエラーを投げない（idempotent 操作）。
   */
  // deleteObject メソッド: オブジェクトを削除する
  deleteObject(bucket: string, key: string): Promise<void>;

  /**
   * listObjects はバケット内のオブジェクトを列挙する。
   */
  // listObjects メソッド: オブジェクトを列挙する
  listObjects(bucket: string, opts?: StorageListOptions): Promise<StorageListResult>;

  /**
   * copyObject は同一バケット内または異なるバケット間でオブジェクトをコピーする。
   */
  // copyObject メソッド: オブジェクトをコピーする
  copyObject(
    srcBucket: string,
    srcKey: string,
    dstBucket: string,
    dstKey: string,
  ): Promise<StorageObjectMeta>;

  /**
   * presignGetUrl は GET 用署名付き URL を生成する。
   * wall-clock TTL 禁止規約に準拠して opts.ttl は HLC ベースで指定する。
   */
  // presignGetUrl メソッド: GET 用署名付き URL を生成する
  presignGetUrl(bucket: string, key: string, opts: PresignedUrlOptions): Promise<string>;

  /**
   * presignPutUrl は PUT 用署名付き URL を生成する。
   * wall-clock TTL 禁止規約に準拠して opts.ttl は HLC ベースで指定する。
   */
  // presignPutUrl メソッド: PUT 用署名付き URL を生成する
  presignPutUrl(bucket: string, key: string, opts: PresignedUrlOptions): Promise<string>;
}

/**
 * MultipartUploadHandle は Multipart Upload セッションを宣言する型。
 * OSS の CreateMultipartUpload レスポンスを露出せず Library 独自語彙で表現する。
 */
// MultipartUploadHandle 型定義
export interface MultipartUploadHandle {
  // uploadId: Multipart Upload セッション識別子
  readonly uploadId: string;
  // bucket: 対象バケット名
  readonly bucket: string;
  // key: 対象オブジェクトキー
  readonly key: string;
}

/**
 * MultipartObjectStorageClient は大容量オブジェクトの Multipart Upload を追加サポートする interface。
 * ObjectStorageClient の上位 interface として宣言する。
 */
// MultipartObjectStorageClient インターフェース定義
export interface MultipartObjectStorageClient extends ObjectStorageClient {
  /**
   * createMultipartUpload は Multipart Upload セッションを開始する。
   */
  // createMultipartUpload メソッド: Multipart Upload を開始する
  createMultipartUpload(
    bucket: string,
    key: string,
    opts?: StoragePutOptions,
  ): Promise<MultipartUploadHandle>;

  /**
   * uploadPart は Multipart Upload のパートをアップロードする。
   * partNumber は 1 始まりのパート番号（1 〜 10000）。
   * 戻り値は ETag（completeMultipartUpload に必要）。
   */
  // uploadPart メソッド: パートをアップロードする
  uploadPart(
    handle: MultipartUploadHandle,
    partNumber: number,
    data: Uint8Array,
  ): Promise<string>;

  /**
   * completeMultipartUpload は全パートのアップロード完了を宣言してオブジェクトを確定する。
   * partEtags はパート番号順の ETag 配列（uploadPart の戻り値を順番に格納する）。
   */
  // completeMultipartUpload メソッド: Multipart Upload を確定する
  completeMultipartUpload(
    handle: MultipartUploadHandle,
    partEtags: readonly string[],
  ): Promise<StorageObjectMeta>;

  /**
   * abortMultipartUpload は Multipart Upload セッションを中止する。
   */
  // abortMultipartUpload メソッド: Multipart Upload を中止する
  abortMultipartUpload(handle: MultipartUploadHandle): Promise<void>;
}
