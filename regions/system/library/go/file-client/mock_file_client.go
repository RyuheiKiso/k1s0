package fileclient

import (
	"context"
	"fmt"
	"sync"
	"time"
)

// MockCall は MockFileClient に記録された呼び出し情報。
type MockCall struct {
	// Method はメソッド名。
	Method string
	// Args は呼び出し引数。
	Args []any
}

// mockReturn は特定のメソッド・引数に対するスタブ応答。
type mockReturn struct {
	values []any
}

// MockFileClient は FileClient インターフェースを実装した録再生可能なモック。
// テストコードでの依存注入・呼び出し検証・スタブ応答注入に使用する。
type MockFileClient struct {
	mu      sync.Mutex
	calls   []MockCall
	returns map[string]*mockReturn
}

// NewMockFileClient は新しい MockFileClient を生成する。
func NewMockFileClient() *MockFileClient {
	return &MockFileClient{
		returns: make(map[string]*mockReturn),
	}
}

// On はメソッド名に対するスタブ応答を登録する。
// values には戻り値を順番に指定する（最後の値がエラーの場合は error 型を渡す）。
//
// 例:
//
//	mock.On("GetMetadata", &FileMetadata{Path: "uploads/a.png"}, nil)
//	mock.On("Delete", nil)                   // error なし
//	mock.On("Delete", fmt.Errorf("not found")) // エラーを返す
func (m *MockFileClient) On(method string, values ...any) *MockFileClient {
	m.mu.Lock()
	defer m.mu.Unlock()
	m.returns[method] = &mockReturn{values: values}
	return m
}

// Calls は記録された全呼び出しを返す。
func (m *MockFileClient) Calls() []MockCall {
	m.mu.Lock()
	defer m.mu.Unlock()
	result := make([]MockCall, len(m.calls))
	copy(result, m.calls)
	return result
}

// AssertCalled はメソッドが指定の引数で呼び出されたかを検証する。
// 呼び出されていない場合は panic する（テスト用ヘルパー）。
func (m *MockFileClient) AssertCalled(t interface{ Errorf(string, ...any) }, method string, args ...any) {
	m.mu.Lock()
	defer m.mu.Unlock()
	for _, c := range m.calls {
		if c.Method != method {
			continue
		}
		if len(args) == 0 {
			return
		}
		if fmt.Sprintf("%v", c.Args) == fmt.Sprintf("%v", args) {
			return
		}
	}
	t.Errorf("MockFileClient: expected call to %s(%v) but not found in calls: %v", method, args, m.calls)
}

func (m *MockFileClient) record(method string, args ...any) {
	m.mu.Lock()
	defer m.mu.Unlock()
	m.calls = append(m.calls, MockCall{Method: method, Args: args})
}

func (m *MockFileClient) getReturn(method string) []any {
	m.mu.Lock()
	defer m.mu.Unlock()
	if r, ok := m.returns[method]; ok {
		return r.values
	}
	return nil
}

func errFrom(values []any) error {
	if len(values) == 0 {
		return nil
	}
	last := values[len(values)-1]
	if last == nil {
		return nil
	}
	if err, ok := last.(error); ok {
		return err
	}
	return nil
}

// GenerateUploadURL は FileClient インターフェースの実装。
func (m *MockFileClient) GenerateUploadURL(_ context.Context, path, contentType string, expiresIn time.Duration) (*PresignedURL, error) {
	m.record("GenerateUploadURL", path, contentType, expiresIn)
	ret := m.getReturn("GenerateUploadURL")
	if err := errFrom(ret); err != nil {
		return nil, err
	}
	if len(ret) > 0 {
		if v, ok := ret[0].(*PresignedURL); ok {
			return v, nil
		}
	}
	return &PresignedURL{
		URL:       "https://mock.example.com/upload/" + path,
		Method:    "PUT",
		ExpiresAt: time.Now().Add(expiresIn),
		Headers:   make(map[string]string),
	}, nil
}

// GenerateDownloadURL は FileClient インターフェースの実装。
func (m *MockFileClient) GenerateDownloadURL(_ context.Context, path string, expiresIn time.Duration) (*PresignedURL, error) {
	m.record("GenerateDownloadURL", path, expiresIn)
	ret := m.getReturn("GenerateDownloadURL")
	if err := errFrom(ret); err != nil {
		return nil, err
	}
	if len(ret) > 0 {
		if v, ok := ret[0].(*PresignedURL); ok {
			return v, nil
		}
	}
	return &PresignedURL{
		URL:       "https://mock.example.com/download/" + path,
		Method:    "GET",
		ExpiresAt: time.Now().Add(expiresIn),
		Headers:   make(map[string]string),
	}, nil
}

// Delete は FileClient インターフェースの実装。
func (m *MockFileClient) Delete(_ context.Context, path string) error {
	m.record("Delete", path)
	return errFrom(m.getReturn("Delete"))
}

// GetMetadata は FileClient インターフェースの実装。
func (m *MockFileClient) GetMetadata(_ context.Context, path string) (*FileMetadata, error) {
	m.record("GetMetadata", path)
	ret := m.getReturn("GetMetadata")
	if err := errFrom(ret); err != nil {
		return nil, err
	}
	if len(ret) > 0 {
		if v, ok := ret[0].(*FileMetadata); ok {
			return v, nil
		}
	}
	return &FileMetadata{Path: path}, nil
}

// List は FileClient インターフェースの実装。
func (m *MockFileClient) List(_ context.Context, prefix string) ([]*FileMetadata, error) {
	m.record("List", prefix)
	ret := m.getReturn("List")
	if err := errFrom(ret); err != nil {
		return nil, err
	}
	if len(ret) > 0 {
		if v, ok := ret[0].([]*FileMetadata); ok {
			return v, nil
		}
	}
	return []*FileMetadata{}, nil
}

// Copy は FileClient インターフェースの実装。
func (m *MockFileClient) Copy(_ context.Context, src, dst string) error {
	m.record("Copy", src, dst)
	return errFrom(m.getReturn("Copy"))
}
