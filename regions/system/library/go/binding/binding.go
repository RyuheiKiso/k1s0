// binding パッケージは OutputBinding/InputBinding インターフェースと複数のバックエンド実装を提供する。
// InMemory（テスト用）、HTTP、ファイルストレージの3種類の OutputBinding をサポートする。
// building-blocks パッケージから移植した独立モジュール版。
package binding

import (
	"bytes"
	"context"
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"strings"
	"sync"
	"time"
)

// ComponentStatus はコンポーネントの現在の状態を表す文字列型。
type ComponentStatus string

// コンポーネントの状態を表す定数群。
const (
	// StatusUninitialized は初期化前の状態を示す。
	StatusUninitialized ComponentStatus = "uninitialized"
	// StatusReady は正常に動作可能な状態を示す。
	StatusReady ComponentStatus = "ready"
	// StatusDegraded は一部機能が低下している状態を示す。
	StatusDegraded ComponentStatus = "degraded"
	// StatusClosed はシャットダウン済みの状態を示す。
	StatusClosed ComponentStatus = "closed"
	// StatusError はエラーが発生している状態を示す。
	StatusError ComponentStatus = "error"
)

// Metadata はコンポーネントのメタデータを保持する構造体。
type Metadata struct {
	// Name はコンポーネント名。
	Name string `json:"name"`
	// Version はコンポーネントのバージョン。
	Version string `json:"version"`
	// Tags は追加タグ情報のマップ（省略可）。
	Tags map[string]string `json:"tags,omitempty"`
}

// Component はすべてのビルディングブロックコンポーネントの基底インターフェース。
type Component interface {
	// Name はコンポーネント識別子を返す。
	Name() string
	// Version はコンポーネントのバージョン文字列を返す。
	Version() string
	// Init はコンポーネントを初期化する。
	Init(ctx context.Context, metadata Metadata) error
	// Close はコンポーネントを終了する。
	Close(ctx context.Context) error
	// Status はコンポーネントの現在のステータスを返す。
	Status(ctx context.Context) ComponentStatus
}

// ComponentError はビルディングブロック操作からのエラーを表す構造体。
type ComponentError struct {
	// Component はエラーが発生したコンポーネント名。
	Component string
	// Operation はエラーが発生した操作名。
	Operation string
	// Message はエラーの説明メッセージ。
	Message string
	// Err は元となるエラー（省略可）。
	Err error
}

// Error はエラーメッセージを文字列として返す。
func (e *ComponentError) Error() string {
	if e.Err != nil {
		return fmt.Sprintf("[%s] %s: %s: %v", e.Component, e.Operation, e.Message, e.Err)
	}
	return fmt.Sprintf("[%s] %s: %s", e.Component, e.Operation, e.Message)
}

// Unwrap は元となるエラーを返す（errors.Is/As のための実装）。
func (e *ComponentError) Unwrap() error {
	return e.Err
}

// NewComponentError は新しい ComponentError を生成して返す。
func NewComponentError(component, operation, message string, err error) *ComponentError {
	return &ComponentError{Component: component, Operation: operation, Message: message, Err: err}
}

// BindingData は入力バインディングからのデータを表す構造体。
type BindingData struct {
	// Data はバイト列形式のペイロード。
	Data []byte
	// Metadata はバインディングデータに付随するメタデータ。
	Metadata map[string]string
}

// BindingResponse は出力バインディング呼び出しのレスポンスを表す構造体。
type BindingResponse struct {
	// Data はレスポンスのバイト列形式のペイロード。
	Data []byte
	// Metadata はレスポンスに付随するメタデータ。
	Metadata map[string]string
}

// InputBinding は外部リソースからデータを読み取るインターフェース。
type InputBinding interface {
	Component
	// Read は外部リソースからデータを読み取る。
	Read(ctx context.Context) (*BindingData, error)
}

// OutputBinding は外部リソースへの操作を実行するインターフェース。
type OutputBinding interface {
	Component
	// Invoke は指定したオペレーションを外部リソースに対して実行する。
	Invoke(ctx context.Context, operation string, data []byte, metadata map[string]string) (*BindingResponse, error)
}

// コンパイル時にインターフェース準拠を検証する。
var _ OutputBinding = (*InMemoryOutputBinding)(nil)
var _ OutputBinding = (*HTTPOutputBinding)(nil)
var _ OutputBinding = (*FileOutputBinding)(nil)

// ─────────────────────────────────────────
// InMemoryOutputBinding
// ─────────────────────────────────────────

// BindingInvocation は InMemoryOutputBinding.Invoke の1回の呼び出し記録を保持する構造体。
type BindingInvocation struct {
	// Operation は呼び出し時のオペレーション名。
	Operation string
	// Data は呼び出し時のデータ。
	Data []byte
	// Metadata は呼び出し時のメタデータ。
	Metadata map[string]string
}

// InMemoryOutputBinding はテスト用の OutputBinding 実装で、呼び出し記録を保持する。
// モックレスポンスやエラーを事前設定することでテストシナリオを制御できる。
type InMemoryOutputBinding struct {
	// mu はフィールドへの並行アクセスを保護するミューテックス。
	mu sync.Mutex
	// last は最後に記録された呼び出し情報（まだ呼ばれていない場合は nil）。
	last *BindingInvocation
	// mockResponse は Invoke が返すモックレスポンス。
	mockResponse *BindingResponse
	// mockErr は Invoke が返すモックエラー。
	mockErr error
	// status はコンポーネントの現在の状態。
	status ComponentStatus
}

// NewInMemoryOutputBinding は新しい InMemoryOutputBinding を生成して返す。
func NewInMemoryOutputBinding() *InMemoryOutputBinding {
	return &InMemoryOutputBinding{status: StatusUninitialized}
}

// Name はコンポーネント識別子を返す。
func (b *InMemoryOutputBinding) Name() string { return "inmemory-binding" }

// Version はコンポーネントのバージョン文字列を返す。
func (b *InMemoryOutputBinding) Version() string { return "1.0.0" }

// Init はコンポーネントを初期化し、ステータスを Ready に遷移させる。
func (b *InMemoryOutputBinding) Init(_ context.Context, _ Metadata) error {
	b.mu.Lock()
	defer b.mu.Unlock()
	b.status = StatusReady
	return nil
}

// Close はコンポーネントを終了し、ステータスを Closed に遷移させる。
func (b *InMemoryOutputBinding) Close(_ context.Context) error {
	b.mu.Lock()
	defer b.mu.Unlock()
	b.status = StatusClosed
	return nil
}

// Status はコンポーネントの現在のステータスを返す。
func (b *InMemoryOutputBinding) Status(_ context.Context) ComponentStatus {
	b.mu.Lock()
	defer b.mu.Unlock()
	return b.status
}

// Invoke は呼び出しを記録し、設定されたモックレスポンスを返す。
// モックエラーが設定されている場合はそのエラーを返す。
func (b *InMemoryOutputBinding) Invoke(_ context.Context, operation string, data []byte, metadata map[string]string) (*BindingResponse, error) {
	b.mu.Lock()
	defer b.mu.Unlock()
	// 呼び出し情報を記録する。
	b.last = &BindingInvocation{Operation: operation, Data: data, Metadata: metadata}
	if b.mockErr != nil {
		return nil, b.mockErr
	}
	if b.mockResponse != nil {
		return b.mockResponse, nil
	}
	return &BindingResponse{}, nil
}

// LastInvocation は最後に記録された呼び出し情報を返す（まだ呼ばれていない場合は nil）。
func (b *InMemoryOutputBinding) LastInvocation() *BindingInvocation {
	b.mu.Lock()
	defer b.mu.Unlock()
	return b.last
}

// SetResponse は Invoke が返すモックレスポンスとエラーを設定する。
func (b *InMemoryOutputBinding) SetResponse(resp *BindingResponse, err error) {
	b.mu.Lock()
	defer b.mu.Unlock()
	b.mockResponse = resp
	b.mockErr = err
}

// Reset は記録された呼び出し情報とモック設定をすべてクリアする。
func (b *InMemoryOutputBinding) Reset() {
	b.mu.Lock()
	defer b.mu.Unlock()
	b.last = nil
	b.mockResponse = nil
	b.mockErr = nil
}

// ─────────────────────────────────────────
// HTTPOutputBinding
// ─────────────────────────────────────────

// HTTPOutputBinding はHTTPリクエストを OutputBinding として提供するアダプター。
// サポートするオペレーション（HTTPメソッド）: GET, POST, PUT, DELETE, PATCH。
// 必須メタデータ:
//   - "url": リクエスト先のURL
//
// 任意メタデータ:
//   - "content-type": リクエストのContent-Type（データが空でない場合はデフォルト "application/octet-stream"）
//   - その他のキーはリクエストヘッダーとして転送される
//
// レスポンスのメタデータには "status-code" とレスポンスヘッダーが含まれる。
type HTTPOutputBinding struct {
	// mu は status フィールドへの並行アクセスを保護するミューテックス。
	mu sync.Mutex
	// client はHTTPリクエストを実行するクライアント。
	client *http.Client
	// status はコンポーネントの現在の状態を表す。
	status ComponentStatus
}

// NewHTTPOutputBinding は新しい HTTPOutputBinding を生成して返す。
// client が nil の場合はデフォルトの http.Client を使用する。
func NewHTTPOutputBinding(client *http.Client) *HTTPOutputBinding {
	if client == nil {
		client = &http.Client{}
	}
	return &HTTPOutputBinding{
		client: client,
		status: StatusUninitialized,
	}
}

// Name はコンポーネント識別子を返す。
func (b *HTTPOutputBinding) Name() string { return "http-binding" }

// Version はコンポーネントのバージョン文字列を返す。
func (b *HTTPOutputBinding) Version() string { return "1.0.0" }

// Init はコンポーネントを初期化し、ステータスを Ready に遷移させる。
func (b *HTTPOutputBinding) Init(_ context.Context, _ Metadata) error {
	b.mu.Lock()
	defer b.mu.Unlock()
	b.status = StatusReady
	return nil
}

// Close はコンポーネントを終了し、ステータスを Closed に遷移させる。
func (b *HTTPOutputBinding) Close(_ context.Context) error {
	b.mu.Lock()
	defer b.mu.Unlock()
	b.status = StatusClosed
	return nil
}

// Status はコンポーネントの現在のステータスを返す。
func (b *HTTPOutputBinding) Status(_ context.Context) ComponentStatus {
	b.mu.Lock()
	defer b.mu.Unlock()
	return b.status
}

// Invoke はHTTPリクエストを実行する。
// operation には有効なHTTPメソッド（GET, POST, PUT, DELETE, PATCH）を指定する。
// metadata["url"] は必須。HTTPステータスコードが400以上の場合はエラーを返す。
func (b *HTTPOutputBinding) Invoke(ctx context.Context, operation string, data []byte, metadata map[string]string) (*BindingResponse, error) {
	// リクエスト先URLはメタデータから取得する（必須項目）。
	targetURL, ok := metadata["url"]
	if !ok || targetURL == "" {
		return nil, NewComponentError("http-binding", "Invoke", `metadata["url"] is required`, nil)
	}

	// データが存在する場合はリクエストボディとして設定する。
	var body io.Reader
	if len(data) > 0 {
		body = bytes.NewReader(data)
	}

	req, err := http.NewRequestWithContext(ctx, operation, targetURL, body)
	if err != nil {
		return nil, NewComponentError("http-binding", "Invoke",
			fmt.Sprintf("failed to create request: %s", err), err)
	}

	// metadata["content-type"] が指定されていればヘッダーに設定し、
	// 未指定かつデータが存在する場合はバイナリとして扱う。
	if ct, ok := metadata["content-type"]; ok {
		req.Header.Set("Content-Type", ct)
	} else if len(data) > 0 {
		req.Header.Set("Content-Type", "application/octet-stream")
	}

	// "url" と "content-type" 以外のメタデータはリクエストヘッダーとして転送する。
	for k, v := range metadata {
		if k == "url" || k == "content-type" {
			continue
		}
		req.Header.Set(k, v)
	}

	resp, err := b.client.Do(req)
	if err != nil {
		return nil, NewComponentError("http-binding", "Invoke",
			fmt.Sprintf("HTTP request failed: %s", err), err)
	}
	defer resp.Body.Close()

	respData, err := io.ReadAll(resp.Body)
	if err != nil {
		return nil, NewComponentError("http-binding", "Invoke",
			fmt.Sprintf("failed to read response: %s", err), err)
	}

	// 4xx/5xx のレスポンスはエラーとして扱い、レスポンスボディを含めて返す。
	if resp.StatusCode >= 400 {
		return nil, NewComponentError("http-binding", "Invoke",
			fmt.Sprintf("HTTP %d: %s", resp.StatusCode, string(respData)), nil)
	}

	// レスポンスのステータスコードとヘッダーをメタデータとして格納する。
	respMeta := make(map[string]string, len(resp.Header)+1)
	respMeta["status-code"] = fmt.Sprintf("%d", resp.StatusCode)
	for k := range resp.Header {
		respMeta[strings.ToLower(k)] = resp.Header.Get(k)
	}

	return &BindingResponse{Data: respData, Metadata: respMeta}, nil
}

// ─────────────────────────────────────────
// FileOutputBinding
// ─────────────────────────────────────────

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
// *fileclient.ServerFileClient または *fileclient.S3FileClient を注入することで満たせる。
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

// FileOutputBinding はS3互換ファイルストレージ操作を OutputBinding として提供するアダプター。
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
		data, _ := json.Marshal(info)
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
		data, _ := json.Marshal(info)
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
		data, _ := json.Marshal(files)
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
