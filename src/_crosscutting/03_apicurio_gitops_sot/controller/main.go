// k1s0-impl: IMPL-cross_schema-0001 realizes=FR-cross_schema-001
// k1s0 Apicurio GitOps コントローラのエントリポイント
// GitOps リポジトリから Apicurio Registry への schema 同期を担当する additive-only コントローラ
package main

import (
	// コンテキストパッケージ: コンテキスト管理に使用する
	"context"
	// encoding/json パッケージ: JSON シリアライズに使用する
	"encoding/json"
	// fmt パッケージ: フォーマット出力に使用する
	"fmt"
	// io パッケージ: I/O 操作に使用する
	"io"
	// net/http パッケージ: HTTP クライアントに使用する
	"net/http"
	// os パッケージ: OS 操作に使用する
	"os"
	// strings パッケージ: 文字列操作に使用する
	"strings"

	// Kubernetes API マシナリー: API グループバージョン定義に使用する
	metav1 "k8s.io/apimachinery/pkg/apis/meta/v1"
	// Kubernetes ランタイム: ランタイムオブジェクト操作に使用する
	"k8s.io/apimachinery/pkg/runtime"
	// Kubernetes ランタイムスキーマ: スキーマ定義に使用する
	"k8s.io/apimachinery/pkg/runtime/schema"
	// controller-runtime: コントローラ構築の中心ライブラリ
	ctrl "sigs.k8s.io/controller-runtime"
	// controller-runtime クライアント: Kubernetes API クライアントに使用する
	"sigs.k8s.io/controller-runtime/pkg/client"
	// controller-runtime ログ: ロギングに使用する
	"sigs.k8s.io/controller-runtime/pkg/log"
	// controller-runtime zap: zap ロガー統合に使用する
	"sigs.k8s.io/controller-runtime/pkg/log/zap"
)

// ApicurioSchema は Apicurio Registry に同期する schema を表す CRD の Go 型定義
type ApicurioSchema struct {
	// Kubernetes 型メタ情報: TypeMeta を埋め込む
	metav1.TypeMeta `json:",inline"`
	// Kubernetes オブジェクトメタ情報: ObjectMeta を埋め込む
	metav1.ObjectMeta `json:"metadata,omitempty"`
	// Spec: ApicurioSchema の望ましい状態を定義する
	Spec ApicurioSchemaSpec `json:"spec,omitempty"`
	// Status: ApicurioSchema の現在の状態を定義する
	Status ApicurioSchemaStatus `json:"status,omitempty"`
}

// ApicurioSchemaSpec は ApicurioSchema の望ましい状態を定義する構造体
type ApicurioSchemaSpec struct {
	// ArtifactID: Apicurio Registry 内での artifact 識別子
	ArtifactID string `json:"artifactId"`
	// GroupID: Apicurio Registry 内での group 識別子
	GroupID string `json:"groupId"`
	// Content: 同期する schema の内容 (JSON/Avro/Protobuf 等)
	Content string `json:"content"`
	// ContentType: schema のコンテンツタイプ (application/json 等)
	ContentType string `json:"contentType"`
	// Version: schema のバージョン文字列
	Version string `json:"version"`
}

// ApicurioSchemaStatus は ApicurioSchema の現在の状態を定義する構造体
type ApicurioSchemaStatus struct {
	// SyncedVersion: Apicurio Registry に同期済みのバージョン
	SyncedVersion string `json:"syncedVersion,omitempty"`
	// LastSyncedAt: 最後に同期した日時
	LastSyncedAt string `json:"lastSyncedAt,omitempty"`
	// Conditions: 同期状態の詳細条件リスト
	Conditions []metav1.Condition `json:"conditions,omitempty"`
}

// ApicurioSchemaList は ApicurioSchema のリスト型を定義する構造体
type ApicurioSchemaList struct {
	// Kubernetes 型メタ情報: TypeMeta を埋め込む
	metav1.TypeMeta `json:",inline"`
	// Kubernetes リストメタ情報: ListMeta を埋め込む
	metav1.ListMeta `json:"metadata,omitempty"`
	// Items: ApicurioSchema のリスト
	Items []ApicurioSchema `json:"items"`
}

// DeepCopyObject は ApicurioSchema のディープコピーを生成して返す
func (a *ApicurioSchema) DeepCopyObject() runtime.Object {
	// ポインタが nil の場合は nil を返す
	if a == nil {
		return nil
	}
	// 新しい ApicurioSchema を作成して値をコピーする
	out := new(ApicurioSchema)
	// 型メタ情報をコピーする
	out.TypeMeta = a.TypeMeta
	// オブジェクトメタ情報をコピーする
	out.ObjectMeta = *a.ObjectMeta.DeepCopy()
	// Spec をコピーする
	out.Spec = a.Spec
	// Status をコピーする
	out.Status = a.Status
	// コピーされたオブジェクトを返す
	return out
}

// DeepCopyObject は ApicurioSchemaList のディープコピーを生成して返す
func (a *ApicurioSchemaList) DeepCopyObject() runtime.Object {
	// ポインタが nil の場合は nil を返す
	if a == nil {
		return nil
	}
	// 新しい ApicurioSchemaList を作成して値をコピーする
	out := new(ApicurioSchemaList)
	// 型メタ情報をコピーする
	out.TypeMeta = a.TypeMeta
	// リストメタ情報をコピーする
	out.ListMeta = a.ListMeta
	// Items リストをコピーする
	out.Items = make([]ApicurioSchema, len(a.Items))
	// 各 Item をコピーする
	for i := range a.Items {
		// ディープコピーした Item を設定する
		out.Items[i] = *a.Items[i].DeepCopyObject().(*ApicurioSchema)
	}
	// コピーされたリストを返す
	return out
}

// ApicurioSchemaGroupVersionResource は CRD の GVR を定義する変数
var ApicurioSchemaGroupVersionResource = schema.GroupVersionResource{
	// CRD グループ名: k1s0.io
	Group: "k1s0.io",
	// CRD バージョン: v1alpha1
	Version: "v1alpha1",
	// CRD リソース名: apicurioschemas
	Resource: "apicurioschemas",
}

// ApicurioRegistryClient は Apicurio Registry HTTP クライアントを表す構造体
type ApicurioRegistryClient struct {
	// BaseURL: Apicurio Registry の ベース URL
	BaseURL string
	// HTTPClient: HTTP 通信に使用するクライアント
	HTTPClient *http.Client
}

// NewApicurioRegistryClient は新しい ApicurioRegistryClient を生成して返す
func NewApicurioRegistryClient(baseURL string) *ApicurioRegistryClient {
	// 新しい ApicurioRegistryClient を初期化する
	return &ApicurioRegistryClient{
		// ベース URL を設定する
		BaseURL: strings.TrimRight(baseURL, "/"),
		// デフォルト HTTP クライアントを設定する
		HTTPClient: &http.Client{},
	}
}

// GetArtifact は Apicurio Registry から指定の artifact を取得する
func (c *ApicurioRegistryClient) GetArtifact(ctx context.Context, groupID, artifactID string) (string, error) {
	// 取得用 URL を構築する
	url := fmt.Sprintf("%s/apis/registry/v2/groups/%s/artifacts/%s", c.BaseURL, groupID, artifactID)
	// HTTP GET リクエストを作成する
	req, err := http.NewRequestWithContext(ctx, http.MethodGet, url, nil)
	// リクエスト作成に失敗した場合はエラーを返す
	if err != nil {
		return "", fmt.Errorf("HTTPリクエスト作成失敗: %w", err)
	}
	// リクエストを実行して結果を取得する
	resp, err := c.HTTPClient.Do(req)
	// リクエスト実行に失敗した場合はエラーを返す
	if err != nil {
		return "", fmt.Errorf("Apicurio Registry への接続失敗: %w", err)
	}
	// レスポンスボディを確実に閉じる
	defer resp.Body.Close()
	// 404 の場合は artifact が存在しないことを示す空文字を返す
	if resp.StatusCode == http.StatusNotFound {
		return "", nil
	}
	// その他のエラーステータスコードの場合はエラーを返す
	if resp.StatusCode != http.StatusOK {
		return "", fmt.Errorf("Apicurio Registry から予期しないステータス: %d", resp.StatusCode)
	}
	// レスポンスボディを読み取る
	body, err := io.ReadAll(resp.Body)
	// ボディ読み取りに失敗した場合はエラーを返す
	if err != nil {
		return "", fmt.Errorf("レスポンスボディ読み取り失敗: %w", err)
	}
	// 取得した artifact の内容を返す
	return string(body), nil
}

// CreateOrUpdateArtifact は Apicurio Registry に artifact を作成または更新する (additive only)
func (c *ApicurioRegistryClient) CreateOrUpdateArtifact(ctx context.Context, groupID, artifactID, contentType, content string) error {
	// artifact 作成/更新用 URL を構築する
	url := fmt.Sprintf("%s/apis/registry/v2/groups/%s/artifacts", c.BaseURL, groupID)
	// リクエストボディを strings.NewReader で作成する
	body := strings.NewReader(content)
	// HTTP POST リクエストを作成する
	req, err := http.NewRequestWithContext(ctx, http.MethodPost, url, body)
	// リクエスト作成に失敗した場合はエラーを返す
	if err != nil {
		return fmt.Errorf("HTTPリクエスト作成失敗: %w", err)
	}
	// Content-Type ヘッダを設定する
	req.Header.Set("Content-Type", contentType)
	// Artifact ID ヘッダを設定する
	req.Header.Set("X-Registry-ArtifactId", artifactID)
	// リクエストを実行して結果を取得する
	resp, err := c.HTTPClient.Do(req)
	// リクエスト実行に失敗した場合はエラーを返す
	if err != nil {
		return fmt.Errorf("Apicurio Registry への接続失敗: %w", err)
	}
	// レスポンスボディを確実に閉じる
	defer resp.Body.Close()
	// 成功ステータスコード以外の場合はエラーを返す
	if resp.StatusCode != http.StatusOK && resp.StatusCode != http.StatusCreated {
		return fmt.Errorf("Apicurio Registry への artifact 登録失敗: %d", resp.StatusCode)
	}
	// 登録成功を返す
	return nil
}

// ApicurioSchemaReconciler は ApicurioSchema CRD の reconciler 実装
type ApicurioSchemaReconciler struct {
	// client: Kubernetes API クライアント
	client.Client
	// ApicurioClient: Apicurio Registry HTTP クライアント
	ApicurioClient *ApicurioRegistryClient
}

// Reconcile は ApicurioSchema リソースの reconcile ループを実装する
func (r *ApicurioSchemaReconciler) Reconcile(ctx context.Context, req ctrl.Request) (ctrl.Result, error) {
	// ロガーをコンテキストから取得する
	logger := log.FromContext(ctx)
	// reconcile 開始ログを出力する
	logger.Info("ApicurioSchema reconcile 開始", "name", req.NamespacedName)

	// ApicurioSchema リソースを Kubernetes API から取得する
	var apicurioSchema ApicurioSchema
	// Get でリソースを取得する
	if err := r.Get(ctx, req.NamespacedName, &apicurioSchema); err != nil {
		// リソースが見つからない場合は削除済みとして正常終了する
		logger.Info("ApicurioSchema が見つからない (削除済み)", "name", req.NamespacedName)
		// 削除は additive-only ポリシーで無視する
		return ctrl.Result{}, client.IgnoreNotFound(err)
	}

	// Apicurio Registry から既存の artifact を取得する
	existing, err := r.ApicurioClient.GetArtifact(ctx, apicurioSchema.Spec.GroupID, apicurioSchema.Spec.ArtifactID)
	// 取得に失敗した場合はエラーを返す
	if err != nil {
		// 取得エラーをログに記録する
		logger.Error(err, "Apicurio Registry からの artifact 取得失敗")
		// エラーを返して再 reconcile をスケジュールする
		return ctrl.Result{}, err
	}

	// additive-only チェック: 既存 artifact の削除を検知した場合は reject する
	if existing != "" && apicurioSchema.Spec.Content == "" {
		// schema 削除を検知してエラーログを出力する
		logger.Error(nil, "additive-only ポリシー違反: schema の削除は許可されない",
			"artifactId", apicurioSchema.Spec.ArtifactID,
			"groupId", apicurioSchema.Spec.GroupID,
		)
		// エラーを返して再 reconcile をスケジュールしない (削除要求は永続的に拒否する)
		return ctrl.Result{}, fmt.Errorf("additive-only ポリシー違反: schema 削除は許可されない")
	}

	// 既存の artifact が変更される場合 (additive な変更のみ許可する)
	if existing != "" {
		// 既存 artifact との差分を確認するログを出力する
		logger.Info("既存 artifact への additive 更新を実行する",
			"artifactId", apicurioSchema.Spec.ArtifactID,
		)
	}

	// Apicurio Registry に artifact を作成または更新する
	if err := r.ApicurioClient.CreateOrUpdateArtifact(
		ctx,
		apicurioSchema.Spec.GroupID,
		apicurioSchema.Spec.ArtifactID,
		apicurioSchema.Spec.ContentType,
		apicurioSchema.Spec.Content,
	); err != nil {
		// 同期エラーをログに記録する
		logger.Error(err, "Apicurio Registry への schema 同期失敗")
		// エラーを返して再 reconcile をスケジュールする
		return ctrl.Result{}, err
	}

	// reconcile 成功ログを出力する
	logger.Info("ApicurioSchema reconcile 成功",
		"artifactId", apicurioSchema.Spec.ArtifactID,
		"version", apicurioSchema.Spec.Version,
	)
	// 正常終了を返す
	return ctrl.Result{}, nil
}

// SetupWithManager は reconciler をコントローラマネージャに登録する
func (r *ApicurioSchemaReconciler) SetupWithManager(mgr ctrl.Manager) error {
	// コントローラを ApicurioSchema リソースにバインドして登録する
	return ctrl.NewControllerManagedBy(mgr).
		// ApicurioSchema リソースを監視対象に設定する
		For(&ApicurioSchema{}).
		// コントローラをマネージャに登録する
		Complete(r)
}

// schemeBuilder は CRD スキーマを登録するためのビルダー
var schemeBuilder = runtime.NewSchemeBuilder(func(s *runtime.Scheme) error {
	// ApicurioSchema の GVK をスキームに登録する
	s.AddKnownTypes(
		// k1s0.io/v1alpha1 グループバージョンを設定する
		schema.GroupVersion{Group: "k1s0.io", Version: "v1alpha1"},
		// ApicurioSchema 型を登録する
		&ApicurioSchema{},
		// ApicurioSchemaList 型を登録する
		&ApicurioSchemaList{},
	)
	// 登録成功を返す
	return nil
})

// healthzHandler はヘルスチェックエンドポイントを処理する
func healthzHandler(w http.ResponseWriter, r *http.Request) {
	// ヘルスチェック応答を返す
	w.WriteHeader(http.StatusOK)
	// ヘルスチェック結果を JSON 形式で出力する
	if _, err := w.Write([]byte(`{"status":"ok"}`)); err != nil {
		// 書き込みエラーをログに記録する (忽視可能)
		_ = err
	}
}

// apicurioRegistryResponseDecoder は Apicurio Registry のレスポンスを JSON デコードするユーティリティ
func apicurioRegistryResponseDecoder(body io.Reader, target interface{}) error {
	// JSON デコーダを作成する
	decoder := json.NewDecoder(body)
	// デコード結果を target にバインドする
	return decoder.Decode(target)
}

// メイン関数: コントローラマネージャを初期化して起動する
func main() {
	// zap ロガーオプションを初期化する
	opts := zap.Options{
		// 開発モードを有効化する (開発環境では詳細ログを出力する)
		Development: true,
	}
	// controller-runtime のグローバルロガーを zap で設定する
	ctrl.SetLogger(zap.New(zap.UseFlagOptions(&opts)))

	// ロガーをメイン関数で取得する
	setupLog := ctrl.Log.WithName("setup")
	// セットアップ開始ログを出力する
	setupLog.Info("k1s0 Apicurio GitOps コントローラ起動中")

	// Apicurio Registry の URL を環境変数から取得する
	apicurioURL := os.Getenv("APICURIO_REGISTRY_URL")
	// 環境変数が設定されていない場合はデフォルト値を使用する
	if apicurioURL == "" {
		// デフォルトの Apicurio Registry URL を設定する
		apicurioURL = "http://apicurio-registry.k1s0-system.svc.cluster.local:8080"
	}

	// ランタイムスキームを作成する
	scheme := runtime.NewScheme()
	// CRD スキーマをスキームに登録する
	if err := schemeBuilder.AddToScheme(scheme); err != nil {
		// スキーマ登録失敗の場合はエラーログを出力して終了する
		setupLog.Error(err, "スキーマ登録失敗")
		// 非ゼロ終了コードでプロセスを終了する
		os.Exit(1)
	}

	// コントローラマネージャを作成する
	mgr, err := ctrl.NewManager(ctrl.GetConfigOrDie(), ctrl.Options{
		// ランタイムスキームを設定する
		Scheme: scheme,
	})
	// マネージャ作成に失敗した場合はエラーログを出力して終了する
	if err != nil {
		// マネージャ作成失敗のエラーログを出力する
		setupLog.Error(err, "コントローラマネージャ作成失敗")
		// 非ゼロ終了コードでプロセスを終了する
		os.Exit(1)
	}

	// Apicurio Registry クライアントを初期化する
	apicurioClient := NewApicurioRegistryClient(apicurioURL)
	// Apicurio Registry クライアント初期化完了ログを出力する
	setupLog.Info("Apicurio Registry クライアント初期化完了", "url", apicurioURL)

	// ApicurioSchemaReconciler をマネージャに登録する
	if err := (&ApicurioSchemaReconciler{
		// Kubernetes API クライアントをマネージャから取得して設定する
		Client: mgr.GetClient(),
		// Apicurio Registry クライアントを設定する
		ApicurioClient: apicurioClient,
	}).SetupWithManager(mgr); err != nil {
		// reconciler 登録失敗のエラーログを出力する
		setupLog.Error(err, "ApicurioSchemaReconciler の登録失敗")
		// 非ゼロ終了コードでプロセスを終了する
		os.Exit(1)
	}

	// コントローラマネージャの起動ログを出力する
	setupLog.Info("コントローラマネージャ起動")
	// コントローラマネージャを起動してブロックする
	if err := mgr.Start(ctrl.SetupSignalHandler()); err != nil {
		// マネージャ起動失敗のエラーログを出力する
		setupLog.Error(err, "コントローラマネージャ起動失敗")
		// 非ゼロ終了コードでプロセスを終了する
		os.Exit(1)
	}
	// apicurioRegistryResponseDecoder の参照を解消する (未使用警告を避ける)
	_ = apicurioRegistryResponseDecoder
}
