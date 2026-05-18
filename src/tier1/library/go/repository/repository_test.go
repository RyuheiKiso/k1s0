// repository_test.go — k1s0 tier1 Library Go repository パッケージのユニットテスト
// Repository interface のユーティリティ関数（VerifyTenantID / CheckTenantScopeFunc）を検証する。
// 04_認証適合仕様.md §CI 不変条件 整合 5「生 SQL 文字列受付 API 禁止」に準拠する。
// parity_vectors.yaml §repository_tenant_scope_query に対応する。

// パッケージ名: repository_test（ブラックボックステストパッケージ）
package repository_test

import (
	// errors: エラー判定に使用する
	"errors"
	// testing: Go テストフレームワークに使用する
	"testing"

	// repository パッケージのインポート: テスト対象パッケージ
	repo "github.com/k1s0-io/k1s0/tier1/library/repository"
)

// TestVerifyTenantID_Match は同一テナント ID の場合に nil が返されることを検証する
// RLS bypass 検出がアプリ層でも二重検証される設計を確認する
func TestVerifyTenantID_Match(t *testing.T) {
	// 同一テナント ID を設定する
	entityTenantID := "tenant-001"
	// コンテキストのテナント ID も同一値を設定する
	contextTenantID := "tenant-001"
	// VerifyTenantID を呼び出す
	err := repo.VerifyTenantID(entityTenantID, contextTenantID)
	// エラーが nil であることを確認する（一致するので正常）
	if err != nil {
		// エラーが返された場合はテスト失敗
		t.Errorf("VerifyTenantID: expected nil error for matching tenant IDs, got %v", err)
	}
}

// TestVerifyTenantID_Mismatch は異なるテナント ID の場合に ErrRLSBypass が返されることを検証する
// RLS bypass 検出の正常動作を確認する
func TestVerifyTenantID_Mismatch(t *testing.T) {
	// エンティティのテナント ID を設定する
	entityTenantID := "tenant-001"
	// コンテキストのテナント ID は異なる値を設定する（不一致を意図する）
	contextTenantID := "tenant-999"
	// VerifyTenantID を呼び出す
	err := repo.VerifyTenantID(entityTenantID, contextTenantID)
	// エラーが nil でないことを確認する（不一致なのでエラーが期待値）
	if err == nil {
		// エラーが nil の場合はテスト失敗（不一致なのでエラーが期待値）
		t.Fatal("VerifyTenantID: expected ErrRLSBypass for mismatched tenant IDs, got nil")
	}
	// エラーが ErrRLSBypass であることを確認する
	if !errors.Is(err, repo.ErrRLSBypass) {
		// ErrRLSBypass でない場合はテスト失敗
		t.Errorf("VerifyTenantID: expected ErrRLSBypass, got %v", err)
	}
}

// TestCheckTenantScopeFunc_InScope は有効な tenantID と resourceID の場合に in_scope=true が返されることを検証する
// parity_vectors.yaml §repository_tenant_scope_query に対応する
func TestCheckTenantScopeFunc_InScope(t *testing.T) {
	// parity_vectors.yaml §repository_tenant_scope_query の入力値と同一値を使用する
	tenantID := "test-tenant-001"
	// リソース ID: parity_vectors.yaml §repository_tenant_scope_query の入力値
	resourceID := "resource-001"
	// CheckTenantScopeFunc を呼び出す
	result := repo.CheckTenantScopeFunc(tenantID, resourceID)
	// in_scope が true であることを確認する（テナントスコープ内）
	if !result.InScope {
		// InScope が false の場合はテスト失敗
		t.Errorf("CheckTenantScopeFunc: expected InScope=true for tenant=%s resource=%s, got false",
			tenantID, resourceID)
	}
}

// TestCheckTenantScopeFunc_EmptyTenantID は空の tenantID の場合に in_scope=false が返されることを検証する
// エラーケースの動作確認
func TestCheckTenantScopeFunc_EmptyTenantID(t *testing.T) {
	// 空のテナント ID を設定する（無効な入力）
	emptyTenantID := ""
	// 有効なリソース ID を設定する
	resourceID := "resource-001"
	// CheckTenantScopeFunc を呼び出す
	result := repo.CheckTenantScopeFunc(emptyTenantID, resourceID)
	// in_scope が false であることを確認する（空の tenantID はスコープ外）
	if result.InScope {
		// InScope が true の場合はテスト失敗
		t.Errorf("CheckTenantScopeFunc: expected InScope=false for empty tenantID, got true")
	}
}
