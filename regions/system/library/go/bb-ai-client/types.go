package bbaiclient

// ChatMessage は AI との会話メッセージを表す。
// role は "user" または "assistant" または "system" のいずれか。
type ChatMessage struct {
	Role    string `json:"role"`
	Content string `json:"content"`
}

// CompleteRequest はテキスト補完リクエストのパラメータを保持する。
type CompleteRequest struct {
	Model       string        `json:"model"`
	Messages    []ChatMessage `json:"messages"`
	MaxTokens   int           `json:"max_tokens,omitempty"`
	Temperature float64       `json:"temperature,omitempty"`
	Stream      bool          `json:"stream,omitempty"`
}

// CompleteResponse はテキスト補完レスポンスを保持する。
type CompleteResponse struct {
	ID      string `json:"id"`
	Model   string `json:"model"`
	Content string `json:"content"`
	Usage   Usage  `json:"usage"`
}

// Usage はトークン使用量を保持する。
type Usage struct {
	InputTokens  int `json:"input_tokens"`
	OutputTokens int `json:"output_tokens"`
}

// EmbedRequest はテキスト埋め込みリクエストのパラメータを保持する。
type EmbedRequest struct {
	Model string   `json:"model"`
	Texts []string `json:"texts"`
}

// EmbedResponse はテキスト埋め込みレスポンスを保持する。
type EmbedResponse struct {
	Model      string      `json:"model"`
	Embeddings [][]float64 `json:"embeddings"`
}

// ModelInfo はモデルの基本情報を保持する。
type ModelInfo struct {
	ID          string `json:"id"`
	Name        string `json:"name"`
	Description string `json:"description"`
}
