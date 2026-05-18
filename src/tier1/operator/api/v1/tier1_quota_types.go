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

// QuotaClassSpec は QuotaClass リソースの spec を定義する
type QuotaClassSpec struct {
	// API リクエスト上限 (1 時間あたり): 09_テナント容量適合仕様.md §api_requests_per_hour に対応する
	ApiRequestsPerHour int64 `json:"apiRequestsPerHour,omitempty"`
	// DB コネクション数の上限: 09_テナント容量適合仕様.md §db_connections_max に対応する
	DbConnectionsMax int32 `json:"dbConnectionsMax,omitempty"`
	// ストレージ上限 (GB): 09_テナント容量適合仕様.md §storage_gb_max に対応する
	StorageGBMax int64 `json:"storageGBMax,omitempty"`
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
