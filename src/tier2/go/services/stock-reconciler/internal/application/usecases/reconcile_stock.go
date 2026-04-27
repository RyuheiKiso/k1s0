// 在庫差分検出 + 同期ユースケース。
//
// docs 正典:
//   docs/05_実装/00_ディレクトリ設計/30_tier2レイアウト/04_サービス単位の内部構造.md
//
// 役割:
//   外部システムから供給された authoritative な在庫数量と、k1s0 State 上の現在値を突き合わせ、
//   差分があれば k1s0 State を更新したうえで PubSub `stock.reconciled` イベントを発火する。
//   tier1 facade を SDK 経由で呼ぶ Application 層の代表ユースケース。

// Package usecases は Application 層のユースケース実装。
package usecases

// 標準 / 内部 import。
import (
	// context 伝搬。
	"context"
	// crypto/rand で ID 生成（uuid 依存を避けたシンプル UUIDv4 風）。
	"crypto/rand"
	// JSON シリアライズ。
	"encoding/json"
	// 16 進エンコード。
	"encoding/hex"
	// errors.Is / .As。
	"errors"
	// 文字列整形。
	"fmt"
	// 現在時刻。
	"time"

	// Stock エンティティ。
	"github.com/k1s0/k1s0/src/tier2/go/services/stock-reconciler/internal/domain/entity"
	// ドメインイベント。
	"github.com/k1s0/k1s0/src/tier2/go/services/stock-reconciler/internal/domain/event"
	// Stock Repository interface。
	"github.com/k1s0/k1s0/src/tier2/go/services/stock-reconciler/internal/domain/repository"
	// SKU 値オブジェクト。
	"github.com/k1s0/k1s0/src/tier2/go/services/stock-reconciler/internal/domain/value"
	// 設定。
	"github.com/k1s0/k1s0/src/tier2/go/services/stock-reconciler/internal/config"
	// k1s0 SDK ラッパー（PubSub publish 用）。
	"github.com/k1s0/k1s0/src/tier2/go/services/stock-reconciler/internal/infrastructure/external"
	// tier2 共通エラー。
	t2errors "github.com/k1s0/k1s0/src/tier2/go/shared/errors"
)

// Publisher は PubSub publish の最小 interface（モック容易性のため Application 層で抽象化）。
type Publisher interface {
	// Publish は topic に対し JSON ペイロードを発火する。idempotencyKey は重複排除キー。
	Publish(ctx context.Context, topic string, data []byte, contentType, idempotencyKey string) (int64, error)
}

// k1s0Publisher は K1s0Client を Publisher interface に適合させる。
type k1s0Publisher struct {
	// k1s0 SDK ラッパー。
	client *external.K1s0Client
}

// Publish は K1s0Client の PubSubPublish を呼ぶ。
func (p *k1s0Publisher) Publish(ctx context.Context, topic string, data []byte, contentType, idempotencyKey string) (int64, error) {
	// SDK ラッパーへ委譲する。
	return p.client.PubSubPublish(ctx, topic, data, contentType, idempotencyKey)
}

// ReconcileUseCase は reconcile ユースケース本体。
type ReconcileUseCase struct {
	// Stock Repository（Domain interface）。
	repo repository.StockRepository
	// PubSub publisher（Application interface）。
	publisher Publisher
	// PubSub topic 設定。
	pubsubCfg config.PubSubConfig
	// 時刻取得関数（テスト容易性のため注入可能）。
	now func() time.Time
}

// NewReconcileUseCase は UseCase を組み立てる。
func NewReconcileUseCase(repo repository.StockRepository, k1s0Client *external.K1s0Client, pubsubCfg config.PubSubConfig) *ReconcileUseCase {
	// 構造体を組み立てる。
	return &ReconcileUseCase{
		// Repository を保持する。
		repo: repo,
		// k1s0 SDK ラッパーを Publisher 適合させて保持する。
		publisher: &k1s0Publisher{client: k1s0Client},
		// PubSub topic 設定。
		pubsubCfg: pubsubCfg,
		// 既定では UTC 現在時刻を使う。
		now: func() time.Time { return time.Now().UTC() },
	}
}

// ReconcileInput は reconcile の入力 DTO。
type ReconcileInput struct {
	// SKU 文字列（Api 層で受け取った生値）。
	SKU string
	// authoritative 数量（外部システムが正と主張する値）。
	Authoritative int64
	// 同期元システム ID（例: "wms-primary"）。
	SourceSystem string
}

// ReconcileResult は reconcile の出力 DTO。
type ReconcileResult struct {
	// SKU 文字列。
	SKU string
	// reconcile 前の数量。
	BeforeQuantity int64
	// reconcile 後の数量。
	AfterQuantity int64
	// 差分（After - Before）。
	Delta int64
	// PubSub 発火に成功したかどうか（false の時はイベントは飛んでいない、log 経由で再 reconcile 推奨）。
	EventPublished bool
	// PubSub の event_id。
	EventID string
}

// Execute は 1 SKU の reconcile を実施する。
//
// 処理順序:
//
//	1. 入力バリデーション
//	2. k1s0 State から現在値を取得
//	3. 差分計算（差分 0 なら publish skip）
//	4. authoritative で State を更新
//	5. PubSub `stock.reconciled` を発火
//
// 戻り値の error は tier2 共通の DomainError 体系で返す（Api 層が HTTP status に写像）。
func (u *ReconcileUseCase) Execute(ctx context.Context, in ReconcileInput) (*ReconcileResult, error) {
	// SKU を Domain 値オブジェクトに変換する（バリデーションを兼ねる）。
	sku, err := value.NewSKU(in.SKU)
	// SKU が不正なら 400。
	if err != nil {
		// VALIDATION カテゴリの DomainError を返す。
		return nil, t2errors.Wrap(t2errors.CategoryValidation, "E-T2-RECON-001", "invalid SKU", err)
	}
	// authoritative が負の値ならドメイン不変条件違反。
	if in.Authoritative < 0 {
		// VALIDATION カテゴリ。
		return nil, t2errors.New(t2errors.CategoryValidation, "E-T2-RECON-002", fmt.Sprintf("authoritative must be >= 0, got %d", in.Authoritative))
	}
	// 同期元システム ID は監査要件で必須。
	if in.SourceSystem == "" {
		// VALIDATION カテゴリ。
		return nil, t2errors.New(t2errors.CategoryValidation, "E-T2-RECON-003", "source_system is required")
	}
	// 現在の在庫を取得する。
	stock, err := u.repo.FindBySKU(ctx, sku)
	// 未存在は 0 個として扱う（初回登録時）。
	if errors.Is(err, repository.ErrNotFound) {
		// 0 個から始まる Stock を生成する（Stock の不変条件は満たす）。
		stock, err = entity.NewStock(sku, 0, u.now())
		// 0 は当然不変条件を満たすが、念のため err チェック。
		if err != nil {
			// 想定外のロジック破綻のため INTERNAL。
			return nil, t2errors.Wrap(t2errors.CategoryInternal, "E-T2-RECON-010", "failed to construct zero stock", err)
		}
	} else if err != nil {
		// k1s0 State 取得失敗は UPSTREAM。
		return nil, t2errors.Wrap(t2errors.CategoryUpstream, "E-T2-RECON-011", "failed to load stock", err)
	}
	// 反映前の数量を保持する（イベント payload 用）。
	before := stock.Quantity()
	// 差分を計算する。
	if before == in.Authoritative {
		// 差分 0 なら何もせず返却する（イベント発火も skip、Idempotency 観点で重複イベント抑制）。
		return &ReconcileResult{
			// SKU 文字列。
			SKU: sku.String(),
			// 前後ともに同値。
			BeforeQuantity: before,
			AfterQuantity:  before,
			// 差分 0。
			Delta: 0,
			// イベント発火 skip。
			EventPublished: false,
			// EventID は空。
			EventID: "",
		}, nil
	}
	// authoritative に揃える。
	if err := stock.SyncTo(in.Authoritative, u.now()); err != nil {
		// 不変条件違反は INTERNAL。
		return nil, t2errors.Wrap(t2errors.CategoryInternal, "E-T2-RECON-012", "sync to authoritative failed", err)
	}
	// k1s0 State に保存する。
	if err := u.repo.Save(ctx, stock); err != nil {
		// 保存失敗は UPSTREAM。
		return nil, t2errors.Wrap(t2errors.CategoryUpstream, "E-T2-RECON-013", "failed to save stock", err)
	}
	// イベント ID を生成する（PubSub 重複排除キー）。
	eventID, idErr := newEventID()
	// 生成失敗時はエラーを返さず空 ID で発火する（イベント自体は届くが冪等性は OS 側 Kafka に依存）。
	if idErr != nil {
		// 安全側に空文字を採用。
		eventID = ""
	}
	// ドメインイベントを組み立てる。
	evt := event.NewStockReconciled(sku.String(), before, in.Authoritative, in.SourceSystem, eventID, u.now())
	// JSON にエンコードする。
	payload, err := json.Marshal(evt)
	// エンコード失敗は INTERNAL。
	if err != nil {
		// 状態は更新済だがイベントは飛んでいないため、運用上は再 reconcile が必要。
		return nil, t2errors.Wrap(t2errors.CategoryInternal, "E-T2-RECON-014", "failed to marshal event", err)
	}
	// PubSub に publish する。
	if _, err := u.publisher.Publish(ctx, u.pubsubCfg.Topic, payload, "application/json", eventID); err != nil {
		// publish 失敗は UPSTREAM（State は更新済なので caller 側で再 publish の判断が必要）。
		return nil, t2errors.Wrap(t2errors.CategoryUpstream, "E-T2-RECON-015", "failed to publish reconciled event", err)
	}
	// 結果を組み立てて返す。
	return &ReconcileResult{
		// SKU 文字列。
		SKU: sku.String(),
		// 反映前の数量。
		BeforeQuantity: before,
		// 反映後の数量。
		AfterQuantity: in.Authoritative,
		// 差分。
		Delta: in.Authoritative - before,
		// イベント発火成功。
		EventPublished: true,
		// イベント ID。
		EventID: eventID,
	}, nil
}

// newEventID は uuid v4 風の 32 hex 文字列を生成する。
//
// uuid library 依存を避け、k1s0 全体の依存軽量化方針に揃える。
func newEventID() (string, error) {
	// 16 byte バッファを用意する。
	b := make([]byte, 16)
	// crypto/rand で埋める。
	if _, err := rand.Read(b); err != nil {
		// 生成失敗時は呼出元へ。
		return "", err
	}
	// hex 文字列に変換して返す。
	return hex.EncodeToString(b), nil
}
