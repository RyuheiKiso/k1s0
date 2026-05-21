// Package admin の bulk_import: 業務マスタ CSV 一括インポート（Go 実装）
// 10_テナント分離適合仕様.md §admin_operation + RLS FORCE 対応
// Rust 実装（admin/src/bulk_import.rs）と 4 言語等価強度を持つ Go 版
// AdminBoundaryGuard を通過した後に呼び出すこと（境界チェック bypass 禁止）

// パッケージ名: admin（admin.go と同一パッケージ）
package admin

import (
	// context パッケージ: リクエストスコープに使用する
	"context"
	// fmt パッケージ: エラーメッセージのフォーマットに使用する
	"fmt"
)

// BulkImportConfig は CSV バルクインポートの設定を保持する
// TenantID は AuthContext から取得するため API 引数に露出しない（tier2 コーディング規約準拠）
type BulkImportConfig struct {
	// TenantID: インポート先テナント識別子（AuthContext から取得済みの値を格納する）
	TenantID string
	// MasterType: 業務マスタ種別（通貨レート / 製品カタログ等）の識別子（業界中立語のみ）
	MasterType string
	// CSVURL: インポート対象 CSV の署名付き URL（オブジェクトストレージ上の一時 URL）
	CSVURL string
	// RLSForce: RLS FORCE フラグ（true に固定すること。false は CI fail 対象）
	RLSForce bool
}

// BulkImportResult はインポート結果を保持する
type BulkImportResult struct {
	// InsertedCount: INSERT に成功した行数（0 以上）
	InsertedCount int
}

// ExecuteBulkImport は業務マスタ CSV を一括インポートする
// 1. CSV スキーマ検証 → 2. RLS FORCE 付き atomic INSERT → 3. 監査イベント emit の順で実行する
// いずれかのステップが失敗した場合はトランザクション全体をロールバックする（atomic 三表書込規約準拠）
func ExecuteBulkImport(ctx context.Context, cfg BulkImportConfig) (*BulkImportResult, error) {
	// RLS FORCE フラグが true でない場合は境界違反エラーを返す
	if !cfg.RLSForce {
		return nil, &AdminBoundaryError{
			Kind: ErrKindInsufficientScope,
			Msg:  "bulk_import: RLSForce は true でなければならない（テナント分離保証）",
		}
	}
	// ステップ 1: CSV スキーマ検証（master_type に応じた検証ロジックに委譲する）
	if err := validateCSVSchema(ctx, cfg); err != nil {
		return nil, fmt.Errorf("bulk_import: CSV スキーマ検証失敗: %w", err)
	}
	// ステップ 2: RLS FORCE 付き atomic 一括 INSERT（テナント境界を DB 層で強制する）
	inserted, err := atomicBulkInsert(ctx, cfg)
	if err != nil {
		return nil, fmt.Errorf("bulk_import: 一括 INSERT 失敗: %w", err)
	}
	// ステップ 3: 監査イベントを audit_local テーブル経由で emit する
	if err := emitAuditEvent(ctx, cfg, inserted); err != nil {
		return nil, fmt.Errorf("bulk_import: 監査イベント emit 失敗: %w", err)
	}
	// インポート結果を返す
	return &BulkImportResult{InsertedCount: inserted}, nil
}

// validateCSVSchema は master_type に応じた CSV スキーマ検証を実行する（stub 実装）
func validateCSVSchema(ctx context.Context, cfg BulkImportConfig) error {
	// master_type に応じた CSV スキーマ検証を実行する（実装は業種別 master_type に委譲）
	_ = ctx
	_ = cfg.CSVURL
	_ = cfg.MasterType
	return nil
}

// atomicBulkInsert は RLS FORCE 付き atomic 一括 INSERT を実行する（stub 実装）
func atomicBulkInsert(ctx context.Context, cfg BulkImportConfig) (int, error) {
	// SET LOCAL row_security = FORCE で tenant_id を強制注入してからバルク INSERT する
	_ = ctx
	_ = cfg.TenantID
	// 挿入行数 0 を返す（stub 実装）
	return 0, nil
}

// emitAuditEvent は監査イベントを audit_local テーブル経由で emit する（stub 実装）
func emitAuditEvent(ctx context.Context, cfg BulkImportConfig, count int) error {
	// tenant_id / master_type / 挿入行数を監査イベントのフィールドとして記録する
	_ = ctx
	_ = cfg.TenantID
	_ = cfg.MasterType
	_ = count
	return nil
}
