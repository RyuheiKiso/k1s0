// k1s0 tier1 operator: Tier1Service CRD の Reconciler エントリポイント
// controller-runtime v0.20 を使用して Tier1Service リソースを管理する

package main

import (
	// コンテキストパッケージ: Reconcile コンテキスト管理に使用する
	"context"
	// encoding/json パッケージ: conformance API レスポンスの JSON デコードに使用する
	"encoding/json"
	// fmt パッケージ: フォーマット出力に使用する
	"fmt"
	// io パッケージ: HTTP レスポンスボディの読み取りに使用する
	"io"
	// net/http パッケージ: conformance エンドポイントへの HTTP GET に使用する
	"net/http"
	// os パッケージ: OS シグナル処理・環境変数取得に使用する
	"os"
	// 時刻パッケージ: LastReconciledAt タイムスタンプ・RequeueAfter に使用する
	"time"

	// tier1 operator API 型定義のインポート
	tier1v1 "github.com/k1s0/tier1-operator/api/v1"
	// tier1 operator internal controller パッケージのインポート: 4 reconciler の登録に使用する
	tiercontroller "github.com/k1s0/tier1-operator/internal/controller"
	// Kubernetes apps/v1 API: Deployment 型に使用する
	appsv1 "k8s.io/api/apps/v1"
	// Kubernetes core/v1 API: Service / Container / Port 型に使用する
	corev1 "k8s.io/api/core/v1"
	// Kubernetes API マシナリー: API グループバージョン定義に使用する
	metav1 "k8s.io/apimachinery/pkg/apis/meta/v1"
	// Kubernetes ランタイム: ランタイムオブジェクト操作に使用する
	"k8s.io/apimachinery/pkg/runtime"
	// Kubernetes ランタイムスキーマ: スキーマ定義に使用する
	"k8s.io/apimachinery/pkg/runtime/schema"
	// Kubernetes util intstr: IntOrString 型に使用する
	"k8s.io/apimachinery/pkg/util/intstr"
	// controller-runtime: コントローラ構築の中心ライブラリ
	ctrl "sigs.k8s.io/controller-runtime"
	// controller-runtime クライアント: Kubernetes API クライアントに使用する
	"sigs.k8s.io/controller-runtime/pkg/client"
	// controller-runtime controllerutil: CreateOrUpdate / owner reference に使用する
	"sigs.k8s.io/controller-runtime/pkg/controller/controllerutil"
	// controller-runtime ログ: ロギングに使用する
	"sigs.k8s.io/controller-runtime/pkg/log"
	// controller-runtime zap: zap ロガー統合に使用する
	"sigs.k8s.io/controller-runtime/pkg/log/zap"
)

// gatewayImage は子 Deployment で使うコンテナイメージのデフォルト値
// 環境変数 GATEWAY_IMAGE で上書き可能とする
const gatewayImageDefault = "ghcr.io/k1s0-io/k1s0-tier1-gateway:latest"

// conformanceReport は /conformance/run エンドポイントのレスポンス JSON 構造体
type conformanceReport struct {
	// AllPassed: 全 conformance cell が pass なら true
	AllPassed bool `json:"all_passed"`
}

// Tier1ServiceReconciler は Tier1Service CRD を Reconcile するコントローラ
type Tier1ServiceReconciler struct {
	// Kubernetes クライアント: リソース読み書きに使用する
	client.Client
	// Scheme: CRD 型登録に使用する
	Scheme *runtime.Scheme
	// HTTPClient: conformance エンドポイント呼び出しに使用する HTTP クライアント
	HTTPClient *http.Client
}

// gatewayImage は GATEWAY_IMAGE 環境変数が設定されている場合はその値を、
// そうでなければデフォルトイメージを返す
func gatewayImage() string {
	// 環境変数を読み取る
	if v := os.Getenv("GATEWAY_IMAGE"); v != "" {
		// 環境変数が設定されていればその値を採用する
		return v
	}
	// デフォルトイメージを返す
	return gatewayImageDefault
}

// labelSelector は Deployment / Service の selector / labels に付与する共通ラベルマップを返す
func labelSelector(name string) map[string]string {
	// アプリ識別ラベルと Tier1Service 名ラベルを返す
	return map[string]string{
		// アプリ識別ラベル: tier1-gateway で固定する
		"app": "tier1-gateway",
		// Tier1Service リソース名: 複数インスタンスを区別するために使用する
		"tier1service": name,
	}
}

// buildDeployment は Tier1Service の spec を元に子 Deployment オブジェクトを構築する
// ownerRef 設定は呼び出し元で controllerutil.SetControllerReference を使って行う
func buildDeployment(owner *tier1v1.Tier1Service) *appsv1.Deployment {
	// Deployment の selector / template に付与するラベルを生成する
	labels := labelSelector(owner.Name)
	// コンテナポート番号: 8080 で固定する
	containerPort := int32(8080)
	// Deployment オブジェクトを構築する
	dep := &appsv1.Deployment{
		// TypeMeta: Kubernetes API group/version/kind を指定する
		TypeMeta: metav1.TypeMeta{
			// APIVersion: apps/v1
			APIVersion: "apps/v1",
			// Kind: Deployment
			Kind: "Deployment",
		},
		// ObjectMeta: Deployment の名前・namespace を Tier1Service と同一にする
		ObjectMeta: metav1.ObjectMeta{
			// Deployment 名: Tier1Service 名に "-deployment" サフィックスを付ける
			Name: owner.Name + "-deployment",
			// Namespace: Tier1Service と同じ namespace に配置する
			Namespace: owner.Namespace,
		},
		// Spec: レプリカ数・セレクタ・Pod テンプレートを定義する
		Spec: appsv1.DeploymentSpec{
			// Replicas: Tier1Service の spec.replicas 値を使用する
			Replicas: &owner.Spec.Replicas,
			// Selector: Pod 選択条件を labels と一致させる
			Selector: &metav1.LabelSelector{
				// MatchLabels: labels マップと同じラベルで Pod を選択する
				MatchLabels: labels,
			},
			// Template: Pod テンプレートを定義する
			Template: corev1.PodTemplateSpec{
				// ObjectMeta: Pod に selector と同じラベルを付与する
				ObjectMeta: metav1.ObjectMeta{
					// Labels: selector と一致するラベルを設定する
					Labels: labels,
				},
				// Spec: コンテナ定義を設定する
				Spec: corev1.PodSpec{
					// Containers: gateway コンテナ 1 つを定義する
					Containers: []corev1.Container{
						{
							// Name: コンテナ名を tier1-gateway に固定する
							Name: "tier1-gateway",
							// Image: GATEWAY_IMAGE 環境変数またはデフォルトイメージを使用する
							Image: gatewayImage(),
							// Ports: コンテナポート 8080 を公開する
							Ports: []corev1.ContainerPort{
								{
									// ContainerPort: gateway が listen するポート番号
									ContainerPort: containerPort,
									// Protocol: TCP プロトコルを指定する
									Protocol: corev1.ProtocolTCP,
								},
							},
						},
					},
				},
			},
		},
	}
	// 構築した Deployment を返す
	return dep
}

// buildService は Tier1Service の spec を元に子 Service オブジェクトを構築する
// ownerRef 設定は呼び出し元で controllerutil.SetControllerReference を使って行う
func buildService(owner *tier1v1.Tier1Service) *corev1.Service {
	// Service の selector に付与するラベルを生成する
	labels := labelSelector(owner.Name)
	// Service オブジェクトを構築する
	svc := &corev1.Service{
		// TypeMeta: Kubernetes API group/version/kind を指定する
		TypeMeta: metav1.TypeMeta{
			// APIVersion: v1（core API group）
			APIVersion: "v1",
			// Kind: Service
			Kind: "Service",
		},
		// ObjectMeta: Service の名前・namespace を Tier1Service と同一にする
		ObjectMeta: metav1.ObjectMeta{
			// Service 名: Tier1Service 名に "-service" サフィックスを付ける
			Name: owner.Name + "-service",
			// Namespace: Tier1Service と同じ namespace に配置する
			Namespace: owner.Namespace,
		},
		// Spec: ポート・セレクタを定義する
		Spec: corev1.ServiceSpec{
			// Ports: ポート 8080 を公開する
			Ports: []corev1.ServicePort{
				{
					// Port: Service が公開するポート番号
					Port: 8080,
					// TargetPort: Pod のコンテナポートへ転送する
					TargetPort: intstr.FromInt(8080),
					// Protocol: TCP プロトコルを指定する
					Protocol: corev1.ProtocolTCP,
				},
			},
			// Selector: Deployment の Pod を選択するラベルセレクタ
			Selector: labels,
		},
	}
	// 構築した Service を返す
	return svc
}

// runConformance は gateway Service の /conformance/run を HTTP GET で呼び出し、
// all_passed が true なら "green"、false なら "red" を返す。
// HTTP 失敗時は "red" とエラーを返す。
func (r *Tier1ServiceReconciler) runConformance(ctx context.Context, gatewayServiceName, namespace string) (string, error) {
	// conformance エンドポイント URL を組み立てる
	// Service 名は "<name>-service"、port は 8080 で固定する
	url := fmt.Sprintf("http://%s.%s.svc.cluster.local:8080/conformance/run", gatewayServiceName, namespace)
	// HTTP GET リクエストをコンテキスト付きで生成する
	req, err := http.NewRequestWithContext(ctx, http.MethodGet, url, nil)
	// リクエスト生成失敗時はエラーを返す
	if err != nil {
		// エラーを "red" と共に返す
		return "red", fmt.Errorf("failed to build conformance request: %w", err)
	}
	// HTTP GET を実行する
	resp, err := r.HTTPClient.Do(req)
	// HTTP 実行失敗時はエラーを返す
	if err != nil {
		// ネットワークエラー等: "red" を返す
		return "red", fmt.Errorf("conformance HTTP GET failed: %w", err)
	}
	// レスポンスボディを必ずクローズする
	defer resp.Body.Close()
	// HTTP ステータスが 2xx 以外の場合はエラーとみなす
	if resp.StatusCode < 200 || resp.StatusCode >= 300 {
		// ステータスコードを含むエラーを返す
		return "red", fmt.Errorf("conformance endpoint returned HTTP %d", resp.StatusCode)
	}
	// レスポンスボディを読み取る（最大 1 MB に制限する）
	body, err := io.ReadAll(io.LimitReader(resp.Body, 1<<20))
	// 読み取り失敗時はエラーを返す
	if err != nil {
		// 読み取りエラー: "red" を返す
		return "red", fmt.Errorf("failed to read conformance response body: %w", err)
	}
	// JSON をデコードする
	var report conformanceReport
	// JSON デコード実行
	if err := json.Unmarshal(body, &report); err != nil {
		// JSON パース失敗: "red" を返す
		return "red", fmt.Errorf("failed to decode conformance response: %w", err)
	}
	// all_passed が true なら "green"、false なら "red" を返す
	if report.AllPassed {
		// 全 cell が pass: "green" を返す
		return "green", nil
	}
	// 1 つ以上 fail: "red" を返す
	return "red", nil
}

// Reconcile は Tier1Service リソースの変更を処理するメインループ
func (r *Tier1ServiceReconciler) Reconcile(ctx context.Context, req ctrl.Request) (ctrl.Result, error) {
	// ロガーをコンテキストから取得する
	logger := log.FromContext(ctx)
	// Reconcile 開始をログに記録する
	logger.Info("Reconciling Tier1Service", "name", req.Name, "namespace", req.Namespace)

	// ---- 1. Tier1Service リソースを取得する ----

	// Tier1Service オブジェクトを宣言する
	var tier1Service tier1v1.Tier1Service
	// API サーバから Tier1Service を取得する
	if err := r.Get(ctx, req.NamespacedName, &tier1Service); err != nil {
		// リソースが見つからない場合は Reconcile を終了する（削除済みの可能性）
		logger.Info("Tier1Service not found, ignoring", "error", err)
		// NotFound エラーは無視して正常終了とする
		return ctrl.Result{}, client.IgnoreNotFound(err)
	}

	// ---- 2. owner reference helper を構築する ----
	// Deployment / Service の OwnerReference に Tier1Service を指定するための
	// スキーム参照を確認する（SetControllerReference が内部で使用する）

	// ---- 3. 子 Deployment を構築して CreateOrUpdate する ----

	// 子 Deployment オブジェクトを生成する
	dep := buildDeployment(&tier1Service)
	// owner reference を設定する（Tier1Service 削除時に Deployment も GC される）
	if err := controllerutil.SetControllerReference(&tier1Service, dep, r.Scheme); err != nil {
		// owner reference 設定失敗: エラーを返してリトライさせる
		logger.Error(err, "Failed to set owner reference on Deployment")
		return ctrl.Result{}, err
	}
	// CreateOrUpdate: Deployment が存在しなければ作成、存在すれば更新する
	depResult, err := controllerutil.CreateOrUpdate(ctx, r.Client, dep, func() error {
		// mutate func: 既存 Deployment の spec を望ましい状態へ更新する
		// Replicas を Tier1Service の spec.replicas に合わせる
		dep.Spec.Replicas = &tier1Service.Spec.Replicas
		// コンテナイメージを最新値に更新する
		if len(dep.Spec.Template.Spec.Containers) > 0 {
			// 既存コンテナのイメージを更新する
			dep.Spec.Template.Spec.Containers[0].Image = gatewayImage()
		}
		// mutate 成功: nil を返す
		return nil
	})
	// Deployment の作成・更新失敗時はエラーを返す
	if err != nil {
		// エラーをログに記録してリトライさせる
		logger.Error(err, "Failed to CreateOrUpdate Deployment")
		return ctrl.Result{}, err
	}
	// Deployment 操作結果をログに記録する
	logger.Info("Deployment reconciled", "result", depResult, "name", dep.Name)

	// ---- 4. 子 Service を構築して CreateOrUpdate する ----

	// 子 Service オブジェクトを生成する
	svc := buildService(&tier1Service)
	// owner reference を設定する（Tier1Service 削除時に Service も GC される）
	if err := controllerutil.SetControllerReference(&tier1Service, svc, r.Scheme); err != nil {
		// owner reference 設定失敗: エラーを返してリトライさせる
		logger.Error(err, "Failed to set owner reference on Service")
		return ctrl.Result{}, err
	}
	// CreateOrUpdate: Service が存在しなければ作成、存在すれば更新する
	svcResult, err := controllerutil.CreateOrUpdate(ctx, r.Client, svc, func() error {
		// mutate func: 既存 Service の spec を望ましい状態へ更新する
		// Selector を最新のラベルマップに合わせる
		svc.Spec.Selector = labelSelector(tier1Service.Name)
		// Port 定義を上書きする
		svc.Spec.Ports = []corev1.ServicePort{
			{
				// Port: Service が公開するポート番号
				Port: 8080,
				// TargetPort: Pod のコンテナポートへ転送する
				TargetPort: intstr.FromInt(8080),
				// Protocol: TCP プロトコルを指定する
				Protocol: corev1.ProtocolTCP,
			},
		}
		// mutate 成功: nil を返す
		return nil
	})
	// Service の作成・更新失敗時はエラーを返す
	if err != nil {
		// エラーをログに記録してリトライさせる
		logger.Error(err, "Failed to CreateOrUpdate Service")
		return ctrl.Result{}, err
	}
	// Service 操作結果をログに記録する
	logger.Info("Service reconciled", "result", svcResult, "name", svc.Name)

	// ---- 5. HTTP GET /conformance/run で conformance 状態を取得する ----

	// gateway Service 名（"<name>-service"）を使って conformance URL を組み立てる
	conformanceStatus, conformanceErr := r.runConformance(ctx, svc.Name, tier1Service.Namespace)
	// conformance 呼び出しエラーをログに記録する（Reconcile は継続する）
	if conformanceErr != nil {
		// conformance 失敗はログのみ: Status は "red" として更新を続ける
		logger.Error(conformanceErr, "conformance check failed, setting status=red")
	}

	// ---- 6. Status.ConformanceStatus を更新する ----

	// conformanceStatus に応じて Condition の Type と Message を設定する
	conditionStatus := metav1.ConditionTrue
	// conformanceStatus が "green" 以外の場合は Condition を False にする
	if conformanceStatus != "green" {
		// "red" の場合は False にする
		conditionStatus = metav1.ConditionFalse
	}
	// Status フィールドを更新する
	tier1Service.Status.ConformanceStatus = conformanceStatus
	// LastReconciledAt を現在時刻で更新する
	tier1Service.Status.LastReconciledAt = time.Now().UTC().Format(time.RFC3339)
	// ConformanceComplete Condition を設定する
	tier1Service.Status.Conditions = []metav1.Condition{
		{
			// 条件タイプ: ConformanceComplete
			Type: "ConformanceComplete",
			// 条件の真偽: all_passed が true なら ConditionTrue
			Status: conditionStatus,
			// 最後に遷移した時刻を記録する
			LastTransitionTime: metav1.Now(),
			// 理由: conformance 結果に応じた短い文字列
			Reason: func() string {
				// green の場合は "AllCellsGreen" を返す
				if conformanceStatus == "green" {
					return "AllCellsGreen"
				}
				// red の場合は "ConformanceFailed" を返す
				return "ConformanceFailed"
			}(),
			// Message: Tier1Service 情報と conformance 状態を含む詳細メッセージ
			Message: fmt.Sprintf(
				"Tier1Service %s/%s conformance=%s class=%s adapter=%s",
				req.Namespace, req.Name,
				conformanceStatus,
				tier1Service.Spec.ConformanceClass,
				tier1Service.Spec.Adapter,
			),
		},
	}
	// Status サブリソースを更新する
	if err := r.Status().Update(ctx, &tier1Service); err != nil {
		// Status 更新失敗をログに記録してリトライさせる
		logger.Error(err, "Failed to update Tier1Service status")
		return ctrl.Result{}, err
	}

	// Reconcile 完了をログに記録する
	logger.Info("Tier1Service reconciled successfully",
		"name", req.Name,
		"conformanceStatus", tier1Service.Status.ConformanceStatus,
		"deployment", dep.Name,
		"service", svc.Name,
	)

	// ---- 7. 30 分後に再 Reconcile をスケジュールする ----
	// RequeueAfter で定期的な conformance チェックを実行する
	return ctrl.Result{RequeueAfter: 30 * time.Minute}, nil
}

// SetupWithManager はコントローラをマネージャーに登録する
func (r *Tier1ServiceReconciler) SetupWithManager(mgr ctrl.Manager) error {
	// Tier1Service の変更イベントを監視する
	return ctrl.NewControllerManagedBy(mgr).
		// Tier1Service リソースを For として監視する
		For(&tier1v1.Tier1Service{}).
		// 子 Deployment の変更を owner reference 経由で監視する
		Owns(&appsv1.Deployment{}).
		// 子 Service の変更を owner reference 経由で監視する
		Owns(&corev1.Service{}).
		// コントローラを登録する
		Complete(r)
}

// メインエントリポイント
func main() {
	// zap ロガーを controller-runtime に設定する
	ctrl.SetLogger(zap.New(zap.UseDevMode(false)))
	// ロガーを取得する
	setupLog := ctrl.Log.WithName("setup")

	// Kubernetes スキームを構築する
	scheme := runtime.NewScheme()
	// apps/v1 スキームを登録する（Deployment 型の認識に必要）
	if err := appsv1.AddToScheme(scheme); err != nil {
		// スキーム登録失敗時は終了する
		setupLog.Error(err, "unable to add apps/v1 to scheme")
		os.Exit(1)
	}
	// core/v1 スキームを登録する（Service 型の認識に必要）
	if err := corev1.AddToScheme(scheme); err != nil {
		// スキーム登録失敗時は終了する
		setupLog.Error(err, "unable to add core/v1 to scheme")
		os.Exit(1)
	}
	// Tier1Service CRD をスキームに登録する
	scheme.AddKnownTypeWithName(
		schema.GroupVersionKind{
			// API グループ: k1s0.io
			Group: "k1s0.io",
			// API バージョン: v1
			Version: "v1",
			// Kind: Tier1Service
			Kind: "Tier1Service",
		},
		&tier1v1.Tier1Service{},
	)
	// Tier1ServiceList をスキームに登録する
	scheme.AddKnownTypeWithName(
		schema.GroupVersionKind{
			// API グループ: k1s0.io
			Group:   "k1s0.io",
			// API バージョン: v1
			Version: "v1",
			// Kind: Tier1ServiceList
			Kind: "Tier1ServiceList",
		},
		&tier1v1.Tier1ServiceList{},
	)
	// KeyClass CRD をスキームに登録する（05_鍵管理適合仕様.md §key_class 5 class 対応）
	scheme.AddKnownTypeWithName(
		schema.GroupVersionKind{Group: "k1s0.io", Version: "v1", Kind: "KeyClass"},
		&tier1v1.KeyClass{},
	)
	// KeyClassList をスキームに登録する
	scheme.AddKnownTypeWithName(
		schema.GroupVersionKind{Group: "k1s0.io", Version: "v1", Kind: "KeyClassList"},
		&tier1v1.KeyClassList{},
	)
	// SLOClass CRD をスキームに登録する（07_SLO適合仕様.md §slo_class 6 class 対応）
	scheme.AddKnownTypeWithName(
		schema.GroupVersionKind{Group: "k1s0.io", Version: "v1", Kind: "SLOClass"},
		&tier1v1.SLOClass{},
	)
	// SLOClassList をスキームに登録する
	scheme.AddKnownTypeWithName(
		schema.GroupVersionKind{Group: "k1s0.io", Version: "v1", Kind: "SLOClassList"},
		&tier1v1.SLOClassList{},
	)
	// QuotaClass CRD をスキームに登録する（09_テナント容量適合仕様.md §quota_class 5 class 対応）
	scheme.AddKnownTypeWithName(
		schema.GroupVersionKind{Group: "k1s0.io", Version: "v1", Kind: "QuotaClass"},
		&tier1v1.QuotaClass{},
	)
	// QuotaClassList をスキームに登録する
	scheme.AddKnownTypeWithName(
		schema.GroupVersionKind{Group: "k1s0.io", Version: "v1", Kind: "QuotaClassList"},
		&tier1v1.QuotaClassList{},
	)
	// OSSInventory CRD をスキームに登録する（08_OSSライフサイクル適合仕様.md §lifecycle_signal 対応）
	scheme.AddKnownTypeWithName(
		schema.GroupVersionKind{Group: "k1s0.io", Version: "v1", Kind: "OSSInventory"},
		&tier1v1.OSSInventory{},
	)
	// OSSInventoryList をスキームに登録する
	scheme.AddKnownTypeWithName(
		schema.GroupVersionKind{Group: "k1s0.io", Version: "v1", Kind: "OSSInventoryList"},
		&tier1v1.OSSInventoryList{},
	)

	// コントローラマネージャーを構築する
	mgr, err := ctrl.NewManager(ctrl.GetConfigOrDie(), ctrl.Options{
		// スキームを設定する
		Scheme: scheme,
	})
	// マネージャー構築失敗時は終了する
	if err != nil {
		// エラーをログに記録して終了する
		setupLog.Error(err, "unable to start manager")
		os.Exit(1)
	}

	// HTTP クライアントを構築する（タイムアウト 10 秒で conformance API を呼び出す）
	httpClient := &http.Client{
		// Timeout: conformance エンドポイントの応答待ち上限を 10 秒とする
		Timeout: 10 * time.Second,
	}

	// Reconciler を構築してマネージャーに登録する
	if err = (&Tier1ServiceReconciler{
		// Kubernetes クライアントを設定する
		Client: mgr.GetClient(),
		// スキームを設定する
		Scheme: mgr.GetScheme(),
		// HTTP クライアントを設定する
		HTTPClient: httpClient,
	}).SetupWithManager(mgr); err != nil {
		// コントローラ登録失敗時は終了する
		setupLog.Error(err, "unable to create controller", "controller", "Tier1Service")
		os.Exit(1)
	}

	// KeyClassReconciler をマネージャーに登録する（05_鍵管理適合仕様.md §rotation_cadence enforcement）
	if err = (&tiercontroller.KeyClassReconciler{
		// Kubernetes クライアントを設定する
		Client: mgr.GetClient(),
	}).SetupWithManager(mgr); err != nil {
		// コントローラ登録失敗時は終了する
		setupLog.Error(err, "unable to create controller", "controller", "KeyClass")
		os.Exit(1)
	}

	// SLOClassReconciler をマネージャーに登録する（07_SLO適合仕様.md §slo_class enforcement）
	if err = (&tiercontroller.SLOClassReconciler{
		// Kubernetes クライアントを設定する
		Client: mgr.GetClient(),
	}).SetupWithManager(mgr); err != nil {
		// コントローラ登録失敗時は終了する
		setupLog.Error(err, "unable to create controller", "controller", "SLOClass")
		os.Exit(1)
	}

	// QuotaClassReconciler をマネージャーに登録する（09_テナント容量適合仕様.md §quota enforcement）
	if err = (&tiercontroller.QuotaClassReconciler{
		// Kubernetes クライアントを設定する
		Client: mgr.GetClient(),
	}).SetupWithManager(mgr); err != nil {
		// コントローラ登録失敗時は終了する
		setupLog.Error(err, "unable to create controller", "controller", "QuotaClass")
		os.Exit(1)
	}

	// LifecycleSignalReconciler をマネージャーに登録する（08_OSSライフサイクル適合仕様.md §lifecycle_signal enforcement）
	if err = (&tiercontroller.LifecycleSignalReconciler{
		// Kubernetes クライアントを設定する
		Client: mgr.GetClient(),
	}).SetupWithManager(mgr); err != nil {
		// コントローラ登録失敗時は終了する
		setupLog.Error(err, "unable to create controller", "controller", "LifecycleSignal")
		os.Exit(1)
	}

	// マネージャーを起動する（シグナル受信まで待機する）
	setupLog.Info("starting tier1 operator manager")
	if err := mgr.Start(ctrl.SetupSignalHandler()); err != nil {
		// マネージャー起動失敗時は終了する
		setupLog.Error(err, "problem running manager")
		os.Exit(1)
	}
}
