/**
 * k1s0 tier2 attachment TypeScript インターフェース定義
 * Rust 実装（attachment/src/attachment_store.rs）と 4 言語等価強度を持つ TypeScript 版
 * テナント分離されたオブジェクトストレージへの添付ファイル操作を抽象化する（設計方針 28）
 */

/**
 * AttachmentMetadata: アップロード完了後に返すメタデータ型
 * Rust の AttachmentMetadata 構造体に対応する
 */
// AttachmentMetadata インターフェース定義
export interface AttachmentMetadata {
  // 添付ファイルの一意識別子（UUID v4）
  readonly attachmentId: string;
  // アップロード先テナント識別子（RLS で自動フィルタリングされる）
  readonly tenantId: string;
  // ファイルの MIME タイプ（例: application/pdf / image/png）
  readonly contentType: string;
  // アップロードされたファイルのバイトサイズ
  readonly sizeBytes: bigint;
  // ストレージ内のオブジェクトキー（テナント分離プレフィックス付き）
  readonly objectKey: string;
  // HLC タイムスタンプ（wall clock TTL 禁止規約により HLC を使用する）
  readonly hlcTimestamp: bigint;
  // エンベロープ暗号化で使用された DEK ハンドル（OpenBao Transit で管理する）
  readonly dekHandle: string;
}

/**
 * IAttachmentStore: テナント分離されたオブジェクトストレージへの操作インターフェース
 * Rust の AttachmentStore トレイトに対応する（設計方針 28）
 * 実装クラスは MinIO または S3 互換ストレージと連携する
 * tenantId は AuthContext から取得するため API 引数で受け取らない（直接渡し禁止）
 */
// IAttachmentStore インターフェース定義
export interface IAttachmentStore {
  /**
   * テナント分離されたオブジェクトストレージに添付ファイルをアップロードする
   * tenantId: アップロード先テナント識別子（AuthContext から注入する / API 引数で受け取り禁止）
   * data: アップロードするファイルの Uint8Array
   * contentType: ファイルの MIME タイプ
   * アップロード完了後に AttachmentMetadata を返す
   */
  // upload メソッド（テナント分離ファイルアップロード操作）
  upload(tenantId: string, data: Uint8Array, contentType: string): Promise<AttachmentMetadata>;

  /**
   * 添付ファイルを取得する
   * tenantId: 取得対象テナント識別子（RLS で自動フィルタリングされる）
   * attachmentId: 取得対象添付ファイル識別子
   * ファイルの Uint8Array を返す
   */
  // fetch メソッド（添付ファイル取得操作）
  fetch(tenantId: string, attachmentId: string): Promise<Uint8Array>;

  /**
   * 添付ファイルを削除する（PII データ削除フローから呼び出される）
   * tenantId: 削除対象テナント識別子
   * attachmentId: 削除対象添付ファイル識別子
   */
  // delete メソッド（添付ファイル削除操作）
  delete(tenantId: string, attachmentId: string): Promise<void>;
}
