package bbaiclient

import "context"

// MockClient はテスト用の AiClient モック実装。
// 各メソッドにカスタムハンドラを設定できる。
type MockClient struct {
	CompleteFn   func(ctx context.Context, req CompleteRequest) (*CompleteResponse, error)
	EmbedFn      func(ctx context.Context, req EmbedRequest) (*EmbedResponse, error)
	ListModelsFn func(ctx context.Context) ([]ModelInfo, error)
}

// Complete は CompleteFn が設定されていればそれを呼び出す。
func (m *MockClient) Complete(ctx context.Context, req CompleteRequest) (*CompleteResponse, error) {
	if m.CompleteFn != nil {
		return m.CompleteFn(ctx, req)
	}
	return &CompleteResponse{ID: "mock", Model: req.Model, Content: "mock response"}, nil
}

// Embed は EmbedFn が設定されていればそれを呼び出す。
func (m *MockClient) Embed(ctx context.Context, req EmbedRequest) (*EmbedResponse, error) {
	if m.EmbedFn != nil {
		return m.EmbedFn(ctx, req)
	}
	return &EmbedResponse{Model: req.Model, Embeddings: [][]float64{{0.1, 0.2}}}, nil
}

// ListModels は ListModelsFn が設定されていればそれを呼び出す。
func (m *MockClient) ListModels(ctx context.Context) ([]ModelInfo, error) {
	if m.ListModelsFn != nil {
		return m.ListModelsFn(ctx)
	}
	return []ModelInfo{{ID: "mock-model", Name: "Mock Model"}}, nil
}
