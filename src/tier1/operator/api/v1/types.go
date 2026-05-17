// k1s0 tier1 operator の CRD 型定義
// Tier1Service リソースの Spec / Status を定義する

// パッケージ名: v1 API グループ
package v1

import (
	// Kubernetes API マシナリーのメタ情報パッケージ
	metav1 "k8s.io/apimachinery/pkg/apis/meta/v1"
	// ランタイムパッケージ: DeepCopyObject インターフェース実装に使用する
	"k8s.io/apimachinery/pkg/runtime"
)

// Tier1ServiceSpec は Tier1Service の望ましい状態を定義する
type Tier1ServiceSpec struct {
	// ConformanceClass: 適用する Bidi conformance class（例: v1_bidi_streaming）
	ConformanceClass string `json:"conformanceClass"`
	// Adapter: 使用する transport adapter（例: grpc_go）
	Adapter string `json:"adapter"`
	// QuotaClass: テナント容量クラス（例: standard / premium）
	QuotaClass string `json:"quotaClass"`
	// SloClass: SLO クラス（例: v1_high_availability）
	SloClass string `json:"sloClass"`
	// Replicas: Pod のレプリカ数
	Replicas int32 `json:"replicas"`
}

// Tier1ServiceStatus は Tier1Service の現在の状態を定義する
type Tier1ServiceStatus struct {
	// ConformanceStatus: conformance テストの結果（green / red / pending）
	ConformanceStatus string `json:"conformanceStatus,omitempty"`
	// LastReconciledAt: 最後に Reconcile した日時
	LastReconciledAt string `json:"lastReconciledAt,omitempty"`
	// Conditions: 詳細な条件リスト
	Conditions []metav1.Condition `json:"conditions,omitempty"`
}

// Tier1Service は tier1 サービスを表す CRD の Go 型定義
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
	// フィールドをコピーする
	*out = *in
	// Items のコピーは不要（Tier1Service は単体オブジェクト）
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
		// 各要素をコピーする
		copy(out.Items, in.Items)
	}
	return out
}
