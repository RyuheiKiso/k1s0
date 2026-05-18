// tier1_quota_types.go — k1s0 tier1 operator: QuotaClass CRD 型定義
// 09_テナント容量適合仕様.md §v1 quota_class セット（5 class）に準拠する。
// QuotaClass CRD はテナントの API リクエスト上限 / DB コネクション数 / ストレージ上限を管理する。

// パッケージ名: v1 API グループ（types.go と同一パッケージに属する）
package v1

import (
	// QuotaClass CRD のメタデータインポート
	metav1 "k8s.io/apimachinery/pkg/apis/meta/v1"
	// ランタイムパッケージ: DeepCopyObject インターフェース実装に使用する
	"k8s.io/apimachinery/pkg/runtime"
)

// QuotaClassID は quota_class の 5 class セットを宣言する (spec 09 §quota_class)
type QuotaClassID string

const (
	// V1QuotaFreeShared: 共有インフラ上の無償プラン
	V1QuotaFreeShared QuotaClassID = "v1_free_shared"
	// V1QuotaStarterShared: スターター共有プラン
	V1QuotaStarterShared QuotaClassID = "v1_starter_shared"
	// V1QuotaTeamShared: チーム共有プラン
	V1QuotaTeamShared QuotaClassID = "v1_team_shared"
	// V1QuotaBusinessDedicated: ビジネス専用インフラ
	V1QuotaBusinessDedicated QuotaClassID = "v1_business_dedicated"
	// V1QuotaEnterpriseDedicated: エンタープライズ専用インフラ
	V1QuotaEnterpriseDedicated QuotaClassID = "v1_enterprise_dedicated"
)

// EnforcementLayer は quota を適用するレイヤを宣言する (spec 09 §enforcement_layer)
type EnforcementLayer string

const (
	// EnforcementEnvoy: Envoy rate-limit フィルタ層
	EnforcementEnvoy EnforcementLayer = "envoy"
	// EnforcementKubernetes: k8s ResourceQuota 層
	EnforcementKubernetes EnforcementLayer = "kubernetes"
	// EnforcementDatabase: DB コネクションプール層
	EnforcementDatabase EnforcementLayer = "database"
	// EnforcementApp: アプリケーション token-bucket 層
	EnforcementApp EnforcementLayer = "app"
)

// FairnessModel は公平性モデルを宣言する (spec 09 §fairness_model)
type FairnessModel string

const (
	// FairnessTokenBucket: トークンバケットモデル
	FairnessTokenBucket FairnessModel = "token_bucket"
	// FairnessLeakyBucket: リーキーバケットモデル
	FairnessLeakyBucket FairnessModel = "leaky_bucket"
	// FairnessFixedWindow: 固定ウィンドウモデル
	FairnessFixedWindow FairnessModel = "fixed_window"
)

// ExhaustionResponse は quota 枯渇時の応答を宣言する (spec 09 §exhaustion_response)
type ExhaustionResponse string

const (
	// ExhaustionReject: リクエストを即時 reject する
	ExhaustionReject ExhaustionResponse = "reject"
	// ExhaustionQueue: リクエストをキュー待機させる
	ExhaustionQueue ExhaustionResponse = "queue"
	// ExhaustionDegrade: 機能縮退モードに切り替える
	ExhaustionDegrade ExhaustionResponse = "degrade"
)

// QuotaClassSpec は QuotaClass リソースの spec を定義する (spec 09 §v1 quota_class)
type QuotaClassSpec struct {
	// ClassID: quota class の 5 class 識別子
	// +kubebuilder:validation:Enum=v1_free_shared;v1_starter_shared;v1_team_shared;v1_business_dedicated;v1_enterprise_dedicated
	ClassID QuotaClassID `json:"classId"`
	// EnforcementLayer: quota を適用するレイヤ
	// +kubebuilder:validation:Enum=envoy;kubernetes;database;app
	EnforcementLayer EnforcementLayer `json:"enforcementLayer"`
	// FairnessModel: 公平性モデル
	// +kubebuilder:validation:Enum=token_bucket;leaky_bucket;fixed_window
	FairnessModel FairnessModel `json:"fairnessModel"`
	// BurstWindowSec: バースト許容ウィンドウ（秒数、0 でバースト不許可）
	BurstWindowSec int32 `json:"burstWindowSec,omitempty"`
	// ExhaustionResponse: quota 枯渇時の応答
	// +kubebuilder:validation:Enum=reject;queue;degrade
	ExhaustionResponse ExhaustionResponse `json:"exhaustionResponse"`
	// RequestsPerMinute: 1 分あたりリクエスト上限（0 = 無制限）
	RequestsPerMinute int64 `json:"requestsPerMinute,omitempty"`
	// DbConnectionsMax: DB コネクション数上限
	DbConnectionsMax int32 `json:"dbConnectionsMax,omitempty"`
}

// QuotaClassStatus は QuotaClass リソースのステータスを定義する
type QuotaClassStatus struct {
	// 適用状態: 実際のクォータが Kubernetes 側に反映済みなら true
	// +optional
	Applied bool `json:"applied,omitempty"`
	// 最終更新時刻: Reconcile が最後に status を更新した時刻
	// +optional
	LastUpdated *metav1.Time `json:"lastUpdated,omitempty"`
}

// QuotaClass は tier1 テナントのクォータクラスを定義する CRD
// +kubebuilder:object:root=true
// +kubebuilder:subresource:status
// +kubebuilder:resource:scope=Namespaced,shortName=qc
// +kubebuilder:printcolumn:name="Applied",type=boolean,JSONPath=".status.applied"
// +kubebuilder:printcolumn:name="Age",type=date,JSONPath=".metadata.creationTimestamp"
type QuotaClass struct {
	// Kubernetes 型メタ情報を埋め込む
	metav1.TypeMeta `json:",inline"`
	// Kubernetes 標準メタデータ
	metav1.ObjectMeta `json:"metadata,omitempty"`
	// クォータクラスの spec
	Spec QuotaClassSpec `json:"spec,omitempty"`
	// クォータクラスのステータス
	Status QuotaClassStatus `json:"status,omitempty"`
}

// QuotaClassList は QuotaClass リソースの一覧を定義する
// +kubebuilder:object:root=true
type QuotaClassList struct {
	// Kubernetes 型メタ情報を埋め込む
	metav1.TypeMeta `json:",inline"`
	// リスト標準メタデータ
	metav1.ListMeta `json:"metadata,omitempty"`
	// QuotaClass アイテム一覧
	Items []QuotaClass `json:"items"`
}

// DeepCopyObject は QuotaClass のディープコピーを返す（runtime.Object インターフェース実装）
func (in *QuotaClass) DeepCopyObject() runtime.Object {
	// コピー先を生成する
	out := new(QuotaClass)
	// TypeMeta / ObjectMeta / Spec / Status は値型なので浅いコピーで OK
	*out = *in
	// LastUpdated は *metav1.Time（ポインタ型）なのでディープコピーが必要
	if in.Status.LastUpdated != nil {
		// 新しい Time 値を生成してポインタを設定する
		t := *in.Status.LastUpdated
		out.Status.LastUpdated = &t
	}
	// コピーした QuotaClass を返す
	return out
}

// DeepCopyObject は QuotaClassList のディープコピーを返す（runtime.Object インターフェース実装）
func (in *QuotaClassList) DeepCopyObject() runtime.Object {
	// コピー先を生成する
	out := new(QuotaClassList)
	// フィールドをコピーする
	*out = *in
	// Items スライスのディープコピーを実行する
	if in.Items != nil {
		// スライスを新規アロケートする
		out.Items = make([]QuotaClass, len(in.Items))
		// 各 QuotaClass をディープコピーする
		for i := range in.Items {
			// 値型フィールドをコピーする
			out.Items[i] = in.Items[i]
			// LastUpdated ポインタをディープコピーする
			if in.Items[i].Status.LastUpdated != nil {
				// 新しい Time 値を生成してポインタを設定する
				t := *in.Items[i].Status.LastUpdated
				out.Items[i].Status.LastUpdated = &t
			}
		}
	}
	// コピーした QuotaClassList を返す
	return out
}
