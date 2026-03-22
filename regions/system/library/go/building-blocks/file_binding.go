package buildingblocks

import (
	"context"
	"encoding/json"
	"fmt"
	"sync"
	"time"
)

// FilePresignedURL はk1s0-file-client の PresignedURL と互換性を持つ署名済みURL構造体。
// ファイルのアップロード/ダウンロードに使用する一時的なURLと付随情報を保持する。
type FilePresignedURL struct {
	// URL は署名済みのアクセスURL。
	URL string `json:"url"`
	// Method はHTTPメソッド（例: "PUT", "GET"）。
	Method string `json:"method"`
	// ExpiresAt はURLの有効期限。
	ExpiresAt time.Time `json:"expires_at"`
	// Headers はリクエスト時に付与すべき追加ヘッダーのマップ。
	Headers map[string]string `json:"headers"`
}

// FileInfo はk1s0-file-client の FileMetadata と互換性を持つファイル情報構造体。
// ストレージ上のファイルのメタデータを表す。
type FileInfo struct {
	// Path はストレージ上のファイルパス。
	Path string `json:"path"`
	// SizeBytes はファイルサイズ（バイト単位）。
	SizeBytes int64 `json:"size_bytes"`
	// ContentType はファイルのMIMEタイプ。
	ContentType string `json:"content_type"`
}

// FileClientIface はk1s0-file-client の FileClient と互換性を持つインターフェース。
// *fileclient.ServerFileClient を注入することで満たせる。
type FileClientIface interface {
	// GenerateUploadURL はアップロード用の署名済みURLを生成する。
	GenerateUploadURL(ctx context.Context, path, contentType string, expiresIn time.Duration) (*FilePresignedURL, error)
	// GenerateDownloadURL はダウンロード用の署名済みURLを生成する。
	GenerateDownloadURL(ctx context.Context, path string, expiresIn time.Duration) (*FilePresignedURL, error)
	// Delete は指定パスのファイルを削除する。
	Delete(ctx context.Context, path string) error
	// List は指定プレフィックス以下のファイル一覧を返す。
	List(ctx context.Context, prefix string) ([]*FileInfo, error)
	// Copy はファイルをsrcからdstへコピーする。
	Copy(ctx context.Context, src, dst string) error
}

// コンパイル時にインターフェース準拠を検証する。
var _ OutputBinding = (*FileOutputBinding)(nil)

// FileOutputBinding はfile-server経由のファイル操作を OutputBinding として提供するアダプター。
// FileClientIface をラップし、以下のオペレーションをサポートする:
//   - "upload-url":   アップロード用署名済みURLを生成（metadata["path"] 必須）
//   - "download-url": ダウンロード用署名済みURLを生成（metadata["path"] 必須）
//   - "delete":       ファイルを削除（metadata["path"] 必須）
//   - "list":         ファイル一覧を取得（metadata["prefix"] 省略可、デフォルトは ""）
//   - "copy":         ファイルをコピー（metadata["src"] と metadata["dst"] 必須）
type FileOutputBinding struct {
	// mu は status フィールドへの並行アクセスを保護するミューテックス。
	mu sync.Mutex
	// name はコンポーネントの識別子。
	name string
	// client はファイルストレージ操作を実行する実装クライアント。
	client FileClientIface
	// status はコンポーネントの現在の状態を表す。
	status ComponentStatus
}

// NewFileOutputBinding は新しい FileOutputBinding を生成して返す。
// name はコンポーネント識別子、client はファイル操作を担うクライアント実装。
func NewFileOutputBinding(name string, client FileClientIface) *FileOutputBinding {
	return &FileOutputBinding{
		name:   name,
		client: client,
		status: StatusUninitialized,
	}
}

// Name はコンポーネント識別子を返す。
func (b *FileOutputBinding) Name() string { return b.name }

// Version はコンポーネントのバージョン文字列を返す。
func (b *FileOutputBinding) Version() string { return "1.0.0" }

// Init はコンポーネントを初期化し、ステータスを Ready に遷移させる。
func (b *FileOutputBinding) Init(_ context.Context, _ Metadata) error {
	b.mu.Lock()
	defer b.mu.Unlock()
	b.status = StatusReady
	return nil
}

// Close はコンポーネントを終了し、ステータスを Closed に遷移させる。
func (b *FileOutputBinding) Close(_ context.Context) error {
	b.mu.Lock()
	defer b.mu.Unlock()
	b.status = StatusClosed
	return nil
}

// Status はコンポーネントの現在のステータスを返す。
func (b *FileOutputBinding) Status(_ context.Context) ComponentStatus {
	b.mu.Lock()
	defer b.mu.Unlock()
	return b.status
}

// Invoke はファイルストレージ操作を実行する。
// operation には "upload-url", "download-url", "delete", "list", "copy" のいずれかを指定する。
// metadata["url"] は不要だが、各オペレーション固有のキーが必要となる。
func (b *FileOutputBinding) Invoke(ctx context.Context, operation string, _ []byte, metadata map[string]string) (*BindingResponse, error) {
	// 署名済みURLのデフォルト有効期限は15分とする。
	const defaultTTL = 15 * time.Minute

	switch operation {
	case "upload-url":
		// アップロード対象のファイルパスを metadata から取得する。
		path := metadata["path"]
		if path == "" {
			return nil, NewComponentError(b.name, "Invoke", `metadata["path"] is required for upload-url`, nil)
		}
		// Content-Type が未指定の場合はバイナリストリームとして扱う。
		ct := metadata["content-type"]
		if ct == "" {
			ct = "application/octet-stream"
		}
		info, err := b.client.GenerateUploadURL(ctx, path, ct, defaultTTL)
		if err != nil {
			return nil, NewComponentError(b.name, "Invoke", "failed to generate upload URL", err)
		}
		// レスポンス構造体を JSON にシリアライズしてバイト列として返す。
		data, err := json.Marshal(info)
		if err != nil {
			return nil, NewComponentError(b.name, "Invoke", "レスポンスのJSON変換に失敗しました", err)
		}
		return &BindingResponse{Data: data}, nil

	case "download-url":
		// ダウンロード対象のファイルパスを metadata から取得する。
		path := metadata["path"]
		if path == "" {
			return nil, NewComponentError(b.name, "Invoke", `metadata["path"] is required for download-url`, nil)
		}
		info, err := b.client.GenerateDownloadURL(ctx, path, defaultTTL)
		if err != nil {
			return nil, NewComponentError(b.name, "Invoke", "failed to generate download URL", err)
		}
		// レスポンス構造体を JSON にシリアライズしてバイト列として返す。
		data, err := json.Marshal(info)
		if err != nil {
			return nil, NewComponentError(b.name, "Invoke", "レスポンスのJSON変換に失敗しました", err)
		}
		return &BindingResponse{Data: data}, nil

	case "delete":
		// 削除対象のファイルパスを metadata から取得する。
		path := metadata["path"]
		if path == "" {
			return nil, NewComponentError(b.name, "Invoke", `metadata["path"] is required for delete`, nil)
		}
		if err := b.client.Delete(ctx, path); err != nil {
			return nil, NewComponentError(b.name, "Invoke", "failed to delete file", err)
		}
		return &BindingResponse{}, nil

	case "list":
		// prefix が省略された場合はルートディレクトリを対象にする。
		files, err := b.client.List(ctx, metadata["prefix"])
		if err != nil {
			return nil, NewComponentError(b.name, "Invoke", "failed to list files", err)
		}
		// ファイル一覧を JSON にシリアライズしてバイト列として返す。
		data, err := json.Marshal(files)
		if err != nil {
			return nil, NewComponentError(b.name, "Invoke", "レスポンスのJSON変換に失敗しました", err)
		}
		return &BindingResponse{Data: data}, nil

	case "copy":
		// コピー元（src）とコピー先（dst）のパスを metadata から取得する。
		src, dst := metadata["src"], metadata["dst"]
		if src == "" || dst == "" {
			return nil, NewComponentError(b.name, "Invoke",
				`metadata["src"] and metadata["dst"] are required for copy`, nil)
		}
		if err := b.client.Copy(ctx, src, dst); err != nil {
			return nil, NewComponentError(b.name, "Invoke", "failed to copy file", err)
		}
		return &BindingResponse{}, nil

	default:
		// サポート外のオペレーションが指定された場合はエラーを返す。
		return nil, NewComponentError(b.name, "Invoke",
			fmt.Sprintf("unsupported operation %q; supported: upload-url, download-url, delete, list, copy", operation), nil)
	}
}
