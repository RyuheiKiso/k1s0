// validating_webhook.go — k1s0 tier1 operator: admission webhook (validating)
// Tier1Service リソースの conformanceClass / quotaClass / authClass / keyClass フィールドを
// admission 時に検証する。
// 04_認証適合仕様.md / 05_鍵管理適合仕様.md / 09_テナント容量適合仕様.md に準拠する。

// パッケージ名: webhook（internal パッケージ: operator 外部からのインポート禁止）
package webhook

import (
	// context パッケージのインポート: admission ハンドラのコンテキスト管理に使用する
	"context"
	// encoding/json パッケージのインポート: admission オブジェクトの JSON デコードに使用する
	"encoding/json"
	// fmt パッケージのインポート: 検証エラーメッセージのフォーマットに使用する
	"fmt"
	// strings パッケージのインポート: class 値の検証に使用する
	"strings"

	// admission パッケージのインポート: webhook.Admission インターフェースに使用する
	"sigs.k8s.io/controller-runtime/pkg/webhook/admission"
	// tier1 v1 API のインポート: Tier1Service 型を参照する
	tier1v1 "github.com/k1s0/tier1-operator/api/v1"
)

// validQuotaClasses は 09_テナント容量適合仕様.md §v1 quota_class セットの有効値一覧
// Kyverno tier1-quota-class-annotation-strict policy と同一の値セットを使用する
var validQuotaClasses = []string{
	// v1_lite クラス: 最小クォータ構成
	"v1_lite",
	// v1_standard クラス: 標準クォータ構成
	"v1_standard",
	// v1_professional クラス: プロフェッショナルクォータ構成
	"v1_professional",
	// v1_enterprise クラス: エンタープライズクォータ構成
	"v1_enterprise",
	// v1_dedicated クラス: 専有クォータ構成
	"v1_dedicated",
}

// validConformanceClasses は 01_Bidi適合仕様.md §v1 conformance_class セットの有効値一覧
// docs/04_詳細設計/01_適合仕様/01_Bidi適合仕様.md §3.2 v1 class セット（5 class）に準拠する
var validConformanceClasses = []string{
	// v1_interactive: 対話型双方向通信クラス（ユーザー操作に対するリアルタイム応答）
	"v1_interactive",
	// v1_alert: アラート通知クラス（サーバ起点のプッシュ通知）
	"v1_alert",
	// v1_event_feed: イベントフィードクラス（ドメインイベントのストリーム配信）
	"v1_event_feed",
	// v1_live_snapshot: ライブスナップショットクラス（集計状態のリアルタイム同期）
	"v1_live_snapshot",
	// v1_bulk_upload: バルクアップロードクラス（クライアント起点の大量データ送信）
	"v1_bulk_upload",
}

// Tier1ServiceValidator は Tier1Service リソースの admission validation を行う構造体
// controller-runtime の admission.Handler インターフェースを実装する
type Tier1ServiceValidator struct {
	// decoder は admission review オブジェクトの JSON デコーダー
	decoder *admission.Decoder
}

// NewTier1ServiceValidator は Tier1ServiceValidator のコンストラクタ
// decoder には admission.NewDecoder(scheme) で生成したデコーダーを渡す
func NewTier1ServiceValidator(decoder *admission.Decoder) *Tier1ServiceValidator {
	// Tier1ServiceValidator を生成して返す
	return &Tier1ServiceValidator{
		// decoder を設定する
		decoder: decoder,
	}
}

// Handle は admission webhook のリクエストを処理する（admission.Handler インターフェース実装）
// conformanceClass / quotaClass の値が spec の有効セットに含まれているかを検証する
func (v *Tier1ServiceValidator) Handle(ctx context.Context, req admission.Request) admission.Response {
	// 未使用 context の suppressをコンパイラに通知する
	_ = ctx

	// ---- 1. オブジェクトの空チェック ----

	// オブジェクトが空の場合は allowed を返す（DELETE webhook では object が nil になる）
	if req.Object.Raw == nil {
		// 空オブジェクトは検証をスキップして通過させる
		return admission.Allowed("empty object: validation skipped")
	}

	// ---- 2. Tier1Service オブジェクトを JSON デコードする ----

	// Tier1Service 変数を宣言する
	var tier1Svc tier1v1.Tier1Service
	// admission review の Object.Raw を JSON デコードする
	if err := json.Unmarshal(req.Object.Raw, &tier1Svc); err != nil {
		// JSON デコード失敗は denied レスポンスを返す
		return admission.Denied(fmt.Sprintf("Tier1Service JSON decode failed: %v", err))
	}

	// ---- 3. conformanceClass フィールドの検証 ----

	// conformanceClass の値が有効セットに含まれているかを確認する
	if tier1Svc.Spec.ConformanceClass != "" {
		// 有効値セットとの照合を実行する
		if !containsString(validConformanceClasses, tier1Svc.Spec.ConformanceClass) {
			// 無効な conformanceClass は denied を返す
			return admission.Denied(fmt.Sprintf(
				"tier1: spec.conformanceClass %q は無効な値です。有効値: %s (01_Bidi適合仕様)",
				tier1Svc.Spec.ConformanceClass,
				strings.Join(validConformanceClasses, " / "),
			))
		}
	}

	// ---- 4. quotaClass フィールドの検証 ----

	// quotaClass の値が有効セットに含まれているかを確認する
	if tier1Svc.Spec.QuotaClass != "" {
		// 有効値セットとの照合を実行する
		if !containsString(validQuotaClasses, tier1Svc.Spec.QuotaClass) {
			// 無効な quotaClass は denied を返す
			return admission.Denied(fmt.Sprintf(
				"tier1: spec.quotaClass %q は無効な値です。有効値: %s (09_テナント容量適合仕様)",
				tier1Svc.Spec.QuotaClass,
				strings.Join(validQuotaClasses, " / "),
			))
		}
	}

	// TODO: authClass / keyClass フィールドの validation を実装する
	// authClass は 04_認証適合仕様.md §v1 auth_class セットと照合する
	// keyClass は 05_鍵管理適合仕様.md §v1 key_class セットと照合する

	// 全検証を通過した場合は allowed を返す
	return admission.Allowed(fmt.Sprintf("Tier1Service %s/%s validation passed", req.Namespace, req.Name))
}

// containsString は haystack スライスの中に needle が含まれているかを返すヘルパー関数
func containsString(haystack []string, needle string) bool {
	// haystack の各要素と needle を比較する
	for _, s := range haystack {
		// 一致する要素が見つかれば true を返す
		if s == needle {
			return true
		}
	}
	// 一致する要素が見つからなかった場合は false を返す
	return false
}
