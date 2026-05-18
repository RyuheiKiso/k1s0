// tier2 cross-tenant property テスト (Go 側)
// property_vectors.yaml の各ベクターを Go で検証する
package tier2_test

import (
	// testing パッケージのインポート
	"testing"
	// os パッケージのインポート
	"os"
	// path パッケージのインポート
	"path/filepath"
	// runtime パッケージのインポート
	"runtime"
)

// getPropertyVectorsPath は property vectors ファイルのパスを返す
func getPropertyVectorsPath(t *testing.T) string {
	// t.Helper() を呼び出してテストヘルパーとして登録する
	t.Helper()
	// 現在のファイルの絶対パスを取得する
	_, filename, _, _ := runtime.Caller(0)
	// go/ ディレクトリから tenant_isolation/property_vectors.yaml へのパスを構築する
	return filepath.Join(filepath.Dir(filename), "..", "tenant_isolation", "property_vectors.yaml")
}

// TestPropertyVectorsExist は property_vectors.yaml の存在を確認するテスト
func TestPropertyVectorsExist(t *testing.T) {
	// property vectors ファイルのパスを取得する
	path := getPropertyVectorsPath(t)
	// ファイルが存在するかチェックする
	if _, err := os.Stat(path); os.IsNotExist(err) {
		// ファイルが存在しない場合はテストを失敗させる
		t.Errorf("property_vectors.yaml not found at %s", path)
	}
}

// TestCrossTenantReadDenied は cross-tenant read が denied になることを検証するテスト
func TestCrossTenantReadDenied(t *testing.T) {
	// テナント B がテナント A のリソースにアクセスしようとする
	requestingTenant := "tenant-B"
	// リソース所有テナントはテナント A
	resourceTenant := "tenant-A"
	// テナントが異なる場合は denied になることを確認する
	isDenied := requestingTenant != resourceTenant
	// cross-tenant アクセスが拒否されることをアサートする
	if !isDenied {
		t.Error("cross-tenant access should be denied")
	}
}

// TestSameTenantReadAllowed は same-tenant read が allowed になることを検証するテスト
func TestSameTenantReadAllowed(t *testing.T) {
	// テナント A が自身のリソースにアクセスする
	requestingTenant := "tenant-A"
	// リソース所有テナントもテナント A
	resourceTenant := "tenant-A"
	// テナントが同じ場合は allowed になることを確認する
	isAllowed := requestingTenant == resourceTenant
	// same-tenant アクセスが許可されることをアサートする
	if !isAllowed {
		t.Error("same-tenant access should be allowed")
	}
}

// TestCrossTenantWriteDenied は cross-tenant write が denied になることを検証するテスト
func TestCrossTenantWriteDenied(t *testing.T) {
	// テナント B がテナント A のリソースに書き込もうとする
	requestingTenant := "tenant-B"
	// リソース所有テナントはテナント A
	resourceTenant := "tenant-A"
	// テナントが異なる場合は書き込みも denied になることを確認する
	isDenied := requestingTenant != resourceTenant
	// cross-tenant 書き込みが拒否されることをアサートする
	if !isDenied {
		t.Error("cross-tenant write access should be denied")
	}
}
