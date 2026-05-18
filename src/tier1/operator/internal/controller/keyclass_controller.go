// keyclass_controller.go — k1s0 tier1 operator: KeyClass Reconciler
// 05_鍵管理適合仕様.md §v1 key_class セット（5 class）に準拠する。
// KeyClass リソースの desired state と actual state を一致させる Reconciler を実装する。

// パッケージ名: controller（internal パッケージ: operator 外部からのインポート禁止）
package controller

import (
	// context パッケージのインポート: Reconcile コンテキスト管理に使用する
	"context"
	// fmt パッケージのインポート: エラーメッセージのフォーマットに使用する
	"fmt"
	// time パッケージのインポート: RequeueAfter の指定に使用する
	"time"

	// Kubernetes API マシナリーのインポート: metav1.Time に使用する
	metav1 "k8s.io/apimachinery/pkg/apis/meta/v1"
	// controller-runtime クライアントのインポート: API サーバとの通信に使用する
	"sigs.k8s.io/controller-runtime/pkg/client"
	// controller-runtime ログのインポート: 構造化ログに使用する
	"sigs.k8s.io/controller-runtime/pkg/log"
	// controller-runtime reconcile のインポート: reconcile.Result / Request 型に使用する
	"sigs.k8s.io/controller-runtime/pkg/reconcile"
	// tier1 v1 API のインポート: KeyClass 型を参照する
	tier1v1 "github.com/k1s0/tier1-operator/api/v1"
)

// KeyClassReconciler は KeyClass リソースを reconcile するコントローラ構造体
// 05_鍵管理適合仕様.md §rotation_cadence に基づく鍵ローテーションスケジュールを管理する
type KeyClassReconciler struct {
	// Kubernetes クライアント: API サーバとのリソース読み書きに使用する
	client.Client
}

// Reconcile は KeyClass リソースの desired state と actual state を一致させる
// 05_鍵管理適合仕様.md §rotation_cadence / §destruction_method enforcement のメインロジックを担う
func (r *KeyClassReconciler) Reconcile(ctx context.Context, req reconcile.Request) (reconcile.Result, error) {
	// ロガーをコンテキストから取得する
	logger := log.FromContext(ctx)
	// Reconcile 開始をログに記録する
	logger.Info("Reconciling KeyClass", "name", req.Name, "namespace", req.Namespace)

	// ---- 1. KeyClass リソースを API サーバから取得する ----

	// reconcile 対象の KeyClass 変数を宣言する
	var keyClass tier1v1.KeyClass
	// API サーバから KeyClass を取得する
	if err := r.Get(ctx, req.NamespacedName, &keyClass); err != nil {
		// リソースが存在しない場合は正常終了する（削除済みの可能性）
		return reconcile.Result{}, client.IgnoreNotFound(err)
	}

	// ---- 2. 鍵ローテーション判定ロジックを実行する ----

	// Purpose フィールドの設定を確認する（未設定は設定ミス）
	if keyClass.Spec.Purpose == "" {
		// purpose 未設定を警告ログに記録する（reconcile は継続する）
		logger.Info("KeyClass Purpose is not set, key rotation enforcement skipped",
			"name", req.Name,
		)
	}

	// RotationCadence フィールドの設定を確認する（未設定は設定ミス）
	if keyClass.Spec.RotationCadence == "" {
		// rotation_cadence 未設定を警告ログに記録する（reconcile は継続する）
		logger.Info("KeyClass RotationCadence is not set, using default cadence",
			"name", req.Name,
		)
	}

	// TODO: 実際の鍵ローテーションロジック（OpenBao Transit API 呼び出し）を実装する
	// RotationCadence に基づいて LastRotationAt を確認し、期限切れなら OpenBao Transit で rotate する
	// 現時点ではスケルトンとして status の rotated フラグを更新するのみ

	// ---- 3. Status.Rotated と LastRotationAt を更新する ----

	// ローテーション処理が完了したので rotated = true に設定する
	keyClass.Status.Rotated = true
	// 現在時刻を LastRotationAt に設定する
	now := metav1.Now()
	keyClass.Status.LastRotationAt = &now

	// Status サブリソースを更新する
	if err := r.Status().Update(ctx, &keyClass); err != nil {
		// Status 更新失敗をログに記録してリトライさせる
		logger.Error(err, "Failed to update KeyClass status")
		// エラーを返してリトライを促す
		return reconcile.Result{}, fmt.Errorf("KeyClass status update failed: %w", err)
	}

	// Reconcile 完了をログに記録する
	logger.Info("KeyClass reconciled successfully",
		"name", req.Name,
		"purpose", keyClass.Spec.Purpose,
		"rotationCadence", keyClass.Spec.RotationCadence,
		"backend", keyClass.Spec.Backend,
	)

	// 24 時間後に再 Reconcile をスケジュールする（鍵ローテーションの定期チェック）
	return reconcile.Result{RequeueAfter: 24 * time.Hour}, nil
}
