package buildingblocks

import (
	"bytes"
	"context"
	"fmt"
	"io"
	"net/http"
	"strings"
	"sync"
	"time"
)

// コンパイル時にインターフェース準拠を検証する。
var _ OutputBinding = (*HTTPOutputBinding)(nil)

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
		// タイムアウト未設定の場合にデフォルト30秒を設定し、無限待ちを防止する
		client = &http.Client{Timeout: 30 * time.Second}
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
