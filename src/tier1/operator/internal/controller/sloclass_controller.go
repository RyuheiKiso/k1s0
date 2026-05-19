// sloclass_controller.go — k1s0 tier1 operator: SLOClass Reconciler
// 07_SLO適合仕様.md §v1 slo_class セット（6 class）に準拠する。
// SLOClass CRD から PrometheusRule を動的生成し、
// error budget burn rate が閾値を超えたら Kyverno freeze ConfigMap をトリガーする。

// パッケージ名: controller（internal パッケージ: operator 外部からのインポート禁止）
package controller

import (
	// context パッケージのインポート: Reconcile コンテキスト管理に使用する
	"context"
	// encoding/json パッケージのインポート: PrometheusRule spec の JSON 生成に使用する
	"encoding/json"
	// fmt パッケージのインポート: エラーメッセージのフォーマットに使用する
	"fmt"
	// time パッケージのインポート: RequeueAfter の指定に使用する
	"time"

	// Kubernetes core/v1 API のインポート: ConfigMap に使用する
	corev1 "k8s.io/api/core/v1"
	// Kubernetes errors パッケージのインポート: IsNotFound 判定に使用する
	apierrors "k8s.io/apimachinery/pkg/api/errors"
	// Kubernetes API マシナリーのインポート: metav1.Time に使用する
	metav1 "k8s.io/apimachinery/pkg/apis/meta/v1"
	// Kubernetes unstructured のインポート: PrometheusRule CRD を動的生成するために使用する
	"k8s.io/apimachinery/pkg/apis/meta/v1/unstructured"
	// Kubernetes runtime schema のインポート: GroupVersionResource 指定に使用する
	"k8s.io/apimachinery/pkg/runtime/schema"
	// controller-runtime ルートパッケージのインポート: ctrl.NewControllerManagedBy / ctrl.Manager に使用する
	ctrl "sigs.k8s.io/controller-runtime"
	// controller-runtime クライアントのインポート: API サーバとの通信に使用する
	"sigs.k8s.io/controller-runtime/pkg/client"
	// controller-runtime ログのインポート: 構造化ログに使用する
	"sigs.k8s.io/controller-runtime/pkg/log"
	// controller-runtime reconcile のインポート: reconcile.Result / Request 型に使用する
	"sigs.k8s.io/controller-runtime/pkg/reconcile"
	// tier1 v1 API のインポート: SLOClass 型を参照する
	tier1v1 "github.com/k1s0/tier1-operator/api/v1"
)

// prometheusRuleGVR は PrometheusRule CRD の GroupVersionResource を宣言する
// prometheus-operator の CRD に合わせて設定する
var prometheusRuleGVR = schema.GroupVersionResource{
	// Group は prometheus-operator の API グループ
	Group: "monitoring.coreos.com",
	// Version は v1
	Version: "v1",
	// Resource は PrometheusRule の複数形
	Resource: "prometheusrules",
}

// mwmbrWindows は MWMBR（Multi-Window Multi-Burn-Rate）の 4 window セットを宣言する
// spec 07 §burn_rate_windows（4 window / 2 severity）に準拠する
var defaultMWMBRWindows = []tier1v1.BurnRateWindow{
	// 5m / 1h ペア: 高速 burn（page immediate）
	{Short: "5m", Long: "1h", BurnRateThreshold: 14.4},
	// 30m / 6h ペア: 中速 burn（page business hours）
	{Short: "30m", Long: "6h", BurnRateThreshold: 6.0},
	// 2h / 1d ペア: 低速 burn（ticket only）
	{Short: "2h", Long: "1d", BurnRateThreshold: 3.0},
	// 6h / 3d ペア: 超低速 burn（ticket only）
	{Short: "6h", Long: "3d", BurnRateThreshold: 1.0},
}

// buildPrometheusRuleSpec は SLOClass spec から PrometheusRule の rules を構築する
// MWMBR アラートを 4 window 分生成して返す
func buildPrometheusRuleSpec(sloClass *tier1v1.SLOClass) map[string]interface{} {
	// PrometheusRule に埋め込む groups を構築する
	groups := []map[string]interface{}{}
	// SLO クラス名をメトリクス名のプレフィックスとして使用する
	sloName := string(sloClass.Spec.SloClass)
	// burn rate windows を取得する（CRD に設定がなければデフォルト 4 window を使用する）
	windows := sloClass.Spec.BurnRateWindows
	// CRD に BurnRateWindows が未設定の場合はデフォルトを使用する
	if len(windows) == 0 {
		// デフォルト MWMBR windows を設定する
		windows = defaultMWMBRWindows
	}
	// 各 window ペアに対応するアラートルールを生成する
	for i, w := range windows {
		// window ペアのアラートルール名を生成する
		alertName := fmt.Sprintf("SLOBurnRate_%s_w%d", sloName, i+1)
		// アラートルールを構築する
		rule := map[string]interface{}{
			// alert はアラート名を設定する
			"alert": alertName,
			// expr は burn rate 計算式を設定する
			// SoT: src/ops/alert_catalog/slo_breach_rules.yaml と同じメトリクス名を使用する
			// k1s0_request_errors_total / k1s0_request_total / k1s0_slo_error_rate_threshold
			"expr": fmt.Sprintf(
				"( sum(rate(k1s0_request_errors_total{slo_class=%q}[%s])) / sum(rate(k1s0_request_total{slo_class=%q}[%s])) ) / on() group_left() k1s0_slo_error_rate_threshold{slo_class=%q} > %f",
				sloName, w.Short,
				sloName, w.Long,
				sloName,
				w.BurnRateThreshold,
			),
			// for は アラート発火までの持続時間（short window の 1/2）
			"for": w.Short,
			// labels はアラートのメタデータ
			"labels": map[string]string{
				// slo_class ラベルを設定する
				"slo_class": sloName,
				// severity はwindow index で判定する（0,1 は page_immediate）
				"severity": func() string {
					// window 0 と 1 は即時 page（高速 burn）
					if i < 2 {
						// 即時 page の severity を返す
						return "page_immediate"
					}
					// window 2 以降は ticket only（低速 burn）
					return "ticket_only"
				}(),
			},
			// annotations はアラートの説明
			"annotations": map[string]string{
				// summary アノテーションを設定する
				"summary": fmt.Sprintf("SLO burn rate too high for %s (window %d: %s/%s)", sloName, i+1, w.Short, w.Long),
				// runbook_url は ops runbook へのリンク（実際の URL に置換する）
				"runbook_url": fmt.Sprintf("https://runbooks.k1s0.internal/slo/%s", sloName),
			},
		}
		// アラートルールを group に追加する
		group := map[string]interface{}{
			// name は ルールグループ名
			"name": fmt.Sprintf("k1s0.slo.%s.w%d", sloName, i+1),
			// rules は このグループのアラートルール一覧
			"rules": []map[string]interface{}{rule},
		}
		// groups に追加する
		groups = append(groups, group)
	}
	// PrometheusRule spec を構築して返す
	return map[string]interface{}{
		// groups はルールグループの一覧
		"groups": groups,
	}
}

// freezeConfigMapName は Kyverno freeze をトリガーする ConfigMap の名前を返す
// error_budget_freeze.yaml が参照する ConfigMap 名と一致させる
func freezeConfigMapName(sloClassName string) string {
	// ConfigMap 名を slo class 名から生成する
	return fmt.Sprintf("k1s0-slo-freeze-%s", sloClassName)
}

// SLOClassReconciler は SLOClass リソースを reconcile するコントローラ構造体
// 07_SLO適合仕様.md §slo_class セット（v1_bronze / v1_silver / v1_gold / v1_platinum / v1_critical）に対応する
type SLOClassReconciler struct {
	// Kubernetes クライアント: API サーバとのリソース読み書きに使用する
	client.Client
}

// SetupWithManager はコントローラをマネージャーに登録して SLOClass イベントを監視する
// 07_SLO適合仕様.md §slo_class enforcement のエントリポイントとなる
func (r *SLOClassReconciler) SetupWithManager(mgr ctrl.Manager) error {
	// ctrl.NewControllerManagedBy でコントローラを構築してマネージャーに登録する
	return ctrl.NewControllerManagedBy(mgr).
		// SLOClass リソースの変更イベントを監視する
		For(&tier1v1.SLOClass{}).
		// コントローラを登録して返す
		Complete(r)
}

// Reconcile は SLOClass リソースの desired state と actual state を一致させる
// 07_SLO適合仕様.md §slo_class enforcement のメインロジックを担う
func (r *SLOClassReconciler) Reconcile(ctx context.Context, req reconcile.Request) (reconcile.Result, error) {
	// ロガーをコンテキストから取得する
	logger := log.FromContext(ctx)
	// Reconcile 開始をログに記録する
	logger.Info("Reconciling SLOClass", "name", req.Name, "namespace", req.Namespace)

	// ---- 1. SLOClass リソースを API サーバから取得する ----

	// reconcile 対象の SLOClass 変数を宣言する
	var sloClass tier1v1.SLOClass
	// API サーバから SLOClass を取得する
	if err := r.Get(ctx, req.NamespacedName, &sloClass); err != nil {
		// リソースが存在しない場合は正常終了する（削除済みの可能性）
		return reconcile.Result{}, client.IgnoreNotFound(err)
	}

	// ---- 2. PrometheusRule を動的生成して apply する ----

	// PrometheusRule の名前を設定する
	prName := fmt.Sprintf("k1s0-slo-%s", sloClass.Name)
	// PrometheusRule spec を構築する
	prSpec := buildPrometheusRuleSpec(&sloClass)
	// PrometheusRule spec を JSON 変換する（unstructured 化のため）
	prSpecBytes, err := json.Marshal(prSpec)
	// JSON 変換失敗時はエラーを返す
	if err != nil {
		// JSON 変換エラーをラップして返す
		return reconcile.Result{}, fmt.Errorf("PrometheusRule spec JSON 変換失敗: %w", err)
	}
	// JSON から map[string]interface{} に戻す
	var prSpecMap map[string]interface{}
	// JSON デコードする
	if err := json.Unmarshal(prSpecBytes, &prSpecMap); err != nil {
		// JSON デコードエラーをラップして返す
		return reconcile.Result{}, fmt.Errorf("PrometheusRule spec JSON デコード失敗: %w", err)
	}

	// 既存の PrometheusRule を取得する（unstructured 経由）
	existingPR := &unstructured.Unstructured{}
	// GVK を設定する
	existingPR.SetGroupVersionKind(schema.GroupVersionKind{
		// Group を設定する
		Group: prometheusRuleGVR.Group,
		// Version を設定する
		Version: prometheusRuleGVR.Version,
		// Kind を設定する
		Kind: "PrometheusRule",
	})
	// API サーバから PrometheusRule を取得する
	prGetErr := r.Get(ctx, client.ObjectKey{Namespace: req.Namespace, Name: prName}, existingPR)

	// 新規 PrometheusRule を構築する
	desiredPR := &unstructured.Unstructured{
		// Object はすべてのフィールドを含む map
		Object: map[string]interface{}{
			// apiVersion を設定する
			"apiVersion": "monitoring.coreos.com/v1",
			// kind を設定する
			"kind": "PrometheusRule",
			// metadata を設定する
			"metadata": map[string]interface{}{
				// name を設定する
				"name": prName,
				// namespace を設定する
				"namespace": req.Namespace,
				// labels に SLO クラス情報を設定する
				"labels": map[string]interface{}{
					// slo_class ラベルを設定する
					"k1s0.io/slo-class": string(sloClass.Spec.SloClass),
					// tier1 ラベルを設定する
					"k1s0.io/axis": "tier1",
				},
			},
			// spec を設定する
			"spec": prSpecMap,
		},
	}

	// PrometheusRule が存在しない場合は新規作成する
	if apierrors.IsNotFound(prGetErr) {
		// 新規 PrometheusRule を作成する
		if err := r.Create(ctx, desiredPR); err != nil {
			// 作成失敗をログに記録してリトライさせる
			logger.Error(err, "failed to create PrometheusRule", "name", prName)
			// エラーを返してリトライを促す
			return reconcile.Result{}, fmt.Errorf("create PrometheusRule %s: %w", prName, err)
		}
		// 作成成功をログに記録する
		logger.Info("PrometheusRule created", "name", prName, "sloClass", sloClass.Spec.SloClass)
	} else if prGetErr == nil {
		// 既存の PrometheusRule の spec を更新する
		existingPR.Object["spec"] = prSpecMap
		// API サーバに更新を送信する
		if err := r.Update(ctx, existingPR); err != nil {
			// 更新失敗をログに記録してリトライさせる
			logger.Error(err, "failed to update PrometheusRule", "name", prName)
			// エラーを返してリトライを促す
			return reconcile.Result{}, fmt.Errorf("update PrometheusRule %s: %w", prName, err)
		}
		// 更新成功をログに記録する
		logger.Info("PrometheusRule updated", "name", prName)
	} else {
		// Get 呼び出し自体のエラーをラップして返す
		return reconcile.Result{}, fmt.Errorf("get PrometheusRule %s: %w", prName, prGetErr)
	}

	// ---- 3. error budget burn を確認して Kyverno freeze ConfigMap を更新する ----
	// NOTE: 実際の burn rate 計算は Prometheus の recording rule に委譲する。
	// ここでは SLOClass に freezePolicyActive フィールドが設定されている場合に ConfigMap を更新する。

	// freeze ConfigMap の名前を取得する
	freezeCMName := freezeConfigMapName(sloClass.Name)
	// freeze ConfigMap の data を構築する
	freezeData := map[string]string{
		// slo_class を設定する
		"slo_class": string(sloClass.Spec.SloClass),
		// target を設定する
		"target": fmt.Sprintf("%f", sloClass.Spec.Target),
		// freeze_active はフリーズが発動中かを示す（現状は CRD の status から読む）
		"freeze_active": fmt.Sprintf("%v", sloClass.Status.FreezePolicyActive),
	}

	// 既存の freeze ConfigMap を取得する
	var existingCM corev1.ConfigMap
	// API サーバから ConfigMap を取得する
	cmGetErr := r.Get(ctx, client.ObjectKey{Namespace: req.Namespace, Name: freezeCMName}, &existingCM)

	// ConfigMap が存在しない場合は新規作成する
	if apierrors.IsNotFound(cmGetErr) {
		// 新規 freeze ConfigMap を作成する
		newCM := corev1.ConfigMap{
			// ObjectMeta を設定する
			ObjectMeta: metav1.ObjectMeta{
				// 名前を設定する
				Name: freezeCMName,
				// namespace を設定する
				Namespace: req.Namespace,
				// Kyverno freeze policy が参照するラベルを設定する
				Labels: map[string]string{
					// freeze policy 識別ラベルを設定する
					"k1s0.io/slo-freeze": "true",
					// SLO クラスラベルを設定する
					"k1s0.io/slo-class": string(sloClass.Spec.SloClass),
				},
			},
			// Data に freeze 状態を設定する
			Data: freezeData,
		}
		// ConfigMap を作成する
		if err := r.Create(ctx, &newCM); err != nil {
			// 作成失敗をログに記録してリトライさせる
			logger.Error(err, "failed to create freeze ConfigMap", "name", freezeCMName)
			// エラーを返してリトライを促す
			return reconcile.Result{}, fmt.Errorf("create freeze ConfigMap %s: %w", freezeCMName, err)
		}
		// 作成成功をログに記録する
		logger.Info("freeze ConfigMap created", "name", freezeCMName)
	} else if cmGetErr == nil {
		// 既存の ConfigMap の data を更新する
		existingCM.Data = freezeData
		// API サーバに更新を送信する
		if err := r.Update(ctx, &existingCM); err != nil {
			// 更新失敗をログに記録してリトライさせる
			logger.Error(err, "failed to update freeze ConfigMap", "name", freezeCMName)
			// エラーを返してリトライを促す
			return reconcile.Result{}, fmt.Errorf("update freeze ConfigMap %s: %w", freezeCMName, err)
		}
		// 更新成功をログに記録する
		logger.Info("freeze ConfigMap updated", "name", freezeCMName, "freeze_active", sloClass.Status.FreezePolicyActive)
	} else {
		// Get 呼び出し自体のエラーをラップして返す
		return reconcile.Result{}, fmt.Errorf("get freeze ConfigMap %s: %w", freezeCMName, cmGetErr)
	}

	// ---- 4. Status を更新する ----

	// PrometheusRule が正常に apply されたことを記録する
	sloClass.Status.PrometheusRuleApplied = true
	// 最終 Reconcile 時刻を更新する（status 記録目的なので wall-clock 許可）
	now := metav1.Now()
	// ポインタを設定する
	sloClass.Status.LastReconcileAt = &now

	// Status サブリソースを更新する
	if err := r.Status().Update(ctx, &sloClass); err != nil {
		// Status 更新失敗をログに記録してリトライさせる
		logger.Error(err, "Failed to update SLOClass status")
		// エラーを返してリトライを促す
		return reconcile.Result{}, fmt.Errorf("SLOClass status update failed: %w", err)
	}

	// Reconcile 完了をログに記録する
	logger.Info("SLOClass reconciled successfully — PrometheusRule and freeze ConfigMap applied",
		"name", req.Name,
		"sloClass", sloClass.Spec.SloClass,
		"target", sloClass.Spec.Target,
	)

	// 30 分後に再 Reconcile をスケジュールする（SLO 状態の定期チェック）
	return reconcile.Result{RequeueAfter: 30 * time.Minute}, nil
}
