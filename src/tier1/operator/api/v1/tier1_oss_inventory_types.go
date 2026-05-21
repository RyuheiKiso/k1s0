// k1s0 tier1 operator の OSSInventory CRD 型定義
// OSS ライフサイクル管理リソース（08_OSSライフサイクル適合仕様.md §lifecycle_class セット対応）
// +groupName=k1s0.io
package v1

import (
	// Kubernetes API マシナリーのメタ情報パッケージ
	metav1 "k8s.io/apimachinery/pkg/apis/meta/v1"
	// ランタイムパッケージ: DeepCopyObject インターフェース実装に使用する
	"k8s.io/apimachinery/pkg/runtime"
)

// OSSInventorySpec は OSSInventory の望ましい状態を定義する
type OSSInventorySpec struct {
	// OssId: OSS パッケージ識別子（oss_inventory_input.yaml の oss_id と一致させる）
	OssId string `json:"ossId"`
	// Ecosystem: エコシステム種別（rust / go / container / kubernetes 等）
	Ecosystem string `json:"ecosystem"`
	// LifecycleClass: lifecycle_class（08_OSSライフサイクル適合仕様.md §v1 lifecycle_class セット）
	// +kubebuilder:validation:Enum=v1_l1plus_primary;v1_l1plus_pair_target;v1_l2star_member;v1_l3_runtime;v1_reserved_category;v1_inhouse_authoritative
	LifecycleClass string `json:"lifecycleClass"`
	// Version: 現行採用バージョン（semver 表記）
	Version string `json:"version"`
	// SignalSourceEndpoint: lifecycle signal 収集元エンドポイント URL
	// +optional
	SignalSourceEndpoint string `json:"signalSourceEndpoint,omitempty"`
}

// OSSInventoryStatus は OSSInventory の現在の状態を定義する
type OSSInventoryStatus struct {
	// DrillState: ドリル実行状態（green / yellow / red / pending）
	// +optional
	DrillState string `json:"drillState,omitempty"`
	// HealthCheckLastAt: lifecycle signal ヘルスチェックの最終実施日時（RFC3339）
	// +optional
	HealthCheckLastAt string `json:"healthCheckLastAt,omitempty"`
	// CosignSignatureVerifiedAt: cosign 署名検証の最終実施日時（RFC3339）
	// +optional
	CosignSignatureVerifiedAt string `json:"cosignSignatureVerifiedAt,omitempty"`
	// Notes: ドリル実行メモ
	// +optional
	Notes string `json:"notes,omitempty"`
	// SbomHash: SBOM の SHA-256 ハッシュ
	// +optional
	SbomHash string `json:"sbomHash,omitempty"`
	// Conditions: Kubernetes 標準の詳細条件リスト
	// +optional
	// +listType=map
	// +listMapKey=type
	Conditions []metav1.Condition `json:"conditions,omitempty"`
}

// OSSInventory は OSS ライフサイクル管理リソースを表す CRD の Go 型定義
// +kubebuilder:object:root=true
// +kubebuilder:subresource:status
// +kubebuilder:resource:scope=Cluster,shortName=ossinv
// +kubebuilder:printcolumn:name="OssId",type=string,JSONPath=".spec.ossId"
// +kubebuilder:printcolumn:name="LifecycleClass",type=string,JSONPath=".spec.lifecycleClass"
// +kubebuilder:printcolumn:name="DrillState",type=string,JSONPath=".status.drillState"
// +kubebuilder:printcolumn:name="Age",type=date,JSONPath=".metadata.creationTimestamp"
type OSSInventory struct {
	// Kubernetes 型メタ情報を埋め込む
	metav1.TypeMeta `json:",inline"`
	// Kubernetes オブジェクトメタ情報を埋め込む
	metav1.ObjectMeta `json:"metadata,omitempty"`
	// Spec: 望ましい状態
	Spec OSSInventorySpec `json:"spec,omitempty"`
	// Status: 現在の状態
	Status OSSInventoryStatus `json:"status,omitempty"`
}

// OSSInventoryList は OSSInventory のリスト型
// +kubebuilder:object:root=true
type OSSInventoryList struct {
	// Kubernetes 型メタ情報を埋め込む
	metav1.TypeMeta `json:",inline"`
	// Kubernetes リストメタ情報を埋め込む
	metav1.ListMeta `json:"metadata,omitempty"`
	// Items: OSSInventory のリスト
	Items []OSSInventory `json:"items"`
}

// DeepCopyObject は OSSInventory のディープコピーを返す（runtime.Object インターフェース実装）
func (in *OSSInventory) DeepCopyObject() runtime.Object {
	// コピー先を生成する
	out := new(OSSInventory)
	// TypeMeta / ObjectMeta / Spec は値型なのでシャローコピーで OK
	*out = *in
	// Status.Conditions はスライス型なのでディープコピーが必要
	if in.Status.Conditions != nil {
		// Conditions スライスを新規アロケートする
		out.Status.Conditions = make([]metav1.Condition, len(in.Status.Conditions))
		// 各要素をコピーする（metav1.Condition は値型なのでコピーで OK）
		copy(out.Status.Conditions, in.Status.Conditions)
	}
	return out
}

// DeepCopyObject は OSSInventoryList のディープコピーを返す（runtime.Object インターフェース実装）
func (in *OSSInventoryList) DeepCopyObject() runtime.Object {
	// コピー先を生成する
	out := new(OSSInventoryList)
	// フィールドをコピーする
	*out = *in
	// Items スライスのディープコピーを実行する
	if in.Items != nil {
		// スライスを新規アロケートする
		out.Items = make([]OSSInventory, len(in.Items))
		// 各 OSSInventory をディープコピーする
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
