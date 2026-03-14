package buildingblocks

import (
	"context"
	"fmt"
	"sync"
)

// ComponentRegistry はビルディングブロックコンポーネントの登録・管理を行うレジストリ。
// スレッドセーフな RWMutex を用いて並行アクセスに対応する。
type ComponentRegistry struct {
	mu         sync.RWMutex
	components map[string]Component
}

// NewComponentRegistry は新しい ComponentRegistry を生成して返す。
func NewComponentRegistry() *ComponentRegistry {
	return &ComponentRegistry{
		components: make(map[string]Component),
	}
}

// Register はコンポーネントをレジストリに登録する。
// 同名のコンポーネントが既に存在する場合はエラーを返す。
func (r *ComponentRegistry) Register(component Component) error {
	r.mu.Lock()
	defer r.mu.Unlock()

	name := component.Name()
	if _, exists := r.components[name]; exists {
		return fmt.Errorf("コンポーネント '%s' は既に登録されています", name)
	}
	r.components[name] = component
	return nil
}

// Get は名前でコンポーネントを取得する。存在しない場合は nil, false を返す。
func (r *ComponentRegistry) Get(name string) (Component, bool) {
	r.mu.RLock()
	defer r.mu.RUnlock()

	c, ok := r.components[name]
	return c, ok
}

// InitAll は登録済みの全コンポーネントを初期化する。
// いずれかのコンポーネントの初期化に失敗した場合、その時点でエラーを返す。
func (r *ComponentRegistry) InitAll(ctx context.Context) error {
	r.mu.RLock()
	defer r.mu.RUnlock()

	for name, component := range r.components {
		meta := Metadata{Name: name, Version: component.Version()}
		if err := component.Init(ctx, meta); err != nil {
			return fmt.Errorf("コンポーネント '%s' の初期化に失敗しました: %w", name, err)
		}
	}
	return nil
}

// CloseAll は登録済みの全コンポーネントをクローズする。
// いずれかのコンポーネントのクローズに失敗した場合、その時点でエラーを返す。
func (r *ComponentRegistry) CloseAll(ctx context.Context) error {
	r.mu.RLock()
	defer r.mu.RUnlock()

	for name, component := range r.components {
		if err := component.Close(ctx); err != nil {
			return fmt.Errorf("コンポーネント '%s' のクローズに失敗しました: %w", name, err)
		}
	}
	return nil
}

// StatusAll は登録済みの全コンポーネントのステータスをマップとして返す。
func (r *ComponentRegistry) StatusAll(ctx context.Context) map[string]ComponentStatus {
	r.mu.RLock()
	defer r.mu.RUnlock()

	statuses := make(map[string]ComponentStatus, len(r.components))
	for name, component := range r.components {
		statuses[name] = component.Status(ctx)
	}
	return statuses
}
