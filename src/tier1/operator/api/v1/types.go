// k1s0 tier1 operator の CRD 型定義
// Tier1Service リソースの Spec / Status を kubebuilder annotation 付きで定義する

// パッケージ名: v1 API グループ
// +groupName=k1s0.io
package v1

import (
	// Kubernetes API マシナリーのメタ情報パッケージ
	metav1 "k8s.io/apimachinery/pkg/apis/meta/v1"
	// ランタイムパッケージ: DeepCopyObject インターフェース実装に使用する
	"k8s.io/apimachinery/pkg/runtime"
)

// Tier1ServiceSpec は Tier1Service の望ましい状態を定義する
type Tier1ServiceSpec struct {
	// ConformanceClass: 適用する Bidi conformance class（01_Bidi適合仕様.md §v1 conformance_class セット）
	ConformanceClass string `json:"conformanceClass"`
	// Adapter: 使用する transport adapter（01_Bidi適合仕様.md §adapter↔class supports 対応）
	Adapter string `json:"adapter"`
	// QuotaClass: テナント容量クラス（09_テナント容量適合仕様.md §v1 quota_class セット）
	QuotaClass string `json:"quotaClass"`
	// SloClass: SLO クラス（07_SLO適合仕様.md §v1 slo_class セット）
	SloClass string `json:"sloClass"`
	// Replicas: Pod のレプリカ数
	Replicas int32 `json:"replicas"`
}

// Tier1ServiceStatus は Tier1Service の現在の状態を定義する
type Tier1ServiceStatus struct {
	// ConformanceStatus: conformance テストの結果（green / red / pending）
	// +optional
	ConformanceStatus string `json:"conformanceStatus,omitempty"`
	// LastReconciledAt: 最後に Reconcile した日時（RFC3339 形式）
	// +optional
	LastReconciledAt string `json:"lastReconciledAt,omitempty"`
	// Conditions: Kubernetes 標準の詳細条件リスト（DeepCopy 対象）
	// +optional
	// +listType=map
	// +listMapKey=type
	Conditions []metav1.Condition `json:"conditions,omitempty"`
}

// Tier1Service は tier1 サービスを表す CRD の Go 型定義
// +kubebuilder:object:root=true
// +kubebuilder:subresource:status
// +kubebuilder:resource:scope=Namespaced,shortName=t1svc
// +kubebuilder:printcolumn:name="ConformanceClass",type=string,JSONPath=".spec.conformanceClass"
// +kubebuilder:printcolumn:name="ConformanceStatus",type=string,JSONPath=".status.conformanceStatus"
// +kubebuilder:printcolumn:name="Age",type=date,JSONPath=".metadata.creationTimestamp"
type Tier1Service struct {
	// Kubernetes 型メタ情報を埋め込む
	metav1.TypeMeta `json:",inline"`
	// Kubernetes オブジェクトメタ情報を埋め込む
	metav1.ObjectMeta `json:"metadata,omitempty"`
	// Spec: 望ましい状態
	Spec Tier1ServiceSpec `json:"spec,omitempty"`
	// Status: 現在の状態
	Status Tier1ServiceStatus `json:"status,omitempty"`
}

// Tier1ServiceList は Tier1Service のリスト型
// +kubebuilder:object:root=true
type Tier1ServiceList struct {
	// Kubernetes 型メタ情報を埋め込む
	metav1.TypeMeta `json:",inline"`
	// Kubernetes リストメタ情報を埋め込む
	metav1.ListMeta `json:"metadata,omitempty"`
	// Items: Tier1Service のリスト
	Items []Tier1Service `json:"items"`
}

// DeepCopyObject は Tier1Service のディープコピーを返す（runtime.Object インターフェース実装）
func (in *Tier1Service) DeepCopyObject() runtime.Object {
	// コピー先を生成する
	out := new(Tier1Service)
	// TypeMeta / ObjectMeta / Spec は値型なので浅いコピーで OK
	*out = *in
	// Status.Conditions はスライス型なのでディープコピーが必要
	if in.Status.Conditions != nil {
		// スライスを新規アロケートする
		out.Status.Conditions = make([]metav1.Condition, len(in.Status.Conditions))
		// 各要素をコピーする（metav1.Condition は値型なのでコピーで OK）
		copy(out.Status.Conditions, in.Status.Conditions)
	}
	return out
}

// DeepCopyObject は Tier1ServiceList のディープコピーを返す（runtime.Object インターフェース実装）
func (in *Tier1ServiceList) DeepCopyObject() runtime.Object {
	// コピー先を生成する
	out := new(Tier1ServiceList)
	// フィールドをコピーする
	*out = *in
	// Items スライスのディープコピーを実行する
	if in.Items != nil {
		// スライスを新規アロケートする
		out.Items = make([]Tier1Service, len(in.Items))
		// 各 Tier1Service をディープコピーする
		for i := range in.Items {
			// 値型コピーで ObjectMeta/Spec を複製する
			out.Items[i] = in.Items[i]
			// Conditions スライスはディープコピーが必要
			if in.Items[i].Status.Conditions != nil {
				// Conditions を新規アロケートしてコピーする
				out.Items[i].Status.Conditions = make([]metav1.Condition, len(in.Items[i].Status.Conditions))
				copy(out.Items[i].Status.Conditions, in.Items[i].Status.Conditions)
			}
		}
	}
	return out
}
