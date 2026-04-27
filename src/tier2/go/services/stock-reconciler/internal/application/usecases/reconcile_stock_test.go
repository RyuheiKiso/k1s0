// ReconcileUseCase の単体テスト。
//
// docs 正典:
//   docs/05_実装/00_ディレクトリ設計/30_tier2レイアウト/04_サービス単位の内部構造.md
//
// 役割:
//   k1s0 SDK / 実 PubSub に依存せずに UseCase の業務ロジックを検証する。
//   Repository / Publisher は in-memory mock で差し替える。

package usecases

// 標準 / 内部 import。
import (
	// context 伝搬。
	"context"
	// errors.Is。
	"errors"
	// 文字列比較。
	"strings"
	// テスト frameworks。
	"testing"
	// 時刻記録。
	"time"

	// Stock エンティティ。
	"github.com/k1s0/k1s0/src/tier2/go/services/stock-reconciler/internal/domain/entity"
	// Stock Repository interface（ErrNotFound）。
	"github.com/k1s0/k1s0/src/tier2/go/services/stock-reconciler/internal/domain/repository"
	// SKU 値オブジェクト。
	"github.com/k1s0/k1s0/src/tier2/go/services/stock-reconciler/internal/domain/value"
	// 設定（PubSubConfig）。
	"github.com/k1s0/k1s0/src/tier2/go/services/stock-reconciler/internal/config"
	// tier2 共通エラー。
	t2errors "github.com/k1s0/k1s0/src/tier2/go/shared/errors"
)

// memRepo は StockRepository の in-memory mock。
type memRepo struct {
	// SKU 文字列をキーにした map。
	store map[string]*entity.Stock
	// 強制エラー（指定時のみ全 RPC が fail）。
	forceErr error
}

// FindBySKU は in-memory store から取得する。
func (m *memRepo) FindBySKU(_ context.Context, sku value.SKU) (*entity.Stock, error) {
	// 強制エラー時はそれを返す。
	if m.forceErr != nil {
		// テスト用エラー。
		return nil, m.forceErr
	}
	// store から検索する。
	stock, ok := m.store[sku.String()]
	// 未存在は ErrNotFound。
	if !ok {
		// Domain 層共通センチネル。
		return nil, repository.ErrNotFound
	}
	// 値を返す。
	return stock, nil
}

// Save は in-memory store に保存する。
func (m *memRepo) Save(_ context.Context, stock *entity.Stock) error {
	// 強制エラー時はそれを返す。
	if m.forceErr != nil {
		// テスト用エラー。
		return m.forceErr
	}
	// map に格納する。
	m.store[stock.SKU().String()] = stock
	// 正常終了。
	return nil
}

// memPublisher は Publisher の in-memory mock。
type memPublisher struct {
	// publish 履歴。
	publishes []publishedRecord
	// 強制エラー（指定時のみ全 publish が fail）。
	forceErr error
}

// publishedRecord は publish された 1 件分の記録。
type publishedRecord struct {
	// topic 名。
	topic string
	// payload bytes。
	data []byte
	// content-type。
	contentType string
	// 冪等性キー。
	idempotencyKey string
}

// Publish は publish 履歴に追加する。
func (m *memPublisher) Publish(_ context.Context, topic string, data []byte, contentType, idempotencyKey string) (int64, error) {
	// 強制エラー時はそれを返す。
	if m.forceErr != nil {
		// テスト用エラー。
		return 0, m.forceErr
	}
	// 履歴に追加する。
	m.publishes = append(m.publishes, publishedRecord{topic: topic, data: data, contentType: contentType, idempotencyKey: idempotencyKey})
	// offset は履歴 index を仮で返す。
	return int64(len(m.publishes) - 1), nil
}

// fixedNow はテスト用の固定時刻を返す。
func fixedNow() time.Time {
	// 固定値を返す。
	return time.Date(2026, 4, 27, 12, 0, 0, 0, time.UTC)
}

// newUseCaseForTest はテスト用に UseCase を組み立てる（時刻と Publisher を固定）。
func newUseCaseForTest(repo repository.StockRepository, pub Publisher) *ReconcileUseCase {
	// 構造体を組み立てる。
	return &ReconcileUseCase{
		// Repository を保持する。
		repo: repo,
		// Publisher を保持する。
		publisher: pub,
		// PubSub 設定。
		pubsubCfg: config.PubSubConfig{ComponentName: "kafka-test", Topic: "stock.reconciled.test"},
		// 時刻を固定する。
		now: fixedNow,
	}
}

// TestExecute_FirstReconcile_PublishesEvent は新規 SKU の reconcile を検証する。
func TestExecute_FirstReconcile_PublishesEvent(t *testing.T) {
	// 空 store。
	repo := &memRepo{store: map[string]*entity.Stock{}}
	// 空 publisher。
	pub := &memPublisher{}
	// UseCase を組み立てる。
	uc := newUseCaseForTest(repo, pub)
	// 実行する。
	result, err := uc.Execute(context.Background(), ReconcileInput{
		// SKU。
		SKU: "ABC-123",
		// authoritative 5。
		Authoritative: 5,
		// 同期元。
		SourceSystem: "wms-primary",
	})
	// 成功するはず。
	if err != nil {
		// 失敗。
		t.Fatalf("Execute failed: %v", err)
	}
	// before=0, after=5, delta=5。
	if result.BeforeQuantity != 0 || result.AfterQuantity != 5 || result.Delta != 5 {
		// 期待外。
		t.Errorf("unexpected result: %+v", result)
	}
	// イベント発火していること。
	if !result.EventPublished {
		// 失敗。
		t.Error("expected EventPublished=true")
	}
	// publish 履歴に 1 件あること。
	if len(pub.publishes) != 1 {
		// 失敗。
		t.Fatalf("expected 1 publish, got %d", len(pub.publishes))
	}
	// topic が一致すること。
	if pub.publishes[0].topic != "stock.reconciled.test" {
		// 失敗。
		t.Errorf("topic = %q", pub.publishes[0].topic)
	}
	// payload に SKU 文字列が含まれること（最低限の中身検査）。
	if !strings.Contains(string(pub.publishes[0].data), `"sku":"ABC-123"`) {
		// 失敗。
		t.Errorf("payload missing sku: %s", string(pub.publishes[0].data))
	}
}

// TestExecute_NoChange_SkipsPublish は差分 0 のケースで publish skip を検証する。
func TestExecute_NoChange_SkipsPublish(t *testing.T) {
	// 既存 Stock を作成する。
	sku := value.MustNewSKU("ABC-123")
	// 既存の在庫 5。
	existing, _ := entity.NewStock(sku, 5, fixedNow())
	// repo に既存 Stock を埋め込む。
	repo := &memRepo{store: map[string]*entity.Stock{"ABC-123": existing}}
	// publisher。
	pub := &memPublisher{}
	// UseCase。
	uc := newUseCaseForTest(repo, pub)
	// authoritative も 5（差分 0）。
	result, err := uc.Execute(context.Background(), ReconcileInput{SKU: "ABC-123", Authoritative: 5, SourceSystem: "wms-primary"})
	// 成功するはず。
	if err != nil {
		// 失敗。
		t.Fatalf("Execute failed: %v", err)
	}
	// 差分 0 / publish skip。
	if result.Delta != 0 || result.EventPublished {
		// 期待外。
		t.Errorf("expected Delta=0 and EventPublished=false, got %+v", result)
	}
	// publish 履歴は空のはず。
	if len(pub.publishes) != 0 {
		// 失敗。
		t.Errorf("expected 0 publishes, got %d", len(pub.publishes))
	}
}

// TestExecute_ValidationErrors は入力バリデーションを検証する。
func TestExecute_ValidationErrors(t *testing.T) {
	// 空 store。
	repo := &memRepo{store: map[string]*entity.Stock{}}
	// 空 publisher。
	pub := &memPublisher{}
	// UseCase。
	uc := newUseCaseForTest(repo, pub)
	// 不正な SKU。
	if _, err := uc.Execute(context.Background(), ReconcileInput{SKU: "_invalid", Authoritative: 1, SourceSystem: "x"}); err == nil {
		// nil は失敗。
		t.Error("expected error for invalid SKU")
	}
	// 負数 authoritative。
	if _, err := uc.Execute(context.Background(), ReconcileInput{SKU: "ABC-1", Authoritative: -1, SourceSystem: "x"}); err == nil {
		// nil は失敗。
		t.Error("expected error for negative authoritative")
	}
	// 同期元なし。
	if _, err := uc.Execute(context.Background(), ReconcileInput{SKU: "ABC-1", Authoritative: 1, SourceSystem: ""}); err == nil {
		// nil は失敗。
		t.Error("expected error for empty source_system")
	}
}

// TestExecute_UpstreamError は repo の取得エラーを UPSTREAM カテゴリで返すことを検証する。
func TestExecute_UpstreamError(t *testing.T) {
	// repo が常に network エラーを返すよう設定する。
	repo := &memRepo{forceErr: errors.New("simulated network failure")}
	// 空 publisher。
	pub := &memPublisher{}
	// UseCase。
	uc := newUseCaseForTest(repo, pub)
	// 実行する。
	_, err := uc.Execute(context.Background(), ReconcileInput{SKU: "ABC-1", Authoritative: 1, SourceSystem: "x"})
	// エラーが返るはず。
	if err == nil {
		// 失敗。
		t.Fatal("expected error, got nil")
	}
	// DomainError であるはず。
	domain, ok := t2errors.AsDomainError(err)
	// 変換失敗は失敗。
	if !ok {
		// 失敗。
		t.Fatalf("expected DomainError, got %T: %v", err, err)
	}
	// UPSTREAM カテゴリのはず。
	if domain.Category != t2errors.CategoryUpstream {
		// 失敗。
		t.Errorf("Category = %q, want UPSTREAM", domain.Category)
	}
}
