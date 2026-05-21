// k1s0 tier3 Go 添付ファイルストア テスト（4 言語等価強度検証）
// Rust / TypeScript 側のテストと同等の振る舞いを Go で検証する
// spec arch.tier3 §26_添付帳票 UX: MIME 検査 / ハッシュチェーン整合性 / signed URL 生成を検証する
package attachments

// testing パッケージをインポートする
import (
	// strings パッケージ（URL プレフィックス検査に使用する）
	"strings"
	// testing パッケージ（テストフレームワーク）
	"testing"
)

// --------- CheckMimeType テスト ---------

// TestCheckMimeType_Allowed は許可 MIME タイプが true を返すことを確認する
// Rust test_check_mime_type_allowed と等価のテスト
func TestCheckMimeType_Allowed(t *testing.T) {
	// PDF は許可されているので true を返すことを確認する
	if !CheckMimeType("application/pdf") {
		// 許可 MIME タイプが false を返すのは異常
		t.Error("application/pdf は許可 MIME タイプのため true を返すこと")
	}
	// JPEG は許可されているので true を返すことを確認する
	if !CheckMimeType("image/jpeg") {
		// 許可 MIME タイプが false を返すのは異常
		t.Error("image/jpeg は許可 MIME タイプのため true を返すこと")
	}
	// CSV は許可されているので true を返すことを確認する
	if !CheckMimeType("text/csv") {
		// 許可 MIME タイプが false を返すのは異常
		t.Error("text/csv は許可 MIME タイプのため true を返すこと")
	}
}

// TestCheckMimeType_Denied は拒否 MIME タイプが false を返すことを確認する
// Rust test_check_mime_type_denied と等価のテスト
func TestCheckMimeType_Denied(t *testing.T) {
	// 実行可能ファイルは拒否されているので false を返すことを確認する
	if CheckMimeType("application/x-executable") {
		// 拒否 MIME タイプが true を返すのは異常
		t.Error("application/x-executable は拒否 MIME タイプのため false を返すこと")
	}
	// ZIP は拒否されているので false を返すことを確認する
	if CheckMimeType("application/zip") {
		// 拒否 MIME タイプが true を返すのは異常
		t.Error("application/zip は拒否 MIME タイプのため false を返すこと")
	}
}

// --------- BuildHashChain / VerifyHashChain テスト ---------

// TestBuildHashChain はハッシュチェーン文字列が正しく生成されることを確認する
func TestBuildHashChain(t *testing.T) {
	// 2 チャンク分のハッシュ配列を用意する
	hashes := []string{"aabbcc", "ddeeff"}
	// ハッシュチェーン文字列を生成する
	chain := BuildHashChain(hashes)
	// 期待値: "sha256:aabbcc|sha256:ddeeff"
	expected := "sha256:aabbcc|sha256:ddeeff"
	// 生成されたチェーンが期待値と一致することを確認する
	if chain != expected {
		// 不一致は異常
		t.Errorf("BuildHashChain: got %q, want %q", chain, expected)
	}
}

// TestVerifyHashChain_Match は正しいハッシュ配列で true を返すことを確認する
func TestVerifyHashChain_Match(t *testing.T) {
	// チャンクハッシュ配列を用意する
	hashes := []string{"aabbcc", "ddeeff"}
	// メタデータを構築する（正しい hashChain を設定する）
	metadata := AttachmentMetadata{
		// 任意の UUID
		ID: "test-id-001",
		// テナント識別子
		TenantID: "tenant-001",
		// MIME タイプ
		MimeType: "application/pdf",
		// ファイルサイズ
		SizeBytes: 1024,
		// 正しいハッシュチェーン
		HashChain: "sha256:aabbcc|sha256:ddeeff",
		// アップロード時刻
		UploadedAt: "2026-05-20T00:00:00Z",
	}
	// VerifyHashChain が true を返すことを確認する
	if !VerifyHashChain(metadata, hashes) {
		// 一致するはずが false を返すのは異常
		t.Error("VerifyHashChain: 正しいハッシュ配列で true を返すこと")
	}
}

// TestVerifyHashChain_Mismatch はハッシュ不一致時に false を返すことを確認する
func TestVerifyHashChain_Mismatch(t *testing.T) {
	// 改ざんされたハッシュ配列を用意する
	tamperedHashes := []string{"000000", "ddeeff"}
	// メタデータを構築する（正しい hashChain を設定する）
	metadata := AttachmentMetadata{
		// 任意の UUID
		ID: "test-id-002",
		// テナント識別子
		TenantID: "tenant-001",
		// MIME タイプ
		MimeType: "application/pdf",
		// ファイルサイズ
		SizeBytes: 1024,
		// 正しいハッシュチェーン（改ざん前）
		HashChain: "sha256:aabbcc|sha256:ddeeff",
		// アップロード時刻
		UploadedAt: "2026-05-20T00:00:00Z",
	}
	// VerifyHashChain が false を返すことを確認する
	if VerifyHashChain(metadata, tamperedHashes) {
		// 不一致のはずが true を返すのは異常
		t.Error("VerifyHashChain: 不一致ハッシュ配列で false を返すこと")
	}
}

// --------- InMemoryAttachmentStore テスト ---------

// TestInMemoryAttachmentStore_UploadAndDownload は upload → download のラウンドトリップを確認する
func TestInMemoryAttachmentStore_UploadAndDownload(t *testing.T) {
	// InMemoryAttachmentStore を生成する
	store := NewInMemoryAttachmentStore()
	// テスト用のバイナリデータを用意する
	data := []byte("test attachment content")
	// PDF MIME タイプでアップロードする
	meta, err := store.Upload("tenant-001", "test.pdf", "application/pdf", data)
	// アップロードがエラーなく完了することを確認する
	if err != nil {
		// アップロードエラーは異常
		t.Fatalf("Upload エラー: %v", err)
	}
	// メタデータが nil でないことを確認する
	if meta == nil {
		// メタデータが nil は異常
		t.Fatal("Upload: メタデータが nil")
	}
	// ID が空文字でないことを確認する
	if meta.ID == "" {
		// ID が空文字は異常
		t.Error("Upload: ID が空文字")
	}
	// TenantID が正しく設定されていることを確認する
	if meta.TenantID != "tenant-001" {
		// TenantID 不一致は異常
		t.Errorf("Upload: TenantID got %q, want %q", meta.TenantID, "tenant-001")
	}
	// HashChain が空文字でないことを確認する
	if meta.HashChain == "" {
		// HashChain が空文字は異常
		t.Error("Upload: HashChain が空文字")
	}
	// 同じ ID でダウンロードする
	downloaded, err := store.Download(meta.ID)
	// ダウンロードがエラーなく完了することを確認する
	if err != nil {
		// ダウンロードエラーは異常
		t.Fatalf("Download エラー: %v", err)
	}
	// ダウンロードしたデータがアップロード時と一致することを確認する
	if string(downloaded) != string(data) {
		// データ不一致は異常
		t.Errorf("Download: got %q, want %q", string(downloaded), string(data))
	}
}

// TestInMemoryAttachmentStore_DeniedMimeType は拒否 MIME タイプでエラーになることを確認する
func TestInMemoryAttachmentStore_DeniedMimeType(t *testing.T) {
	// InMemoryAttachmentStore を生成する
	store := NewInMemoryAttachmentStore()
	// ZIP（拒否 MIME タイプ）でアップロードを試みる
	_, err := store.Upload("tenant-001", "evil.zip", "application/zip", []byte("evil"))
	// エラーが返ることを確認する
	if err == nil {
		// エラーなしは異常（拒否 MIME タイプは必ずエラーになる）
		t.Error("拒否 MIME タイプでのアップロードはエラーになること")
	}
}

// TestInMemoryAttachmentStore_Delete は削除後にダウンロードがエラーになることを確認する
func TestInMemoryAttachmentStore_Delete(t *testing.T) {
	// InMemoryAttachmentStore を生成する
	store := NewInMemoryAttachmentStore()
	// PDF でアップロードする
	meta, err := store.Upload("tenant-001", "delete_me.pdf", "application/pdf", []byte("to delete"))
	// アップロードがエラーなく完了することを確認する
	if err != nil {
		// アップロードエラーは異常
		t.Fatalf("Upload エラー: %v", err)
	}
	// 削除する
	if delErr := store.Delete(meta.ID); delErr != nil {
		// 削除エラーは異常
		t.Fatalf("Delete エラー: %v", delErr)
	}
	// 削除後にダウンロードするとエラーになることを確認する
	_, dlErr := store.Download(meta.ID)
	// エラーが返ることを確認する
	if dlErr == nil {
		// 削除後のダウンロードがエラーにならないのは異常
		t.Error("削除後のダウンロードはエラーになること")
	}
}

// TestInMemoryAttachmentStore_GenerateSignedURL は signed URL 生成を確認する
func TestInMemoryAttachmentStore_GenerateSignedURL(t *testing.T) {
	// InMemoryAttachmentStore を生成する
	store := NewInMemoryAttachmentStore()
	// PDF でアップロードする
	meta, err := store.Upload("tenant-001", "signed.pdf", "application/pdf", []byte("signed content"))
	// アップロードがエラーなく完了することを確認する
	if err != nil {
		// アップロードエラーは異常
		t.Fatalf("Upload エラー: %v", err)
	}
	// 有効期限 300 秒で signed URL を生成する
	url, urlErr := store.GenerateSignedURL(meta.ID, 300)
	// エラーなく URL が生成されることを確認する
	if urlErr != nil {
		// signed URL 生成エラーは異常
		t.Fatalf("GenerateSignedURL エラー: %v", urlErr)
	}
	// URL が空文字でないことを確認する
	if url == "" {
		// 空文字 URL は異常
		t.Error("GenerateSignedURL: URL が空文字")
	}
	// URL が "https://" で始まることを確認する
	if !strings.HasPrefix(url, "https://") {
		// https 以外のプロトコルは異常
		t.Errorf("GenerateSignedURL: URL は https:// で始まること: got %q", url)
	}
}

// TestCreateSandboxURL は sandbox iframe 用の URL 形式を確認する
func TestCreateSandboxURL(t *testing.T) {
	// 添付ファイル ID と BFF オリジンを用意する
	attachmentID := "test-attach-001"
	// BFF オリジン
	bffOrigin := "https://bff.example.com"
	// sandbox URL を生成する
	url := CreateSandboxURL(attachmentID, bffOrigin)
	// 期待値: "https://bff.example.com/attachments/test-attach-001/view"
	expected := "https://bff.example.com/attachments/test-attach-001/view"
	// URL が期待値と一致することを確認する
	if url != expected {
		// 不一致は異常
		t.Errorf("CreateSandboxURL: got %q, want %q", url, expected)
	}
}
