// tier1_keyclass_types.go — k1s0 tier1 operator: KeyClass CRD 型定義
// 05_鍵管理適合仕様.md §v1 key_class セット（5 class）に準拠する。
// KeyClass CRD はテナントの鍵管理ポリシー（purpose / rotation_cadence / scope / backend / destruction_method）を管理する。

// パッケージ名: v1 API グループ（types.go と同一パッケージに属する）
package v1

import (
	// KeyClass CRD のメタデータインポート
	metav1 "k8s.io/apimachinery/pkg/apis/meta/v1"
	// ランタイムパッケージ: DeepCopyObject インターフェース実装に使用する
	"k8s.io/apimachinery/pkg/runtime"
)

// KeyClassSpec は KeyClass リソースの spec を定義する
// 05_鍵管理適合仕様.md §v1 key_class セットの各 class 値を格納する
type KeyClassSpec struct {
	// 鍵の用途: signing / encryption / mTLS 等のいずれかを指定する（purpose bundle の代表値）
	Purpose string `json:"purpose"`
	// 鍵のローテーション周期: 05_鍵管理適合仕様.md §rotation_cadence に対応する（例: "30d"）
	RotationCadence string `json:"rotationCadence,omitempty"`
	// 鍵のスコープ: per-tenant / platform / per_workload のいずれかを指定する
	Scope string `json:"scope,omitempty"`
	// 鍵管理バックエンド: software_kms_wrapped / hsm_pkcs11 / spire 等を指定する
	Backend string `json:"backend,omitempty"`
	// 鍵の廃棄方法: crypto_shred / hsm_zeroize_all_shares / jwks_revoke 等を指定する
	DestructionMethod string `json:"destructionMethod,omitempty"`
}

// KeyClassStatus は KeyClass リソースのステータスを定義する
type KeyClassStatus struct {
	// ローテーション済みフラグ: 最新のローテーションが完了したら true になる
	// +optional
	Rotated bool `json:"rotated,omitempty"`
	// 最終ローテーション時刻: 最後にローテーションが完了した時刻
	// +optional
	LastRotationAt *metav1.Time `json:"lastRotationAt,omitempty"`
}

// KeyClass は tier1 テナントの鍵管理ポリシーを定義する CRD
// +kubebuilder:object:root=true
// +kubebuilder:subresource:status
// +kubebuilder:resource:scope=Namespaced,shortName=kc
// +kubebuilder:printcolumn:name="Purpose",type=string,JSONPath=".spec.purpose"
// +kubebuilder:printcolumn:name="Rotated",type=boolean,JSONPath=".status.rotated"
// +kubebuilder:printcolumn:name="Age",type=date,JSONPath=".metadata.creationTimestamp"
type KeyClass struct {
	// Kubernetes 型メタ情報を埋め込む
	metav1.TypeMeta `json:",inline"`
	// Kubernetes 標準メタデータ
	metav1.ObjectMeta `json:"metadata,omitempty"`
	// 鍵管理ポリシーの spec
	Spec KeyClassSpec `json:"spec,omitempty"`
	// 鍵管理ポリシーのステータス
	Status KeyClassStatus `json:"status,omitempty"`
}

// KeyClassList は KeyClass リソースの一覧を定義する
// +kubebuilder:object:root=true
type KeyClassList struct {
	// Kubernetes 型メタ情報を埋め込む
	metav1.TypeMeta `json:",inline"`
	// リスト標準メタデータ
	metav1.ListMeta `json:"metadata,omitempty"`
	// KeyClass アイテム一覧
	Items []KeyClass `json:"items"`
}

// DeepCopyObject は KeyClass のディープコピーを返す（runtime.Object インターフェース実装）
func (in *KeyClass) DeepCopyObject() runtime.Object {
	// コピー先を生成する
	out := new(KeyClass)
	// TypeMeta / ObjectMeta / Spec / Status は値型なので浅いコピーで OK
	*out = *in
	// LastRotationAt は *metav1.Time（ポインタ型）なのでディープコピーが必要
	if in.Status.LastRotationAt != nil {
		// 新しい Time 値を生成してポインタを設定する
		t := *in.Status.LastRotationAt
		out.Status.LastRotationAt = &t
	}
	// コピーした KeyClass を返す
	return out
}

// DeepCopyObject は KeyClassList のディープコピーを返す（runtime.Object インターフェース実装）
func (in *KeyClassList) DeepCopyObject() runtime.Object {
	// コピー先を生成する
	out := new(KeyClassList)
	// フィールドをコピーする
	*out = *in
	// Items スライスのディープコピーを実行する
	if in.Items != nil {
		// スライスを新規アロケートする
		out.Items = make([]KeyClass, len(in.Items))
		// 各 KeyClass をディープコピーする
		for i := range in.Items {
			// 値型フィールドをコピーする
			out.Items[i] = in.Items[i]
			// LastRotationAt ポインタをディープコピーする
			if in.Items[i].Status.LastRotationAt != nil {
				// 新しい Time 値を生成してポインタを設定する
				t := *in.Items[i].Status.LastRotationAt
				out.Items[i].Status.LastRotationAt = &t
			}
		}
	}
	// コピーした KeyClassList を返す
	return out
}
