// k1s0 tier2 attachment Go インターフェース定義
// Rust 実装（attachment/src/attachment_store.rs）と 4 言語等価強度を持つ Go 版
// テナント分離されたオブジェクトストレージへの添付ファイル操作を抽象化する（設計方針 28）

// パッケージ名: attachment
package attachment

import (
	// context パッケージ: リクエストスコープおよびキャンセルに使用する
	"context"

	// uuid パッケージ: テナント識別子および添付ファイル識別子型に使用する
	"github.com/google/uuid"
)

// AttachmentMetadata: アップロード完了後に返すメタデータ構造体
// Rust の AttachmentMetadata 構造体に対応する
type AttachmentMetadata struct {
	// 添付ファイルの一意識別子（UUID v4）
	AttachmentID uuid.UUID `json:"attachment_id"`
	// アップロード先テナント識別子（RLS で自動フィルタリングされる）
	TenantID uuid.UUID `json:"tenant_id"`
	// ファイルの MIME タイプ（例: application/pdf / image/png）
	ContentType string `json:"content_type"`
	// アップロードされたファイルのバイトサイズ
	SizeBytes uint64 `json:"size_bytes"`
	// ストレージ内のオブジェクトキー（テナント分離プレフィックス付き）
	ObjectKey string `json:"object_key"`
	// HLC タイムスタンプ（wall clock TTL 禁止規約により HLC を使用する）
	HlcTimestamp uint64 `json:"hlc_timestamp"`
	// エンベロープ暗号化で使用された DEK ハンドル（OpenBao Transit で管理する）
	DekHandle string `json:"dek_handle"`
}

// AttachmentStore: テナント分離されたオブジェクトストレージへの操作インターフェース
// Rust の AttachmentStore トレイトに対応する（設計方針 28）
// 実装クラスは MinIO または S3 互換ストレージと連携する
// tenant_id は AuthContext から取得するため API 引数で受け取らない（直接渡し禁止）
type AttachmentStore interface {
	// Upload: テナント分離されたオブジェクトストレージに添付ファイルをアップロードする
	// tenantID: アップロード先テナント識別子（AuthContext から注入する / API 引数で受け取り禁止）
	// data: アップロードするファイルのバイトスライス
	// contentType: ファイルの MIME タイプ
	// アップロード完了後に *AttachmentMetadata を返す
	Upload(ctx context.Context, tenantID uuid.UUID, data []byte, contentType string) (*AttachmentMetadata, error)

	// Fetch: 添付ファイルを取得する
	// tenantID: 取得対象テナント識別子（RLS で自動フィルタリングされる）
	// attachmentID: 取得対象添付ファイル識別子
	// ファイルのバイトスライスを返す
	Fetch(ctx context.Context, tenantID uuid.UUID, attachmentID uuid.UUID) ([]byte, error)

	// Delete: 添付ファイルを削除する（PII データ削除フローから呼び出される）
	// tenantID: 削除対象テナント識別子
	// attachmentID: 削除対象添付ファイル識別子
	Delete(ctx context.Context, tenantID uuid.UUID, attachmentID uuid.UUID) error
}
