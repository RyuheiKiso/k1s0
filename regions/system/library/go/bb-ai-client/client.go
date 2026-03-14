package bbaiclient

import "context"

// AiClient は AI ゲートウェイへのアクセスを抽象化するインターフェース。
// テスト時はモック実装に差し替えられる。
type AiClient interface {
	// Complete はチャット補完を実行する。
	Complete(ctx context.Context, req CompleteRequest) (*CompleteResponse, error)
	// Embed はテキスト埋め込みを生成する。
	Embed(ctx context.Context, req EmbedRequest) (*EmbedResponse, error)
	// ListModels は利用可能なモデル一覧を返す。
	ListModels(ctx context.Context) ([]ModelInfo, error)
}
