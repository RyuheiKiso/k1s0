// k1s0-impl: IMPL-tier1-0009 realizes=FR-tier1-009
// quotaclass_controller.go — k1s0 tier1 operator: QuotaClass Reconciler
// 09_テナント容量適合仕様.md §quota enforcement に準拠する。
// QuotaClass CRD から namespace 単位の ResourceQuota / LimitRange を動的生成して適用する。

// パッケージ名: controller（internal パッケージ: operator 外部からのインポート禁止）
package controller

import (
	// context パッケージのインポート: Reconcile コンテキスト管理に使用する
	"context"
	// fmt パッケージのインポート: エラーメッセージのフォーマットに使用する
	"fmt"
	// time パッケージのインポート: RequeueAfter の指定に使用する
	"time"

	// Kubernetes core/v1 API のインポート: ResourceQuota / LimitRange に使用する
	corev1 "k8s.io/api/core/v1"
	// Kubernetes resource.Quantity 型のインポート: CPU / Memory 上限値に使用する
	"k8s.io/apimachinery/pkg/api/resource"
	// Kubernetes API マシナリーのインポート: metav1.Time に使用する
	metav1 "k8s.io/apimachinery/pkg/apis/meta/v1"
	// errors パッケージのインポート: IsNotFound エラー判定に使用する
	apierrors "k8s.io/apimachinery/pkg/api/errors"
	// controller-runtime ルートパッケージのインポート: ctrl.NewControllerManagedBy / ctrl.Manager に使用する
	ctrl "sigs.k8s.io/controller-runtime"
	// controller-runtime クライアントのインポート: API サーバとの通信に使用する
	"sigs.k8s.io/controller-runtime/pkg/client"
	// controller-runtime ログのインポート: 構造化ログに使用する
	"sigs.k8s.io/controller-runtime/pkg/log"
	// controller-runtime reconcile のインポート: reconcile.Result / Request 型に使用する
	"sigs.k8s.io/controller-runtime/pkg/reconcile"
	// tier1 v1 API のインポート: QuotaClass 型を参照する
	tier1v1 "github.com/k1s0/tier1-operator/api/v1"
)

// QuotaClassReconciler は QuotaClass リソースを reconcile するコントローラ構造体
type QuotaClassReconciler struct {
	// Kubernetes クライアント: API サーバとのリソース読み書きに使用する
	client.Client
}

// SetupWithManager はコントローラをマネージャーに登録して QuotaClass イベントを監視する
// 09_テナント容量適合仕様.md §quota enforcement のエントリポイントとなる
func (r *QuotaClassReconciler) SetupWithManager(mgr ctrl.Manager) error {
	// ctrl.NewControllerManagedBy でコントローラを構築してマネージャーに登録する
	return ctrl.NewControllerManagedBy(mgr).
		// QuotaClass リソースの変更イベントを監視する
		For(&tier1v1.QuotaClass{}).
		// コントローラを登録して返す
		Complete(r)
}

// quotaLimitsForClass は quota_class ごとの ResourceQuota / LimitRange 値を返す
// 09_テナント容量適合仕様.md §quota_class 別 limit 表（5 class）に準拠する
func quotaLimitsForClass(classID tier1v1.QuotaClassID) (cpuLimit, memLimit, storageLimit, podLimit string) {
	// クラス別に CPU / Memory / Storage / Pod 上限を設定する
	switch classID {
	// v1_free_shared: 共有インフラの無償プラン（最小構成）
	case tier1v1.V1QuotaFreeShared:
		// CPU 0.5 core / Memory 512Mi / Storage 5Gi / Pod 5 個を上限とする
		return "500m", "512Mi", "5Gi", "5"
	// v1_starter_shared: スターター共有プラン
	case tier1v1.V1QuotaStarterShared:
		// CPU 1 core / Memory 1Gi / Storage 20Gi / Pod 10 個を上限とする
		return "1", "1Gi", "20Gi", "10"
	// v1_team_shared: チーム共有プラン
	case tier1v1.V1QuotaTeamShared:
		// CPU 4 core / Memory 8Gi / Storage 100Gi / Pod 30 個を上限とする
		return "4", "8Gi", "100Gi", "30"
	// v1_business_dedicated: ビジネス専用インフラ
	case tier1v1.V1QuotaBusinessDedicated:
		// CPU 16 core / Memory 32Gi / Storage 500Gi / Pod 100 個を上限とする
		return "16", "32Gi", "500Gi", "100"
	// v1_enterprise_dedicated: エンタープライズ専用インフラ（上限なし扱い）
	case tier1v1.V1QuotaEnterpriseDedicated:
		// CPU 64 core / Memory 128Gi / Storage 2Ti / Pod 500 個を上限とする
		return "64", "128Gi", "2Ti", "500"
	// 未知のクラスは安全側（free_shared 相当）にフォールバックする
	default:
		// デフォルトは free_shared 相当の最小 limit を設定する
		return "500m", "512Mi", "5Gi", "5"
	}
}

// Reconcile は QuotaClass リソースの desired state と actual state を一致させる
// 09_テナント容量適合仕様.md §quota enforcement のメインロジックを担う
func (r *QuotaClassReconciler) Reconcile(ctx context.Context, req reconcile.Request) (reconcile.Result, error) {
	// ロガーをコンテキストから取得する
	logger := log.FromContext(ctx)
	// Reconcile 開始をログに記録する
	logger.Info("Reconciling QuotaClass", "name", req.Name, "namespace", req.Namespace)

	// ---- 1. QuotaClass リソースを API サーバから取得する ----

	// reconcile 対象の QuotaClass 変数を宣言する
	var quotaClass tier1v1.QuotaClass
	// API サーバから QuotaClass を取得する
	if err := r.Get(ctx, req.NamespacedName, &quotaClass); err != nil {
		// リソースが存在しない場合は正常終了する（削除済みの可能性）
		return reconcile.Result{}, client.IgnoreNotFound(err)
	}

	// ---- 2. ClassID の検証 ----

	// ClassID が未設定の場合は警告して再試行する
	if quotaClass.Spec.ClassID == "" {
		// ClassID 未設定を警告ログに記録する
		logger.Info("QuotaClass ClassID is not set, quota enforcement skipped", "name", req.Name)
		// 10 分後に再 Reconcile して設定待ちにする
		return reconcile.Result{RequeueAfter: 10 * time.Minute}, nil
	}

	// ---- 3. quota_class に対応する limit 値を取得する ----

	// quota_class ごとの CPU / Memory / Storage / Pod 上限を取得する
	cpuLimit, memLimit, storageLimit, podLimit := quotaLimitsForClass(quotaClass.Spec.ClassID)

	// ---- 4. ResourceQuota を apply する（Create or Update）----

	// ResourceQuota の名前を設定する（QuotaClass 名に "-quota" サフィックスを付ける）
	rqName := fmt.Sprintf("%s-quota", quotaClass.Name)
	// 既存の ResourceQuota を取得する
	var existingRQ corev1.ResourceQuota
	// API サーバから ResourceQuota を取得する（存在しなければ新規作成する）
	rqErr := r.Get(ctx, client.ObjectKey{Namespace: req.Namespace, Name: rqName}, &existingRQ)

	// 新規作成対象の ResourceQuota を構築する
	desiredRQ := corev1.ResourceQuota{
		// TypeMeta を設定する
		TypeMeta: metav1.TypeMeta{
			// Kind を設定する
			Kind: "ResourceQuota",
			// API バージョンを設定する
			APIVersion: "v1",
		},
		// ObjectMeta を設定する
		ObjectMeta: metav1.ObjectMeta{
			// 名前を設定する
			Name: rqName,
			// namespace を設定する
			Namespace: req.Namespace,
			// owner reference を設定して QuotaClass 削除時に ResourceQuota も削除されるようにする
			OwnerReferences: []metav1.OwnerReference{
				{
					// API バージョンを設定する
					APIVersion: "k1s0.io/v1",
					// Kind を設定する
					Kind: "QuotaClass",
					// 名前を設定する
					Name: quotaClass.Name,
					// UID を設定する
					UID: quotaClass.UID,
					// GC で自動削除されるよう controller フラグを立てる
					Controller: func() *bool { b := true; return &b }(),
					// BlockOwnerDeletion を true に設定する
					BlockOwnerDeletion: func() *bool { b := true; return &b }(),
				},
			},
		},
		// Spec に hard limit を設定する
		Spec: corev1.ResourceQuotaSpec{
			// Hard は namespace 全体の ResourceQuota hard limit
			Hard: corev1.ResourceList{
				// CPU 上限を設定する
				corev1.ResourceCPU: resource.MustParse(cpuLimit),
				// Memory 上限を設定する
				corev1.ResourceMemory: resource.MustParse(memLimit),
				// Storage 上限を設定する（PVC の合計）
				corev1.ResourceRequestsStorage: resource.MustParse(storageLimit),
				// Pod 数上限を設定する
				corev1.ResourcePods: resource.MustParse(podLimit),
			},
		},
	}

	// ResourceQuota が存在しない場合は新規作成する
	if apierrors.IsNotFound(rqErr) {
		// 新規 ResourceQuota を作成する
		if err := r.Create(ctx, &desiredRQ); err != nil {
			// 作成失敗をログに記録してリトライさせる
			logger.Error(err, "failed to create ResourceQuota", "name", rqName)
			// エラーを返してリトライを促す
			return reconcile.Result{}, fmt.Errorf("create ResourceQuota %s: %w", rqName, err)
		}
		// 作成成功をログに記録する
		logger.Info("ResourceQuota created", "name", rqName, "cpu", cpuLimit, "memory", memLimit)
	} else if rqErr == nil {
		// 既存の ResourceQuota の Hard limit を更新する
		existingRQ.Spec.Hard = desiredRQ.Spec.Hard
		// API サーバに更新を送信する
		if err := r.Update(ctx, &existingRQ); err != nil {
			// 更新失敗をログに記録してリトライさせる
			logger.Error(err, "failed to update ResourceQuota", "name", rqName)
			// エラーを返してリトライを促す
			return reconcile.Result{}, fmt.Errorf("update ResourceQuota %s: %w", rqName, err)
		}
		// 更新成功をログに記録する
		logger.Info("ResourceQuota updated", "name", rqName, "cpu", cpuLimit, "memory", memLimit)
	} else {
		// Get 呼び出し自体のエラーをラップして返す
		return reconcile.Result{}, fmt.Errorf("get ResourceQuota %s: %w", rqName, rqErr)
	}

	// ---- 5. LimitRange を apply する（Create or Update）----

	// LimitRange の名前を設定する（QuotaClass 名に "-limitrange" サフィックスを付ける）
	lrName := fmt.Sprintf("%s-limitrange", quotaClass.Name)
	// 既存の LimitRange を取得する
	var existingLR corev1.LimitRange
	// API サーバから LimitRange を取得する
	lrErr := r.Get(ctx, client.ObjectKey{Namespace: req.Namespace, Name: lrName}, &existingLR)

	// コンテナ単体のデフォルト limit を計算する（quota の 1/10 を default request にする）
	containerDefaultCPU := "100m"
	// コンテナデフォルトメモリリクエスト
	containerDefaultMem := "128Mi"
	// quota_class に応じてコンテナデフォルト値を調整する
	switch quotaClass.Spec.ClassID {
	// v1_business_dedicated と v1_enterprise_dedicated はデフォルト値を大きくする
	case tier1v1.V1QuotaBusinessDedicated, tier1v1.V1QuotaEnterpriseDedicated:
		// ビジネス / エンタープライズはコンテナデフォルトを大きくする
		containerDefaultCPU = "500m"
		// ビジネス / エンタープライズのデフォルト Memory を設定する
		containerDefaultMem = "512Mi"
	}

	// 新規作成対象の LimitRange を構築する
	desiredLR := corev1.LimitRange{
		// TypeMeta を設定する
		TypeMeta: metav1.TypeMeta{
			// Kind を設定する
			Kind: "LimitRange",
			// API バージョンを設定する
			APIVersion: "v1",
		},
		// ObjectMeta を設定する（OwnerReference で QuotaClass に紐付ける）
		ObjectMeta: metav1.ObjectMeta{
			// 名前を設定する
			Name: lrName,
			// namespace を設定する
			Namespace: req.Namespace,
			// owner reference を設定する
			OwnerReferences: []metav1.OwnerReference{
				{
					// API バージョンを設定する
					APIVersion: "k1s0.io/v1",
					// Kind を設定する
					Kind: "QuotaClass",
					// 名前を設定する
					Name: quotaClass.Name,
					// UID を設定する
					UID: quotaClass.UID,
					// GC フラグを立てる
					Controller: func() *bool { b := true; return &b }(),
					// BlockOwnerDeletion を true に設定する
					BlockOwnerDeletion: func() *bool { b := true; return &b }(),
				},
			},
		},
		// Spec に LimitRange アイテムを設定する
		Spec: corev1.LimitRangeSpec{
			// Limits にコンテナ単体の制限を設定する
			Limits: []corev1.LimitRangeItem{
				{
					// Type に Container を設定する（Pod 内の各コンテナに適用する）
					Type: corev1.LimitTypeContainer,
					// Default は limit 未指定時に適用されるデフォルト値
					Default: corev1.ResourceList{
						// CPU のデフォルト limit を設定する
						corev1.ResourceCPU: resource.MustParse(cpuLimit),
						// Memory のデフォルト limit を設定する
						corev1.ResourceMemory: resource.MustParse(memLimit),
					},
					// DefaultRequest は request 未指定時のデフォルト値
					DefaultRequest: corev1.ResourceList{
						// CPU のデフォルト request を設定する
						corev1.ResourceCPU: resource.MustParse(containerDefaultCPU),
						// Memory のデフォルト request を設定する
						corev1.ResourceMemory: resource.MustParse(containerDefaultMem),
					},
				},
			},
		},
	}

	// LimitRange が存在しない場合は新規作成する
	if apierrors.IsNotFound(lrErr) {
		// 新規 LimitRange を作成する
		if err := r.Create(ctx, &desiredLR); err != nil {
			// 作成失敗をログに記録してリトライさせる
			logger.Error(err, "failed to create LimitRange", "name", lrName)
			// エラーを返してリトライを促す
			return reconcile.Result{}, fmt.Errorf("create LimitRange %s: %w", lrName, err)
		}
		// 作成成功をログに記録する
		logger.Info("LimitRange created", "name", lrName)
	} else if lrErr == nil {
		// 既存の LimitRange の Spec を更新する
		existingLR.Spec = desiredLR.Spec
		// API サーバに更新を送信する
		if err := r.Update(ctx, &existingLR); err != nil {
			// 更新失敗をログに記録してリトライさせる
			logger.Error(err, "failed to update LimitRange", "name", lrName)
			// エラーを返してリトライを促す
			return reconcile.Result{}, fmt.Errorf("update LimitRange %s: %w", lrName, err)
		}
		// 更新成功をログに記録する
		logger.Info("LimitRange updated", "name", lrName)
	} else {
		// Get 呼び出し自体のエラーをラップして返す
		return reconcile.Result{}, fmt.Errorf("get LimitRange %s: %w", lrName, lrErr)
	}

	// ---- 6. Status.Applied を更新する ----

	// quota enforcement が完了したので applied = true に設定する
	quotaClass.Status.Applied = true
	// 現在時刻を LastUpdated に設定する（status 記録目的なので wall-clock 許可）
	now := metav1.Now()
	// LastUpdated ポインタを設定する
	quotaClass.Status.LastUpdated = &now

	// Status サブリソースを更新する
	if err := r.Status().Update(ctx, &quotaClass); err != nil {
		// Status 更新失敗をログに記録してリトライさせる
		logger.Error(err, "Failed to update QuotaClass status")
		// エラーを返してリトライを促す
		return reconcile.Result{}, fmt.Errorf("QuotaClass status update failed: %w", err)
	}

	// Reconcile 完了をログに記録する
	logger.Info("QuotaClass reconciled successfully — ResourceQuota and LimitRange applied",
		"name", req.Name,
		"classId", quotaClass.Spec.ClassID,
		"cpuLimit", cpuLimit,
		"memLimit", memLimit,
		"storageLimit", storageLimit,
		"podLimit", podLimit,
	)

	// 1 時間後に再 Reconcile をスケジュールする（quota 設定の定期チェック）
	return reconcile.Result{RequeueAfter: 1 * time.Hour}, nil
}
