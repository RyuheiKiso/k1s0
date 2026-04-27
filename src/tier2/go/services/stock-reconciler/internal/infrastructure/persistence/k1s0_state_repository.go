// k1s0 State backed の Stock Repository 実装。
//
// docs 正典:
//   docs/05_実装/00_ディレクトリ設計/30_tier2レイアウト/04_サービス単位の内部構造.md
//
// 役割:
//   Domain の StockRepository interface を k1s0 State（DS-SW-COMP-005）で実装する。
//   key は "stock:<sku>"、値は JSON シリアライズされた Stock スナップショット。

// Package persistence は Domain Repository interface の実装を集約する。
package persistence

// 標準 / 内部 import。
import (
	// context 伝搬。
	"context"
	// JSON シリアライズ。
	"encoding/json"
	// エラー連結。
	"errors"
	// 文字列整形。
	"fmt"
	// 時刻記録。
	"time"

	// Stock エンティティ。
	"github.com/k1s0/k1s0/src/tier2/go/services/stock-reconciler/internal/domain/entity"
	// Stock Repository interface（ErrNotFound 含む）。
	"github.com/k1s0/k1s0/src/tier2/go/services/stock-reconciler/internal/domain/repository"
	// SKU 値オブジェクト。
	"github.com/k1s0/k1s0/src/tier2/go/services/stock-reconciler/internal/domain/value"
	// k1s0 SDK ラッパー。
	"github.com/k1s0/k1s0/src/tier2/go/services/stock-reconciler/internal/infrastructure/external"
)

// stockSnapshot は永続化用の DTO（JSON シリアライズ）。
type stockSnapshot struct {
	// SKU 文字列。
	SKU string `json:"sku"`
	// 数量。
	Quantity int64 `json:"quantity"`
	// 最終同期時刻（RFC3339）。
	SyncedAt time.Time `json:"synced_at"`
}

// K1s0StateRepository は k1s0 State をバックエンドにした Stock Repository。
type K1s0StateRepository struct {
	// k1s0 Infrastructure ラッパー。
	client *external.K1s0Client
	// State Store 名（Dapr Component の metadata.name）。
	store string
}

// NewK1s0StateRepository は K1s0StateRepository を組み立てる。
func NewK1s0StateRepository(client *external.K1s0Client, store string) *K1s0StateRepository {
	// store 名が空なら "postgres" を既定にする。
	if store == "" {
		// k1s0 Component の既定 store 名。
		store = "postgres"
	}
	// インスタンスを返す。
	return &K1s0StateRepository{client: client, store: store}
}

// FindBySKU は SKU で Stock を取得する。
func (r *K1s0StateRepository) FindBySKU(ctx context.Context, sku value.SKU) (*entity.Stock, error) {
	// State key を組み立てる。
	key := keyForSKU(sku)
	// k1s0 State から取得する。
	data, _, found, err := r.client.StateGet(ctx, r.store, key)
	// 取得エラーは伝搬する。
	if err != nil {
		// caller でラップ済 error として扱う。
		return nil, fmt.Errorf("persistence.FindBySKU: state get failed: %w", err)
	}
	// 未存在は ErrNotFound センチネル。
	if !found {
		// Domain 層共通のセンチネル。
		return nil, repository.ErrNotFound
	}
	// JSON をデコードする。
	var snap stockSnapshot
	// デコード失敗は破損データ扱いで error。
	if err := json.Unmarshal(data, &snap); err != nil {
		// caller でログ + alert を判断する。
		return nil, fmt.Errorf("persistence.FindBySKU: unmarshal failed: %w", err)
	}
	// SKU 値オブジェクトに復元する（snapshot の SKU 文字列が無効ならエラー）。
	skuVO, err := value.NewSKU(snap.SKU)
	// 不正値は破損扱い。
	if err != nil {
		// caller でログ + alert を判断する。
		return nil, fmt.Errorf("persistence.FindBySKU: invalid SKU in snapshot: %w", err)
	}
	// Stock エンティティを復元する。
	stock, err := entity.NewStock(skuVO, snap.Quantity, snap.SyncedAt)
	// 不変条件違反は破損扱い。
	if err != nil {
		// caller でログ + alert を判断する。
		return nil, fmt.Errorf("persistence.FindBySKU: invalid stock in snapshot: %w", err)
	}
	// 復元済 Stock を返す。
	return stock, nil
}

// Save は Stock を k1s0 State に保存する。
func (r *K1s0StateRepository) Save(ctx context.Context, stock *entity.Stock) error {
	// nil ガード。
	if stock == nil {
		// caller の実装ミスを早期に検知する。
		return errors.New("persistence.Save: stock is nil")
	}
	// snapshot を組み立てる。
	snap := stockSnapshot{
		// SKU 文字列。
		SKU: stock.SKU().String(),
		// 数量。
		Quantity: stock.Quantity(),
		// 最終同期時刻。
		SyncedAt: stock.SyncedAt(),
	}
	// JSON にエンコードする。
	data, err := json.Marshal(snap)
	// エンコード失敗は実装バグ扱い。
	if err != nil {
		// 上層でログ。
		return fmt.Errorf("persistence.Save: marshal failed: %w", err)
	}
	// State key を組み立てる。
	key := keyForSKU(stock.SKU())
	// k1s0 State に保存する（リリース時点 は楽観的排他なし、リリース時点 で ETag 連携）。
	if _, err := r.client.StateSave(ctx, r.store, key, data); err != nil {
		// caller で retry / dlq 判断。
		return fmt.Errorf("persistence.Save: state save failed: %w", err)
	}
	// 正常終了。
	return nil
}

// keyForSKU は SKU から State key を組み立てる。
func keyForSKU(sku value.SKU) string {
	// プレフィックス + SKU 文字列。
	return "stock:" + sku.String()
}
