// tier1_conformance_types.go — k1s0 tier1 operator: OSSInventory CRD 型定義
// 08_OSSライフサイクル適合仕様.md §lifecycle_class セットに準拠する。
// OSSInventory CRD は tier1 OSS パッケージのライフサイクル管理（lifecycle_class / license 追跡）を行う。

// パッケージ名: v1 API グループ（types.go と同一パッケージに属する）
package v1

import (
	// OSSInventory CRD のメタデータインポート
	metav1 "k8s.io/apimachinery/pkg/apis/meta/v1"
	// ランタイムパッケージ: DeepCopyObject インターフェース実装に使用する
	"k8s.io/apimachinery/pkg/runtime"
)

// OSSInventorySpec は OSSInventory リソースの spec を定義する
// 08_OSSライフサイクル適合仕様.md の OSS パッケージ追跡要件に対応する
type OSSInventorySpec struct {
	// OSS パッケージ名: crate 名 / npm パッケージ名 / Go module パス 等を記載する
	PackageName string `json:"packageName"`
	// ライセンス種別: MIT / Apache-2.0 / GPL-3.0 等の SPDX 表記を使用する
	LicenseType string `json:"licenseType,omitempty"`
	// lifecycle_class: 08_OSSライフサイクル適合仕様.md §lifecycle_class セットの値を指定する
	// L1_active / L2_maintenance / L3_deprecated / L3_eol のいずれかを指定する
	LifecycleClass string `json:"lifecycleClass,omitempty"`
	// バージョン: 使用中の OSS パッケージバージョン（semver 表記）
	Version string `json:"version,omitempty"`
	// dry_run_last_at: OSS ライフサイクル dry_run の最終実行日時（08_OSSライフサイクル適合仕様.md §dry_run）
	// +optional
	DryRunLastAt *metav1.Time `json:"dryRunLastAt,omitempty"`
}

// OSSInventoryStatus は OSSInventory リソースのステータスを定義する
type OSSInventoryStatus struct {
	// ライフサイクル状態が有効であるか: lifecycle_class が L1_active なら true
	// +optional
	Active bool `json:"active,omitempty"`
	// 最終確認時刻: Reconciler が lifecycle 状態を確認した時刻
	// +optional
	LastVerifiedAt *metav1.Time `json:"lastVerifiedAt,omitempty"`
}

// OSSInventory は tier1 OSS ライフサイクル管理の CRD
// +kubebuilder:object:root=true
// +kubebuilder:subresource:status
// +kubebuilder:resource:scope=Namespaced,shortName=ossinv
// +kubebuilder:printcolumn:name="Package",type=string,JSONPath=".spec.packageName"
// +kubebuilder:printcolumn:name="LifecycleClass",type=string,JSONPath=".spec.lifecycleClass"
// +kubebuilder:printcolumn:name="Active",type=boolean,JSONPath=".status.active"
// +kubebuilder:printcolumn:name="Age",type=date,JSONPath=".metadata.creationTimestamp"
type OSSInventory struct {
	// Kubernetes 型メタ情報を埋め込む
	metav1.TypeMeta `json:",inline"`
	// Kubernetes 標準メタデータ
	metav1.ObjectMeta `json:"metadata,omitempty"`
	// OSS パッケージ管理の spec
	Spec OSSInventorySpec `json:"spec,omitempty"`
	// OSS パッケージ管理のステータス
	Status OSSInventoryStatus `json:"status,omitempty"`
}

// OSSInventoryList は OSSInventory リソースの一覧を定義する
// +kubebuilder:object:root=true
type OSSInventoryList struct {
	// Kubernetes 型メタ情報を埋め込む
	metav1.TypeMeta `json:",inline"`
	// リスト標準メタデータ
	metav1.ListMeta `json:"metadata,omitempty"`
	// OSSInventory アイテム一覧
	Items []OSSInventory `json:"items"`
}

// DeepCopyObject は OSSInventory のディープコピーを返す（runtime.Object インターフェース実装）
func (in *OSSInventory) DeepCopyObject() runtime.Object {
	// コピー先を生成する
	out := new(OSSInventory)
	// TypeMeta / ObjectMeta / Spec / Status は値型なので浅いコピーで OK
	*out = *in
	// DryRunLastAt は *metav1.Time（ポインタ型）なのでディープコピーが必要
	if in.Spec.DryRunLastAt != nil {
		// 新しい Time 値を生成してポインタを設定する
		t := *in.Spec.DryRunLastAt
		out.Spec.DryRunLastAt = &t
	}
	// LastVerifiedAt は *metav1.Time（ポインタ型）なのでディープコピーが必要
	if in.Status.LastVerifiedAt != nil {
		// 新しい Time 値を生成してポインタを設定する
		t := *in.Status.LastVerifiedAt
		out.Status.LastVerifiedAt = &t
	}
	// コピーした OSSInventory を返す
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
			// 値型フィールドをコピーする
			out.Items[i] = in.Items[i]
			// DryRunLastAt ポインタをディープコピーする
			if in.Items[i].Spec.DryRunLastAt != nil {
				// 新しい Time 値を生成してポインタを設定する
				t := *in.Items[i].Spec.DryRunLastAt
				out.Items[i].Spec.DryRunLastAt = &t
			}
			// LastVerifiedAt ポインタをディープコピーする
			if in.Items[i].Status.LastVerifiedAt != nil {
				// 新しい Time 値を生成してポインタを設定する
				t := *in.Items[i].Status.LastVerifiedAt
				out.Items[i].Status.LastVerifiedAt = &t
			}
		}
	}
	// コピーした OSSInventoryList を返す
	return out
}
