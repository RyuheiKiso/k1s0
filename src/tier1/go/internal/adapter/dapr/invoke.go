// 本ファイルは Dapr Service Invocation building block のアダプタ。
//
// 設計正典:
//   docs/04_概要設計/20_ソフトウェア方式設計/01_コンポーネント方式設計/02_Daprファサード層コンポーネント.md
//     - Service Invoke API → Dapr Service Invocation
//   docs/03_要件定義/20_機能要件/40_tier1_API契約IDL/01_Service_Invoke_API.md
//
// 役割（plan 04-11 結線済）:
//   handler.go が呼び出す Service Invocation を Dapr SDK の InvokeMethodWithCustomContent
//   で実行する。verb は POST 固定（gRPC ファサード経由の Dapr Service Invocation 慣用）。
//   タイムアウト指定は context の Deadline で表現するため、TimeoutMs は handler 側で
//   context.WithTimeout を作る前提（adapter は context をそのまま SDK に渡す）。

package dapr

import (
	// 全 RPC で context を伝搬する。
	"context"
)

// InvokeRequest は ServiceInvocation 呼出の入力。
type InvokeRequest struct {
	// 呼出先アプリ識別子（Dapr app_id）。
	AppID string
	// メソッド名（HTTP では path 相当）。
	Method string
	// 呼出データ。
	Data []byte
	// Content-Type。
	ContentType string
	// テナント識別子（adapter 上は metadata 利用なし、現状 SDK の InvokeMethod 系は
	// metadata を直接受けないため、テナント伝搬は handler 側で context value 経由とする）。
	TenantID string
	// タイムアウト（ミリ秒、0 で 5000ms 既定）。handler 側で context.WithTimeout を作る前提。
	TimeoutMs int32
}

// InvokeResponse は ServiceInvocation 応答。
type InvokeResponse struct {
	// 応答本文。
	Data []byte
	// Content-Type。SDK の InvokeMethod 系は応答 ContentType を返さないため、
	// 呼び出し側 ContentType を echo で返す（厳密に正しくはないが proto 契約を満たすため）。
	ContentType string
	// HTTP ステータス相当。SDK は応答ステータスを exposing しないため
	// 成功時は 200、エラー時は handler 側で gRPC status code に変換される。
	Status int32
}

// InvokeAdapter は ServiceInvocation 操作の interface。
type InvokeAdapter interface {
	// 任意サービスの任意メソッド呼出。
	Invoke(ctx context.Context, req InvokeRequest) (InvokeResponse, error)
}

// daprInvokeAdapter は Client（narrow interface）越しに SDK を呼ぶ実装。
type daprInvokeAdapter struct {
	client *Client
}

// NewInvokeAdapter は InvokeAdapter を生成する。
func NewInvokeAdapter(client *Client) InvokeAdapter {
	return &daprInvokeAdapter{client: client}
}

// Invoke は他サービスの任意メソッドを呼び出す。
// SDK の InvokeMethodWithCustomContent は verb と contentType と content（任意型）を取る。
// k1s0 では gRPC ファサード越しの呼び出しを想定し verb は POST 固定とする。
func (a *daprInvokeAdapter) Invoke(ctx context.Context, req InvokeRequest) (InvokeResponse, error) {
	// content-type が空ならデフォルト値（octet-stream）を使う。
	ct := req.ContentType
	if ct == "" {
		ct = "application/octet-stream"
	}
	// SDK 呼び出し（verb は POST 固定、Dapr ファサード経由の慣用）。
	out, err := a.client.invokeClient().InvokeMethodWithCustomContent(ctx, req.AppID, req.Method, "POST", ct, req.Data)
	if err != nil {
		return InvokeResponse{}, err
	}
	// 応答 ContentType / Status は SDK が exposing しないため echo / 200 を返す。
	return InvokeResponse{
		Data:        out,
		ContentType: ct,
		Status:      200,
	}, nil
}
