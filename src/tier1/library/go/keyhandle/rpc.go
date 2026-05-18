// rpc.go — k1s0 tier1 Library Go 実装: RPC / Gateway の L3 interface
// 10_RPC適合仕様.md §RpcClient / §GatewayHandler（OSS 中立 L3）に準拠する。
// gRPC / ConnectRPC / HTTP/2 等 OSS の API を一切露出しない Wire protocol 抽象 interface を宣言する。
// 公開シグネチャに OSS 型（grpc.ClientConn / connect.Client 等）を一切含まない。

// パッケージ名: keyhandle（tier1 Library の RPC / Gateway API を提供する）
package keyhandle

import (
	// context: context.Context（非同期操作 / キャンセル制御に使用する）
	"context"
)

// RpcMetadata は RPC 呼び出しのメタデータ（ヘッダー相当）を宣言する型。
// OSS の metadata.MD / http.Header を露出せず Library 独自語彙で表現する。
type RpcMetadata map[string][]string

// RpcStatusCode は RPC 呼び出しのステータスコードを宣言する型。
// gRPC Status Code に準拠した Library 独自語彙とする。
type RpcStatusCode int

const (
	// RpcOK: 成功（gRPC OK = 0）
	RpcOK RpcStatusCode = 0
	// RpcCanceled: キャンセル（gRPC Canceled = 1）
	RpcCanceled RpcStatusCode = 1
	// RpcUnknown: 不明なエラー（gRPC Unknown = 2）
	RpcUnknown RpcStatusCode = 2
	// RpcInvalidArgument: 無効な引数（gRPC InvalidArgument = 3）
	RpcInvalidArgument RpcStatusCode = 3
	// RpcDeadlineExceeded: デッドライン超過（gRPC DeadlineExceeded = 4）
	RpcDeadlineExceeded RpcStatusCode = 4
	// RpcNotFound: リソース未発見（gRPC NotFound = 5）
	RpcNotFound RpcStatusCode = 5
	// RpcAlreadyExists: リソース重複（gRPC AlreadyExists = 6）
	RpcAlreadyExists RpcStatusCode = 6
	// RpcPermissionDenied: 権限エラー（gRPC PermissionDenied = 7）
	RpcPermissionDenied RpcStatusCode = 7
	// RpcUnauthenticated: 認証エラー（gRPC Unauthenticated = 16）
	RpcUnauthenticated RpcStatusCode = 16
	// RpcResourceExhausted: リソース枯渇（gRPC ResourceExhausted = 8）
	RpcResourceExhausted RpcStatusCode = 8
	// RpcFailedPrecondition: 前提条件不満（gRPC FailedPrecondition = 9）
	RpcFailedPrecondition RpcStatusCode = 9
	// RpcAborted: 中断（gRPC Aborted = 10）
	RpcAborted RpcStatusCode = 10
	// RpcInternal: 内部エラー（gRPC Internal = 13）
	RpcInternal RpcStatusCode = 13
	// RpcUnavailable: サービス不可（gRPC Unavailable = 14）
	RpcUnavailable RpcStatusCode = 14
)

// RpcError は RPC 呼び出しエラーを宣言する型。
// OSS の status.Error / connect.Error を露出せず Library 独自語彙で表現する。
type RpcError struct {
	// Code: gRPC Status Code 準拠のステータスコード
	Code RpcStatusCode
	// Message: エラーメッセージ
	Message string
}

// Error は error interface を実装する（RpcError をエラーとして使用可能にする）。
func (e *RpcError) Error() string {
	// エラーメッセージを整形して返す
	return e.Message
}

// RpcCallOptions は Unary / Streaming RPC 呼び出しに渡すオプションを宣言する型。
type RpcCallOptions struct {
	// Metadata: 送信するリクエストメタデータ（ヘッダー相当）
	Metadata RpcMetadata
	// AuthContext: RPC 呼び出しに付与する認証コンテキスト（tenant 分離に必須）
	// nil の場合は ctx から AuthContext を自動抽出する実装を推奨する。
	AuthContext *AuthContext
	// TimeoutMs: タイムアウトミリ秒（0 = ctx のデッドラインに従う）
	TimeoutMs int64
}

// RpcUnaryClient は Unary RPC の L3 抽象 interface を宣言する。
// 全 OSS の transport を透過的に切り替え可能にする。
// OSS 型（grpc.ClientConn / connect.Client 等）を引数・戻り値に一切含まない。
type RpcUnaryClient interface {
	// Call は Unary RPC を呼び出す。
	// service はフルサービス名（"k1s0.tier1.KeySvc" 等）。
	// method は RPC メソッド名（"Sign" 等）。
	// req は リクエストペイロード（protobuf メッセージの JSON バイト列）。
	// 戻り値はレスポンスペイロード（protobuf メッセージの JSON バイト列）。
	Call(ctx context.Context, service, method string, req []byte, opts *RpcCallOptions) ([]byte, RpcMetadata, error)
}

// RpcStreamClient は Streaming RPC の L3 抽象 interface を宣言する。
// Server / Client / Bidirectional streaming を統一 interface で表現する。
type RpcStreamClient interface {
	// Send はストリームにメッセージを送信する（Client streaming / Bidirectional 用）。
	Send(ctx context.Context, payload []byte) error

	// Recv はストリームからメッセージを受信する（Server streaming / Bidirectional 用）。
	// ストリーム終了時は io.EOF に相当するエラーを返す。
	Recv(ctx context.Context) ([]byte, error)

	// CloseSend はクライアント側の送信を完了する（Half-close）。
	CloseSend() error

	// Metadata は受信したレスポンスメタデータ（トレーラー等）を返す。
	Metadata() RpcMetadata
}

// RpcStreamingClient は Streaming RPC セッションを開始する L3 interface を宣言する。
type RpcStreamingClient interface {
	// OpenStream は Streaming RPC セッションを開始して RpcStreamClient を返す。
	// service / method は Call と同様のフルサービス名・RPC メソッド名。
	// opts は RpcCallOptions（メタデータ・タイムアウト設定）。
	OpenStream(ctx context.Context, service, method string, opts *RpcCallOptions) (RpcStreamClient, error)
}

// GatewayRequest は Gateway 経由の HTTP/gRPC リクエストを宣言する型。
// OSS の http.Request を露出せず Library 独自語彙で表現する。
type GatewayRequest struct {
	// Method: HTTP メソッド（"GET" / "POST" 等）
	Method string
	// Path: リクエストパス（"/v1/keys/sign" 等）
	Path string
	// Headers: リクエストヘッダー（OSS の http.Header を直接使わない）
	Headers RpcMetadata
	// Body: リクエストボディバイト列
	Body []byte
	// AuthContext: Gateway が検証済み認証コンテキスト（tenant 分離必須）
	AuthContext *AuthContext
}

// GatewayResponse は Gateway から返す HTTP レスポンスを宣言する型。
// OSS の http.ResponseWriter を露出せず Library 独自語彙で表現する。
type GatewayResponse struct {
	// StatusCode: HTTP ステータスコード
	StatusCode int
	// Headers: レスポンスヘッダー
	Headers RpcMetadata
	// Body: レスポンスボディバイト列
	Body []byte
}

// GatewayHandler は Gateway のリクエストハンドラー interface を宣言する。
// envoy / nginx / Kubernetes Ingress 等の Gateway proxy を Library 独自語彙で抽象化する。
type GatewayHandler interface {
	// Handle は GatewayRequest を受け取って GatewayResponse を返す。
	// ctx には AuthContext が伝播されている前提とする（tenant 分離必須）。
	Handle(ctx context.Context, req *GatewayRequest) (*GatewayResponse, error)
}

// GatewayMiddleware は Gateway ミドルウェアを宣言する型。
// GatewayHandler をラップして前処理・後処理を挿入する。
type GatewayMiddleware func(next GatewayHandler) GatewayHandler
