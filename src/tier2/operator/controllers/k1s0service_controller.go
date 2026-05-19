// k1s0service_controller.go — k1s0 tier2 operator: K1s0Service Reconciler
// tier2 適合仕様 10 テナント分離: K1s0Service CRD を監視し、必要な Kubernetes リソースを自動生成・同期する。
// Deployment / Service / ServiceAccount / NetworkPolicy / HPA / Ingress の 6 リソースを管理する。

// パッケージ名: controllers（internal ではなく controllers: Kubebuilder 標準ディレクトリ名に従う）
package controllers

import (
	// context パッケージのインポート: Reconcile コンテキスト管理に使用する
	"context"
	// fmt パッケージのインポート: エラーメッセージのフォーマットに使用する
	"fmt"
	// time パッケージのインポート: RequeueAfter の指定に使用する
	"time"

	// Kubernetes apps/v1 API のインポート: Deployment に使用する
	appsv1 "k8s.io/api/apps/v1"
	// Kubernetes autoscaling/v2 API のインポート: HPA に使用する
	autoscalingv2 "k8s.io/api/autoscaling/v2"
	// Kubernetes core/v1 API のインポート: Service / ServiceAccount / NetworkPolicy に使用する
	corev1 "k8s.io/api/core/v1"
	// Kubernetes networking/v1 API のインポート: NetworkPolicy / Ingress に使用する
	networkingv1 "k8s.io/api/networking/v1"
	// Kubernetes resource.Quantity 型のインポート: CPU / Memory 上限値に使用する
	"k8s.io/apimachinery/pkg/api/resource"
	// Kubernetes errors パッケージのインポート: IsNotFound エラー判定に使用する
	apierrors "k8s.io/apimachinery/pkg/api/errors"
	// Kubernetes API マシナリーのインポート: metav1 / intstr に使用する
	metav1 "k8s.io/apimachinery/pkg/apis/meta/v1"
	// intstr パッケージのインポート: IntOrString 型に使用する
	"k8s.io/apimachinery/pkg/util/intstr"
	// controller-runtime クライアントのインポート: API サーバとの通信に使用する
	"sigs.k8s.io/controller-runtime/pkg/client"
	// controller-runtime ログのインポート: 構造化ログに使用する
	"sigs.k8s.io/controller-runtime/pkg/log"
	// controller-runtime reconcile のインポート: reconcile.Result / Request 型に使用する
	"sigs.k8s.io/controller-runtime/pkg/reconcile"
)

// K1s0ServicePhase は K1s0Service の処理フェーズを表す文字列型エイリアス
type K1s0ServicePhase string

const (
	// PhaseProvisioning は初期プロビジョニング中フェーズを表す
	PhaseProvisioning K1s0ServicePhase = "Provisioning"
	// PhaseActive はサービスが正常稼働中のフェーズを表す
	PhaseActive K1s0ServicePhase = "Active"
	// PhaseSuspended はサービスが一時停止中のフェーズを表す
	PhaseSuspended K1s0ServicePhase = "Suspended"
	// PhaseTerminating はサービス終了処理中のフェーズを表す
	PhaseTerminating K1s0ServicePhase = "Terminating"
)

// K1s0ServiceSpec は K1s0Service CRD の spec フィールドを表す構造体
// k1s0service_crd.yaml の openAPIV3Schema.properties.spec に対応する
type K1s0ServiceSpec struct {
	// TenantId はテナント識別子（UUID v4 形式 / Keycloak テナント ID と一致する）
	TenantId string `json:"tenantId"`
	// QuotaClass はクォータクラス名（standard / enterprise / unlimited）
	QuotaClass string `json:"quotaClass"`
	// FeatureFlags はテナントごとのフィーチャーフラグマップ（key: フラグ名 / value: 有効無効）
	FeatureFlags map[string]bool `json:"featureFlags,omitempty"`
}

// K1s0ServiceStatus は K1s0Service CRD の status フィールドを表す構造体
// k1s0service_crd.yaml の openAPIV3Schema.properties.status に対応する
type K1s0ServiceStatus struct {
	// Phase は Operator の処理フェーズ（Provisioning / Active / Suspended / Terminating）
	Phase string `json:"phase,omitempty"`
	// LastProcessedHlc は最終処理 HLC タイムスタンプ（wall clock TTL 禁止規約により HLC を使用する）
	LastProcessedHlc string `json:"lastProcessedHlc,omitempty"`
}

// K1s0Service は K1s0Service CRD を表す構造体（controller-runtime Object を実装する）
type K1s0Service struct {
	// TypeMeta は Kubernetes リソースの API バージョンと Kind を保持する
	metav1.TypeMeta `json:",inline"`
	// ObjectMeta は Kubernetes リソースのメタデータを保持する
	metav1.ObjectMeta `json:"metadata,omitempty"`
	// Spec は K1s0Service の desired state を保持する
	Spec K1s0ServiceSpec `json:"spec,omitempty"`
	// Status は K1s0Service の actual state を保持する
	Status K1s0ServiceStatus `json:"status,omitempty"`
}

// K1s0ServiceList は K1s0Service のリストリソースを表す構造体
type K1s0ServiceList struct {
	// TypeMeta は Kubernetes リソースの API バージョンと Kind を保持する
	metav1.TypeMeta `json:",inline"`
	// ListMeta は Kubernetes リストリソースのメタデータを保持する
	metav1.ListMeta `json:"metadata,omitempty"`
	// Items は K1s0Service のリストを保持する
	Items []K1s0Service `json:"items"`
}

// DeepCopyObject は runtime.Object インターフェースの実装（controller-runtime が要求する）
func (in *K1s0Service) DeepCopyObject() interface{} {
	// K1s0Service のディープコピーを返す
	out := new(K1s0Service)
	// TypeMeta をコピーする
	out.TypeMeta = in.TypeMeta
	// ObjectMeta をコピーする
	out.ObjectMeta = *in.ObjectMeta.DeepCopy()
	// Spec をコピーする
	out.Spec = in.Spec
	// FeatureFlags マップを個別にコピーする
	if in.Spec.FeatureFlags != nil {
		// 新しいマップを作成してコピーする
		out.Spec.FeatureFlags = make(map[string]bool, len(in.Spec.FeatureFlags))
		// 各エントリをコピーする
		for k, v := range in.Spec.FeatureFlags {
			// キーと値をコピーする
			out.Spec.FeatureFlags[k] = v
		}
	}
	// Status をコピーする
	out.Status = in.Status
	// ディープコピーを返す
	return out
}

// DeepCopyObject は K1s0ServiceList の runtime.Object インターフェースの実装
func (in *K1s0ServiceList) DeepCopyObject() interface{} {
	// K1s0ServiceList のディープコピーを返す
	out := new(K1s0ServiceList)
	// TypeMeta をコピーする
	out.TypeMeta = in.TypeMeta
	// ListMeta をコピーする
	out.ListMeta = in.ListMeta
	// Items スライスを個別にコピーする
	if in.Items != nil {
		// 新しいスライスを作成する
		out.Items = make([]K1s0Service, len(in.Items))
		// 各 K1s0Service をコピーする
		for i := range in.Items {
			// DeepCopyObject を使って各要素をコピーする
			out.Items[i] = *in.Items[i].DeepCopyObject().(*K1s0Service)
		}
	}
	// ディープコピーを返す
	return out
}

// K1s0ServiceReconciler は K1s0Service リソースを reconcile するコントローラ構造体
type K1s0ServiceReconciler struct {
	// Kubernetes クライアント: API サーバとのリソース読み書きに使用する
	client.Client
}

// replicasForQuotaClass は quotaClass に応じたレプリカ数を返す
// テナント分離適合仕様 §quota-class-replica-map に準拠する
func replicasForQuotaClass(quotaClass string) int32 {
	// quotaClass に応じてレプリカ数を決定する
	switch quotaClass {
	// standard: 最小 2 レプリカ（可用性確保）
	case "standard":
		// standard は 2 レプリカ
		return 2
	// enterprise: 3 レプリカ（高可用性）
	case "enterprise":
		// enterprise は 3 レプリカ
		return 3
	// unlimited: 3 レプリカ（上限なしクラスは HPA でスケールアウトする）
	case "unlimited":
		// unlimited は最低 3 レプリカを確保する
		return 3
	// 不明なクラスは安全側の最小レプリカにフォールバックする
	default:
		// デフォルトは 2 レプリカ
		return 2
	}
}

// resourceLimitsForQuotaClass は quotaClass に応じたリソース制限を返す
// テナント分離適合仕様 §quota-class-resource-map に準拠する
func resourceLimitsForQuotaClass(quotaClass string) (cpuReq, cpuLimit, memReq, memLimit string) {
	// quotaClass に応じてリソース制限を決定する
	switch quotaClass {
	// standard: 控えめなリソース制限
	case "standard":
		// CPU 要求 100m / 制限 500m / Memory 要求 128Mi / 制限 512Mi
		return "100m", "500m", "128Mi", "512Mi"
	// enterprise: 余裕のあるリソース制限
	case "enterprise":
		// CPU 要求 250m / 制限 2 / Memory 要求 512Mi / 制限 2Gi
		return "250m", "2", "512Mi", "2Gi"
	// unlimited: 大きなリソース制限（HPA で自動スケール）
	case "unlimited":
		// CPU 要求 500m / 制限 4 / Memory 要求 1Gi / 制限 4Gi
		return "500m", "4", "1Gi", "4Gi"
	// 不明なクラスは standard 相当の最小値にフォールバックする
	default:
		// デフォルトは standard 相当のリソース制限
		return "100m", "500m", "128Mi", "512Mi"
	}
}

// buildOwnerRef は K1s0Service を owner として OwnerReference を生成する
// GC によるカスケード削除を有効にするために controller フラグを立てる
func buildOwnerRef(svc *K1s0Service) metav1.OwnerReference {
	// controller フラグ用のポインタを生成する
	isController := true
	// blockOwnerDeletion フラグ用のポインタを生成する
	blockOwnerDeletion := true
	// OwnerReference を構築して返す
	return metav1.OwnerReference{
		// API バージョンを設定する
		APIVersion: "k1s0.io/v1alpha1",
		// Kind を設定する
		Kind: "K1s0Service",
		// 名前を設定する
		Name: svc.Name,
		// UID を設定する
		UID: svc.UID,
		// controller フラグを立てる（GC の対象にする）
		Controller: &isController,
		// BlockOwnerDeletion を有効にする
		BlockOwnerDeletion: &blockOwnerDeletion,
	}
}

// reconcileServiceAccount は K1s0Service に対応する ServiceAccount を作成または確認する
func (r *K1s0ServiceReconciler) reconcileServiceAccount(ctx context.Context, svc *K1s0Service) error {
	// ロガーをコンテキストから取得する
	logger := log.FromContext(ctx)
	// ServiceAccount 名を K1s0Service 名から生成する
	saName := fmt.Sprintf("%s-sa", svc.Name)
	// 既存の ServiceAccount を取得する
	var existingSA corev1.ServiceAccount
	// API サーバから ServiceAccount を取得する
	err := r.Get(ctx, client.ObjectKey{Namespace: svc.Namespace, Name: saName}, &existingSA)
	// ServiceAccount が存在しない場合は新規作成する
	if apierrors.IsNotFound(err) {
		// 新規 ServiceAccount を構築する
		sa := corev1.ServiceAccount{
			// TypeMeta を設定する
			ObjectMeta: metav1.ObjectMeta{
				// ServiceAccount 名を設定する
				Name: saName,
				// Namespace を設定する
				Namespace: svc.Namespace,
				// K1s0Service を owner に設定する（cascade delete を有効にする）
				OwnerReferences: []metav1.OwnerReference{buildOwnerRef(svc)},
				// テナント ID ラベルを付与する（NetworkPolicy のセレクタで使用する）
				Labels: map[string]string{
					// テナント ID ラベル
					"app.tenant-id": svc.Spec.TenantId,
					// K1s0Service 名ラベル
					"app.k1s0service": svc.Name,
				},
			},
		}
		// ServiceAccount を作成する
		if createErr := r.Create(ctx, &sa); createErr != nil {
			// 作成失敗をログに記録してエラーを返す
			logger.Error(createErr, "ServiceAccount 作成失敗", "name", saName)
			// エラーをラップして返す
			return fmt.Errorf("ServiceAccount %s 作成失敗: %w", saName, createErr)
		}
		// 作成成功をログに記録する
		logger.Info("ServiceAccount 作成完了", "name", saName)
	} else if err != nil {
		// Get エラーをラップして返す
		return fmt.Errorf("ServiceAccount %s 取得失敗: %w", saName, err)
	}
	// 既存の ServiceAccount はそのまま使用する
	return nil
}

// reconcileDeployment は K1s0Service に対応する Deployment を作成または更新する
func (r *K1s0ServiceReconciler) reconcileDeployment(ctx context.Context, svc *K1s0Service) error {
	// ロガーをコンテキストから取得する
	logger := log.FromContext(ctx)
	// Deployment 名を K1s0Service 名から生成する
	deployName := svc.Name
	// quotaClass に応じたレプリカ数を取得する
	replicas := replicasForQuotaClass(svc.Spec.QuotaClass)
	// quotaClass に応じたリソース制限を取得する
	cpuReq, cpuLim, memReq, memLim := resourceLimitsForQuotaClass(svc.Spec.QuotaClass)
	// ServiceAccount 名を生成する（reconcileServiceAccount と同じ命名規則を使用する）
	saName := fmt.Sprintf("%s-sa", svc.Name)

	// desired Deployment を構築する
	desired := appsv1.Deployment{
		// TypeMeta を設定する
		TypeMeta: metav1.TypeMeta{
			// Kind を設定する
			Kind: "Deployment",
			// API バージョンを設定する
			APIVersion: "apps/v1",
		},
		// ObjectMeta を設定する
		ObjectMeta: metav1.ObjectMeta{
			// Deployment 名を設定する
			Name: deployName,
			// Namespace を設定する
			Namespace: svc.Namespace,
			// K1s0Service を owner に設定する（cascade delete を有効にする）
			OwnerReferences: []metav1.OwnerReference{buildOwnerRef(svc)},
			// テナント ID と K1s0Service 名のラベルを付与する
			Labels: map[string]string{
				// テナント ID ラベル
				"app.tenant-id": svc.Spec.TenantId,
				// K1s0Service 名ラベル
				"app.k1s0service": svc.Name,
			},
		},
		Spec: appsv1.DeploymentSpec{
			// レプリカ数を設定する
			Replicas: &replicas,
			// Pod セレクタを設定する（Deployment が管理する Pod を識別する）
			Selector: &metav1.LabelSelector{
				MatchLabels: map[string]string{
					// K1s0Service 名でセレクタを設定する
					"app.k1s0service": svc.Name,
				},
			},
			// Pod テンプレートを設定する
			Template: corev1.PodTemplateSpec{
				// Pod ラベルを設定する（selector.matchLabels と一致させる）
				ObjectMeta: metav1.ObjectMeta{
					Labels: map[string]string{
						// テナント ID ラベル
						"app.tenant-id": svc.Spec.TenantId,
						// K1s0Service 名ラベル
						"app.k1s0service": svc.Name,
					},
				},
				Spec: corev1.PodSpec{
					// ServiceAccount を指定する（RBAC 権限を分離する）
					ServiceAccountName: saName,
					// 非 root ユーザーでの実行を強制する（production / development 区別禁止規約準拠）
					SecurityContext: &corev1.PodSecurityContext{
						// 非 root ユーザーで実行する
						RunAsNonRoot: func() *bool { b := true; return &b }(),
						// 実行ユーザー ID（1000 番台の一般ユーザー）
						RunAsUser: func() *int64 { uid := int64(1000); return &uid }(),
						// ファイルシステムグループ ID を設定する
						FSGroup: func() *int64 { gid := int64(1000); return &gid }(),
					},
					Containers: []corev1.Container{
						{
							// メインコンテナ名を設定する
							Name: "tier2-api",
							// コンテナイメージを設定する（Harbor 内部レジストリを使用する）
							Image: "harbor.k1s0.internal/k1s0/tier2:latest",
							// イメージプルポリシーを設定する
							ImagePullPolicy: corev1.PullIfNotPresent,
							// コンテナがリッスンするポートを設定する
							Ports: []corev1.ContainerPort{
								{
									// HTTP API ポート
									Name: "http",
									// ポート番号を設定する
									ContainerPort: 8080,
									// TCP プロトコルを指定する
									Protocol: corev1.ProtocolTCP,
								},
							},
							// Liveness プローブを設定する（Pod が生存していることを確認する）
							LivenessProbe: &corev1.Probe{
								ProbeHandler: corev1.ProbeHandler{
									HTTPGet: &corev1.HTTPGetAction{
										// ヘルスチェックエンドポイントを設定する
										Path: "/healthz",
										// ポート名を参照する
										Port: intstr.FromString("http"),
									},
								},
								// 初期遅延（コンテナ起動完了を待つ）
								InitialDelaySeconds: 10,
								// チェック周期を設定する
								PeriodSeconds: 10,
							},
							// Readiness プローブを設定する（Pod がトラフィックを受け入れる準備ができているか確認する）
							ReadinessProbe: &corev1.Probe{
								ProbeHandler: corev1.ProbeHandler{
									HTTPGet: &corev1.HTTPGetAction{
										// 準備完了チェックエンドポイントを設定する
										Path: "/readyz",
										// ポート名を参照する
										Port: intstr.FromString("http"),
									},
								},
								// 初期遅延を設定する
								InitialDelaySeconds: 5,
								// チェック周期を設定する
								PeriodSeconds: 5,
							},
							// リソース制限を設定する（quotaClass に応じた値を使用する）
							Resources: corev1.ResourceRequirements{
								Requests: corev1.ResourceList{
									// CPU 要求値を設定する
									corev1.ResourceCPU: resource.MustParse(cpuReq),
									// Memory 要求値を設定する
									corev1.ResourceMemory: resource.MustParse(memReq),
								},
								Limits: corev1.ResourceList{
									// CPU 上限を設定する
									corev1.ResourceCPU: resource.MustParse(cpuLim),
									// Memory 上限を設定する
									corev1.ResourceMemory: resource.MustParse(memLim),
								},
							},
							// TLS 証明書マウントを設定する
							VolumeMounts: []corev1.VolumeMount{
								{
									// TLS 証明書ボリューム名
									Name: "tls-certs",
									// TLS 証明書マウントパスを設定する
									MountPath: "/etc/tls",
									// 読み取り専用マウントにする
									ReadOnly: true,
								},
							},
						},
					},
					// Pod にマウントするボリュームを定義する
					Volumes: []corev1.Volume{
						{
							// TLS 証明書ボリューム名
							Name: "tls-certs",
							VolumeSource: corev1.VolumeSource{
								// Secret から TLS 証明書をマウントする
								Secret: &corev1.SecretVolumeSource{
									// TLS 証明書 Secret 名（K1s0Service 名に -tls サフィックスを付ける）
									SecretName: fmt.Sprintf("%s-tls", svc.Name),
									// Secret が存在しない場合は optional にする
									Optional: func() *bool { b := true; return &b }(),
								},
							},
						},
					},
				},
			},
		},
	}

	// 既存の Deployment を取得する
	var existing appsv1.Deployment
	// API サーバから Deployment を取得する
	getErr := r.Get(ctx, client.ObjectKey{Namespace: svc.Namespace, Name: deployName}, &existing)
	// Deployment が存在しない場合は新規作成する
	if apierrors.IsNotFound(getErr) {
		// 新規 Deployment を作成する
		if createErr := r.Create(ctx, &desired); createErr != nil {
			// 作成失敗をログに記録してエラーを返す
			logger.Error(createErr, "Deployment 作成失敗", "name", deployName)
			// エラーをラップして返す
			return fmt.Errorf("Deployment %s 作成失敗: %w", deployName, createErr)
		}
		// 作成成功をログに記録する
		logger.Info("Deployment 作成完了", "name", deployName, "replicas", replicas)
	} else if getErr == nil {
		// 既存 Deployment のレプリカ数とリソース制限を更新する
		existing.Spec.Replicas = desired.Spec.Replicas
		// コンテナのリソース制限を更新する
		if len(existing.Spec.Template.Spec.Containers) > 0 {
			// 最初のコンテナ（tier2-api）のリソースを更新する
			existing.Spec.Template.Spec.Containers[0].Resources = desired.Spec.Template.Spec.Containers[0].Resources
		}
		// API サーバに更新を送信する
		if updateErr := r.Update(ctx, &existing); updateErr != nil {
			// 更新失敗をログに記録してエラーを返す
			logger.Error(updateErr, "Deployment 更新失敗", "name", deployName)
			// エラーをラップして返す
			return fmt.Errorf("Deployment %s 更新失敗: %w", deployName, updateErr)
		}
		// 更新成功をログに記録する
		logger.Info("Deployment 更新完了", "name", deployName, "replicas", replicas)
	} else {
		// Get エラーをラップして返す
		return fmt.Errorf("Deployment %s 取得失敗: %w", deployName, getErr)
	}
	// 正常終了を返す
	return nil
}

// reconcileService は K1s0Service に対応する Kubernetes Service を作成または確認する
func (r *K1s0ServiceReconciler) reconcileService(ctx context.Context, svc *K1s0Service) error {
	// ロガーをコンテキストから取得する
	logger := log.FromContext(ctx)
	// Service 名を K1s0Service 名から生成する
	svcName := svc.Name
	// desired Service を構築する
	desired := corev1.Service{
		// TypeMeta を設定する
		TypeMeta: metav1.TypeMeta{
			// Kind を設定する
			Kind: "Service",
			// API バージョンを設定する
			APIVersion: "v1",
		},
		// ObjectMeta を設定する
		ObjectMeta: metav1.ObjectMeta{
			// Service 名を設定する
			Name: svcName,
			// Namespace を設定する
			Namespace: svc.Namespace,
			// K1s0Service を owner に設定する
			OwnerReferences: []metav1.OwnerReference{buildOwnerRef(svc)},
			// テナント ID ラベルを付与する
			Labels: map[string]string{
				// テナント ID ラベル
				"app.tenant-id": svc.Spec.TenantId,
				// K1s0Service 名ラベル
				"app.k1s0service": svc.Name,
			},
		},
		Spec: corev1.ServiceSpec{
			// ClusterIP 型: クラスター内部からのみアクセス可能にする
			Type: corev1.ServiceTypeClusterIP,
			// セレクタを設定する（Deployment の Pod を選択する）
			Selector: map[string]string{
				// K1s0Service 名でセレクタを設定する
				"app.k1s0service": svc.Name,
			},
			// 公開ポートを設定する
			Ports: []corev1.ServicePort{
				{
					// HTTP API ポート名
					Name: "http",
					// ポート番号を設定する
					Port: 8080,
					// コンテナのターゲットポートを設定する
					TargetPort: intstr.FromString("http"),
					// TCP プロトコルを指定する
					Protocol: corev1.ProtocolTCP,
				},
			},
		},
	}

	// 既存の Service を取得する
	var existing corev1.Service
	// API サーバから Service を取得する
	getErr := r.Get(ctx, client.ObjectKey{Namespace: svc.Namespace, Name: svcName}, &existing)
	// Service が存在しない場合は新規作成する
	if apierrors.IsNotFound(getErr) {
		// 新規 Service を作成する
		if createErr := r.Create(ctx, &desired); createErr != nil {
			// 作成失敗をログに記録してエラーを返す
			logger.Error(createErr, "Service 作成失敗", "name", svcName)
			// エラーをラップして返す
			return fmt.Errorf("Service %s 作成失敗: %w", svcName, createErr)
		}
		// 作成成功をログに記録する
		logger.Info("Service 作成完了", "name", svcName)
	} else if getErr != nil {
		// Get エラーをラップして返す
		return fmt.Errorf("Service %s 取得失敗: %w", svcName, getErr)
	}
	// 既存の Service は selector / port 変更が少ないためそのまま使用する
	return nil
}

// reconcileNetworkPolicy は K1s0Service に対応する NetworkPolicy を作成または確認する
// 同一 namespace からの ingress のみを許可してテナント分離を実現する
func (r *K1s0ServiceReconciler) reconcileNetworkPolicy(ctx context.Context, svc *K1s0Service) error {
	// ロガーをコンテキストから取得する
	logger := log.FromContext(ctx)
	// NetworkPolicy 名を K1s0Service 名から生成する
	npName := fmt.Sprintf("%s-netpol", svc.Name)
	// desired NetworkPolicy を構築する
	desired := networkingv1.NetworkPolicy{
		// TypeMeta を設定する
		TypeMeta: metav1.TypeMeta{
			// Kind を設定する
			Kind: "NetworkPolicy",
			// API バージョンを設定する
			APIVersion: "networking.k8s.io/v1",
		},
		// ObjectMeta を設定する
		ObjectMeta: metav1.ObjectMeta{
			// NetworkPolicy 名を設定する
			Name: npName,
			// Namespace を設定する
			Namespace: svc.Namespace,
			// K1s0Service を owner に設定する
			OwnerReferences: []metav1.OwnerReference{buildOwnerRef(svc)},
			// テナント ID ラベルを付与する（cross-tenant-query-deny ポリシーのセレクタと一致させる）
			Labels: map[string]string{
				// テナント ID ラベル（Kyverno ポリシーの検証対象となる）
				"app.tenant-id": svc.Spec.TenantId,
				// K1s0Service 名ラベル
				"app.k1s0service": svc.Name,
			},
		},
		Spec: networkingv1.NetworkPolicySpec{
			// 対象 Pod を K1s0Service 名でセレクタ指定する
			PodSelector: metav1.LabelSelector{
				MatchLabels: map[string]string{
					// K1s0Service 名ラベルで Pod を選択する
					"app.k1s0service": svc.Name,
				},
			},
			// Ingress ルール（同一 namespace からのアクセスのみ許可する）
			Ingress: []networkingv1.NetworkPolicyIngressRule{
				{
					// 同一 namespace 内の Pod からの ingress を許可する
					From: []networkingv1.NetworkPolicyPeer{
						{
							// namespaceSelector を空にすると同一 namespace が選択される
							PodSelector: &metav1.LabelSelector{},
						},
					},
					// HTTP API ポートへのアクセスのみ許可する
					Ports: []networkingv1.NetworkPolicyPort{
						{
							// TCP プロトコルを指定する
							Protocol: func() *corev1.Protocol { p := corev1.ProtocolTCP; return &p }(),
							// ポート 8080 を許可する
							Port: func() *intstr.IntOrString { p := intstr.FromInt(8080); return &p }(),
						},
					},
				},
			},
			// PolicyTypes にIngress を指定する（Egress は制限しない）
			PolicyTypes: []networkingv1.PolicyType{
				// Ingress トラフィックを NetworkPolicy で制御する
				networkingv1.PolicyTypeIngress,
			},
		},
	}

	// 既存の NetworkPolicy を取得する
	var existing networkingv1.NetworkPolicy
	// API サーバから NetworkPolicy を取得する
	getErr := r.Get(ctx, client.ObjectKey{Namespace: svc.Namespace, Name: npName}, &existing)
	// NetworkPolicy が存在しない場合は新規作成する
	if apierrors.IsNotFound(getErr) {
		// 新規 NetworkPolicy を作成する
		if createErr := r.Create(ctx, &desired); createErr != nil {
			// 作成失敗をログに記録してエラーを返す
			logger.Error(createErr, "NetworkPolicy 作成失敗", "name", npName)
			// エラーをラップして返す
			return fmt.Errorf("NetworkPolicy %s 作成失敗: %w", npName, createErr)
		}
		// 作成成功をログに記録する
		logger.Info("NetworkPolicy 作成完了", "name", npName)
	} else if getErr != nil {
		// Get エラーをラップして返す
		return fmt.Errorf("NetworkPolicy %s 取得失敗: %w", npName, getErr)
	}
	// 既存の NetworkPolicy はそのまま使用する
	return nil
}

// reconcileHPA は K1s0Service に対応する HorizontalPodAutoscaler を作成または更新する
func (r *K1s0ServiceReconciler) reconcileHPA(ctx context.Context, svc *K1s0Service) error {
	// ロガーをコンテキストから取得する
	logger := log.FromContext(ctx)
	// HPA 名を K1s0Service 名から生成する
	hpaName := fmt.Sprintf("%s-hpa", svc.Name)
	// quotaClass に応じた最大レプリカ数を設定する
	var maxReplicas int32
	// quotaClass に応じて最大レプリカ数を決定する
	switch svc.Spec.QuotaClass {
	// standard: 最大 5 レプリカ
	case "standard":
		// standard の最大レプリカ数を設定する
		maxReplicas = 5
	// enterprise: 最大 10 レプリカ
	case "enterprise":
		// enterprise の最大レプリカ数を設定する
		maxReplicas = 10
	// unlimited: 最大 50 レプリカ
	case "unlimited":
		// unlimited の最大レプリカ数を設定する
		maxReplicas = 50
	// 不明なクラスは safe fallback の 5 レプリカ
	default:
		// デフォルトの最大レプリカ数を設定する
		maxReplicas = 5
	}
	// 最小レプリカ数は quotaClass に応じた通常レプリカ数を使用する
	minReplicas := replicasForQuotaClass(svc.Spec.QuotaClass)
	// CPU 使用率ターゲット（70% でスケールアウトする）
	cpuTarget := int32(70)

	// desired HPA を構築する
	desired := autoscalingv2.HorizontalPodAutoscaler{
		// TypeMeta を設定する
		TypeMeta: metav1.TypeMeta{
			// Kind を設定する
			Kind: "HorizontalPodAutoscaler",
			// API バージョンを設定する
			APIVersion: "autoscaling/v2",
		},
		// ObjectMeta を設定する
		ObjectMeta: metav1.ObjectMeta{
			// HPA 名を設定する
			Name: hpaName,
			// Namespace を設定する
			Namespace: svc.Namespace,
			// K1s0Service を owner に設定する
			OwnerReferences: []metav1.OwnerReference{buildOwnerRef(svc)},
			// テナント ID ラベルを付与する
			Labels: map[string]string{
				// テナント ID ラベル
				"app.tenant-id": svc.Spec.TenantId,
				// K1s0Service 名ラベル
				"app.k1s0service": svc.Name,
			},
		},
		Spec: autoscalingv2.HorizontalPodAutoscalerSpec{
			// スケール対象の Deployment を指定する
			ScaleTargetRef: autoscalingv2.CrossVersionObjectReference{
				// スケール対象の API バージョンを設定する
				APIVersion: "apps/v1",
				// スケール対象の Kind を設定する
				Kind: "Deployment",
				// スケール対象の Deployment 名を設定する（K1s0Service 名と同一）
				Name: svc.Name,
			},
			// 最小レプリカ数を設定する
			MinReplicas: &minReplicas,
			// 最大レプリカ数を設定する
			MaxReplicas: maxReplicas,
			// スケーリングメトリクスを設定する
			Metrics: []autoscalingv2.MetricSpec{
				{
					// Resource メトリクス（CPU 使用率）でスケーリングする
					Type: autoscalingv2.ResourceMetricSourceType,
					Resource: &autoscalingv2.ResourceMetricSource{
						// CPU メトリクスを指定する
						Name: corev1.ResourceCPU,
						Target: autoscalingv2.MetricTarget{
							// Utilization タイプ（Pod の CPU 使用率でスケーリングする）
							Type: autoscalingv2.UtilizationMetricType,
							// CPU 使用率ターゲットを設定する
							AverageUtilization: &cpuTarget,
						},
					},
				},
			},
		},
	}

	// 既存の HPA を取得する
	var existing autoscalingv2.HorizontalPodAutoscaler
	// API サーバから HPA を取得する
	getErr := r.Get(ctx, client.ObjectKey{Namespace: svc.Namespace, Name: hpaName}, &existing)
	// HPA が存在しない場合は新規作成する
	if apierrors.IsNotFound(getErr) {
		// 新規 HPA を作成する
		if createErr := r.Create(ctx, &desired); createErr != nil {
			// 作成失敗をログに記録してエラーを返す
			logger.Error(createErr, "HPA 作成失敗", "name", hpaName)
			// エラーをラップして返す
			return fmt.Errorf("HPA %s 作成失敗: %w", hpaName, createErr)
		}
		// 作成成功をログに記録する
		logger.Info("HPA 作成完了", "name", hpaName, "maxReplicas", maxReplicas)
	} else if getErr == nil {
		// 既存 HPA の MinReplicas / MaxReplicas / Metrics を更新する
		existing.Spec.MinReplicas = desired.Spec.MinReplicas
		// 最大レプリカ数を更新する
		existing.Spec.MaxReplicas = desired.Spec.MaxReplicas
		// メトリクスを更新する
		existing.Spec.Metrics = desired.Spec.Metrics
		// API サーバに更新を送信する
		if updateErr := r.Update(ctx, &existing); updateErr != nil {
			// 更新失敗をログに記録してエラーを返す
			logger.Error(updateErr, "HPA 更新失敗", "name", hpaName)
			// エラーをラップして返す
			return fmt.Errorf("HPA %s 更新失敗: %w", hpaName, updateErr)
		}
		// 更新成功をログに記録する
		logger.Info("HPA 更新完了", "name", hpaName, "maxReplicas", maxReplicas)
	} else {
		// Get エラーをラップして返す
		return fmt.Errorf("HPA %s 取得失敗: %w", hpaName, getErr)
	}
	// 正常終了を返す
	return nil
}

// reconcileIngress は K1s0Service に対応する Ingress を作成または更新する
func (r *K1s0ServiceReconciler) reconcileIngress(ctx context.Context, svc *K1s0Service) error {
	// ロガーをコンテキストから取得する
	logger := log.FromContext(ctx)
	// Ingress 名を K1s0Service 名から生成する
	ingName := fmt.Sprintf("%s-ingress", svc.Name)
	// Ingress ホスト名をテナント ID から生成する（テナント別サブドメイン方式）
	host := fmt.Sprintf("%s.tier2.k1s0.internal", svc.Spec.TenantId)
	// TLS Secret 名を K1s0Service 名から生成する
	tlsSecretName := fmt.Sprintf("%s-tls", svc.Name)
	// IngressClassName を設定する（nginx を使用する）
	ingressClass := "nginx"
	// pathType を設定する（Prefix マッチ）
	pathType := networkingv1.PathTypePrefix

	// desired Ingress を構築する
	desired := networkingv1.Ingress{
		// TypeMeta を設定する
		TypeMeta: metav1.TypeMeta{
			// Kind を設定する
			Kind: "Ingress",
			// API バージョンを設定する
			APIVersion: "networking.k8s.io/v1",
		},
		// ObjectMeta を設定する
		ObjectMeta: metav1.ObjectMeta{
			// Ingress 名を設定する
			Name: ingName,
			// Namespace を設定する
			Namespace: svc.Namespace,
			// K1s0Service を owner に設定する
			OwnerReferences: []metav1.OwnerReference{buildOwnerRef(svc)},
			// テナント ID ラベルを付与する
			Labels: map[string]string{
				// テナント ID ラベル
				"app.tenant-id": svc.Spec.TenantId,
				// K1s0Service 名ラベル
				"app.k1s0service": svc.Name,
			},
			// Ingress アノテーションを設定する
			Annotations: map[string]string{
				// TLS リダイレクトを有効にする
				"nginx.ingress.kubernetes.io/ssl-redirect": "true",
				// プロキシボディサイズ上限を設定する
				"nginx.ingress.kubernetes.io/proxy-body-size": "10m",
			},
		},
		Spec: networkingv1.IngressSpec{
			// IngressClass 名を設定する
			IngressClassName: &ingressClass,
			// TLS 設定を行う（tenant 別サブドメインに証明書を適用する）
			TLS: []networkingv1.IngressTLS{
				{
					// TLS 対象ホスト名を設定する
					Hosts: []string{host},
					// TLS Secret 名を設定する
					SecretName: tlsSecretName,
				},
			},
			// ルーティングルールを設定する
			Rules: []networkingv1.IngressRule{
				{
					// ホスト名を設定する
					Host: host,
					IngressRuleValue: networkingv1.IngressRuleValue{
						HTTP: &networkingv1.HTTPIngressRuleValue{
							Paths: []networkingv1.HTTPIngressPath{
								{
									// ルートパスへのルーティングを設定する
									Path: "/",
									// パスタイプを設定する
									PathType: &pathType,
									Backend: networkingv1.IngressBackend{
										Service: &networkingv1.IngressServiceBackend{
											// バックエンド Service 名を設定する（K1s0Service 名と同一）
											Name: svc.Name,
											Port: networkingv1.ServiceBackendPort{
												// バックエンドポート番号を設定する
												Number: 8080,
											},
										},
									},
								},
							},
						},
					},
				},
			},
		},
	}

	// 既存の Ingress を取得する
	var existing networkingv1.Ingress
	// API サーバから Ingress を取得する
	getErr := r.Get(ctx, client.ObjectKey{Namespace: svc.Namespace, Name: ingName}, &existing)
	// Ingress が存在しない場合は新規作成する
	if apierrors.IsNotFound(getErr) {
		// 新規 Ingress を作成する
		if createErr := r.Create(ctx, &desired); createErr != nil {
			// 作成失敗をログに記録してエラーを返す
			logger.Error(createErr, "Ingress 作成失敗", "name", ingName)
			// エラーをラップして返す
			return fmt.Errorf("Ingress %s 作成失敗: %w", ingName, createErr)
		}
		// 作成成功をログに記録する
		logger.Info("Ingress 作成完了", "name", ingName, "host", host)
	} else if getErr == nil {
		// 既存 Ingress の Rules と TLS を更新する
		existing.Spec.Rules = desired.Spec.Rules
		// TLS 設定を更新する
		existing.Spec.TLS = desired.Spec.TLS
		// API サーバに更新を送信する
		if updateErr := r.Update(ctx, &existing); updateErr != nil {
			// 更新失敗をログに記録してエラーを返す
			logger.Error(updateErr, "Ingress 更新失敗", "name", ingName)
			// エラーをラップして返す
			return fmt.Errorf("Ingress %s 更新失敗: %w", ingName, updateErr)
		}
		// 更新成功をログに記録する
		logger.Info("Ingress 更新完了", "name", ingName, "host", host)
	} else {
		// Get エラーをラップして返す
		return fmt.Errorf("Ingress %s 取得失敗: %w", ingName, getErr)
	}
	// 正常終了を返す
	return nil
}

// Reconcile は K1s0Service リソースの desired state と actual state を一致させる
// tier2 テナント分離適合仕様に準拠して 6 種類の Kubernetes リソースを管理する
func (r *K1s0ServiceReconciler) Reconcile(ctx context.Context, req reconcile.Request) (reconcile.Result, error) {
	// ロガーをコンテキストから取得する
	logger := log.FromContext(ctx)
	// Reconcile 開始をログに記録する
	logger.Info("K1s0Service Reconcile 開始", "name", req.Name, "namespace", req.Namespace)

	// ---- 1. K1s0Service リソースを API サーバから取得する ----

	// reconcile 対象の K1s0Service 変数を宣言する
	var svc K1s0Service
	// API サーバから K1s0Service を取得する
	if err := r.Get(ctx, req.NamespacedName, &svc); err != nil {
		// リソースが存在しない場合は正常終了する（削除済みの可能性）
		return reconcile.Result{}, client.IgnoreNotFound(err)
	}

	// ---- 2. TenantId と QuotaClass の検証 ----

	// TenantId が未設定の場合は警告して再試行する
	if svc.Spec.TenantId == "" {
		// TenantId 未設定を警告ログに記録する
		logger.Info("K1s0Service TenantId 未設定 — Reconcile スキップ", "name", req.Name)
		// 10 分後に再試行する
		return reconcile.Result{RequeueAfter: 10 * time.Minute}, nil
	}
	// QuotaClass が未設定の場合は警告して再試行する
	if svc.Spec.QuotaClass == "" {
		// QuotaClass 未設定を警告ログに記録する
		logger.Info("K1s0Service QuotaClass 未設定 — Reconcile スキップ", "name", req.Name)
		// 10 分後に再試行する
		return reconcile.Result{RequeueAfter: 10 * time.Minute}, nil
	}

	// ---- 3. Status を Provisioning に更新する ----

	// プロビジョニング開始を status に記録する
	svc.Status.Phase = string(PhaseProvisioning)
	// Status サブリソースを更新する（エラーは非致命的として続行する）
	if statusErr := r.Status().Update(ctx, &svc); statusErr != nil {
		// Status 更新失敗を警告ログに記録する
		logger.Info("Status Provisioning 更新失敗 — 続行する", "error", statusErr.Error())
	}

	// ---- 4. ServiceAccount を reconcile する ----

	// ServiceAccount の reconcile を実行する
	if err := r.reconcileServiceAccount(ctx, &svc); err != nil {
		// エラーをログに記録してリトライさせる
		logger.Error(err, "ServiceAccount reconcile 失敗")
		// エラーを返してリトライを促す
		return reconcile.Result{}, err
	}

	// ---- 5. Deployment を reconcile する ----

	// Deployment の reconcile を実行する
	if err := r.reconcileDeployment(ctx, &svc); err != nil {
		// エラーをログに記録してリトライさせる
		logger.Error(err, "Deployment reconcile 失敗")
		// エラーを返してリトライを促す
		return reconcile.Result{}, err
	}

	// ---- 6. Service を reconcile する ----

	// Service の reconcile を実行する
	if err := r.reconcileService(ctx, &svc); err != nil {
		// エラーをログに記録してリトライさせる
		logger.Error(err, "Service reconcile 失敗")
		// エラーを返してリトライを促す
		return reconcile.Result{}, err
	}

	// ---- 7. NetworkPolicy を reconcile する ----

	// NetworkPolicy の reconcile を実行する
	if err := r.reconcileNetworkPolicy(ctx, &svc); err != nil {
		// エラーをログに記録してリトライさせる
		logger.Error(err, "NetworkPolicy reconcile 失敗")
		// エラーを返してリトライを促す
		return reconcile.Result{}, err
	}

	// ---- 8. HPA を reconcile する ----

	// HPA の reconcile を実行する
	if err := r.reconcileHPA(ctx, &svc); err != nil {
		// エラーをログに記録してリトライさせる
		logger.Error(err, "HPA reconcile 失敗")
		// エラーを返してリトライを促す
		return reconcile.Result{}, err
	}

	// ---- 9. Ingress を reconcile する ----

	// Ingress の reconcile を実行する
	if err := r.reconcileIngress(ctx, &svc); err != nil {
		// エラーをログに記録してリトライさせる
		logger.Error(err, "Ingress reconcile 失敗")
		// エラーを返してリトライを促す
		return reconcile.Result{}, err
	}

	// ---- 10. Status を Active に更新する ----

	// 全リソースの reconcile 完了後に status を Active に更新する
	svc.Status.Phase = string(PhaseActive)
	// Status サブリソースを更新する
	if statusErr := r.Status().Update(ctx, &svc); statusErr != nil {
		// Status 更新失敗をエラーログに記録してリトライさせる
		logger.Error(statusErr, "Status Active 更新失敗")
		// エラーを返してリトライを促す
		return reconcile.Result{}, fmt.Errorf("Status Active 更新失敗: %w", statusErr)
	}

	// Reconcile 完了をログに記録する
	logger.Info("K1s0Service Reconcile 完了",
		"name", req.Name,
		"tenantId", svc.Spec.TenantId,
		"quotaClass", svc.Spec.QuotaClass,
		"phase", PhaseActive,
	)

	// 5 分後に再 Reconcile をスケジュールする（状態の定期チェック）
	return reconcile.Result{RequeueAfter: 5 * time.Minute}, nil
}
