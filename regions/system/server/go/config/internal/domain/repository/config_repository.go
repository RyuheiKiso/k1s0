package repository

import (
	"context"

	"github.com/k1s0-platform/system-server-go-config/internal/domain/model"
)

// ConfigRepository は設定エントリの永続化インターフェース。
type ConfigRepository interface {
	// GetByKey は namespace と key で設定エントリを取得する。
	GetByKey(ctx context.Context, namespace, key string) (*model.ConfigEntry, error)

	// ListByNamespace は namespace 内の設定エントリを一覧取得する。
	ListByNamespace(ctx context.Context, params ConfigListParams) ([]*model.ConfigEntry, int, error)

	// GetByServiceName はサービス名に対応する設定エントリを取得する。
	GetByServiceName(ctx context.Context, serviceName string) ([]*model.ConfigEntry, error)

	// Create は設定エントリを作成する。
	Create(ctx context.Context, entry *model.ConfigEntry) error

	// Update は設定エントリを更新する。バージョンが一致しない場合はエラーを返す。
	Update(ctx context.Context, entry *model.ConfigEntry, expectedVersion int) error

	// Delete は設定エントリを削除する。
	Delete(ctx context.Context, namespace, key string) error
}

// ConfigListParams は設定エントリ一覧取得のパラメータ。
type ConfigListParams struct {
	Namespace string
	Search    string
	Page      int
	PageSize  int
}
