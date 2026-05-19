// tier1_sloclass_types.go — k1s0 tier1 operator: SLOClass CRD 型定義
// 07_SLO適合仕様.md §v1 slo_class セット（6 class）に準拠する。
// SLOClass CRD は SLI 算出 / PrometheusRule 生成 / MWMBR burn rate alert の設定を管理する。

// パッケージ名: v1 API グループ（types.go と同一パッケージに属する）
package v1

import (
	// SLOClass CRD のメタデータインポート
	metav1 "k8s.io/apimachinery/pkg/apis/meta/v1"
	// ランタイムパッケージ: DeepCopyObject インターフェース実装に使用する
	"k8s.io/apimachinery/pkg/runtime"
)

// SloClassID は slo_class の 6 class セットを宣言する (spec 07 §slo_class)
type SloClassID string

const (
	// V1SloRequestAvailability: HTTP/gRPC API 可用性 SLO 99.9%（30d window）
	V1SloRequestAvailability SloClassID = "v1_request_availability"
	// V1SloRequestLatencyP99: request 経路 p99 latency SLO
	V1SloRequestLatencyP99 SloClassID = "v1_request_latency_p99"
	// V1SloRequestLatencyP99CrossRegion: cross-region write p99 latency SLO
	V1SloRequestLatencyP99CrossRegion SloClassID = "v1_request_latency_p99_cross_region"
	// V1SloEventFreshness: server-driven event 配信 freshness SLO（p95 ≤ target、7d window）
	V1SloEventFreshness SloClassID = "v1_event_freshness"
	// V1SloWorkflowCompletion: Temporal Workflow / Saga 完了率 SLO（99.5%、30d window）
	V1SloWorkflowCompletion SloClassID = "v1_workflow_completion"
	// V1SloDataDurability: tenant data 永続性 SLO（11 nines）
	V1SloDataDurability SloClassID = "v1_data_durability"
)

// BurnRateWindow は MWMBR アラートで使用する burn rate window を宣言する
type BurnRateWindow struct {
	// Short は短期 burn rate window（例: "5m"）
	Short string `json:"short"`
	// Long は長期 burn rate window（例: "1h"）
	Long string `json:"long"`
	// BurnRateThreshold は burn rate がこの値を超えたらアラートを発火する
	BurnRateThreshold float64 `json:"burnRateThreshold"`
}

// SLOClassSpec は SLOClass リソースの spec を定義する (spec 07 §v1 slo_class)
type SLOClassSpec struct {
	// SloClass: SLO クラス識別子（6 class セット）
	// +kubebuilder:validation:Enum=v1_request_availability;v1_request_latency_p99;v1_request_latency_p99_cross_region;v1_event_freshness;v1_workflow_completion;v1_data_durability
	SloClass SloClassID `json:"sloClass"`
	// Target: SLO 目標値（0.0–1.0）例: 0.999 は 99.9%
	// +kubebuilder:validation:Minimum=0.0
	// +kubebuilder:validation:Maximum=1.0
	Target float64 `json:"target"`
	// Window: SLO 計測ウィンドウ（例: "30d" / "7d"）
	Window string `json:"window,omitempty"`
	// LatencyThresholdMs: latency SLO の場合の閾値（ms）
	// +optional
	LatencyThresholdMs int32 `json:"latencyThresholdMs,omitempty"`
	// BurnRateWindows: MWMBR burn rate アラートの window / threshold セット（通常 4 window）
	// +optional
	BurnRateWindows []BurnRateWindow `json:"burnRateWindows,omitempty"`
	// ServiceSelector: PrometheusRule を適用するサービスの label selector
	// +optional
	ServiceSelector map[string]string `json:"serviceSelector,omitempty"`
}

// SLOClassStatus は SLOClass リソースのステータスを定義する
type SLOClassStatus struct {
	// PrometheusRuleApplied: PrometheusRule が正常に apply されたか
	// +optional
	PrometheusRuleApplied bool `json:"prometheusRuleApplied,omitempty"`
	// FreezePolicyActive: error budget burn で freeze policy が発動中か
	// +optional
	FreezePolicyActive bool `json:"freezePolicyActive,omitempty"`
	// LastReconcileAt: 最後に Reconcile した時刻
	// +optional
	LastReconcileAt *metav1.Time `json:"lastReconcileAt,omitempty"`
	// Conditions: 標準 Kubernetes 条件リスト
	// +optional
	// +listType=map
	// +listMapKey=type
	Conditions []metav1.Condition `json:"conditions,omitempty"`
}

// SLOClass は tier1 SLO クラスを定義する CRD
// +kubebuilder:object:root=true
// +kubebuilder:subresource:status
// +kubebuilder:resource:scope=Namespaced,shortName=sloclass
// +kubebuilder:printcolumn:name="SloClass",type=string,JSONPath=".spec.sloClass"
// +kubebuilder:printcolumn:name="Target",type=number,JSONPath=".spec.target"
// +kubebuilder:printcolumn:name="RuleApplied",type=boolean,JSONPath=".status.prometheusRuleApplied"
// +kubebuilder:printcolumn:name="Age",type=date,JSONPath=".metadata.creationTimestamp"
type SLOClass struct {
	// Kubernetes 型メタ情報を埋め込む
	metav1.TypeMeta `json:",inline"`
	// Kubernetes 標準メタデータ
	metav1.ObjectMeta `json:"metadata,omitempty"`
	// SLO クラスの spec
	Spec SLOClassSpec `json:"spec,omitempty"`
	// SLO クラスのステータス
	Status SLOClassStatus `json:"status,omitempty"`
}

// SLOClassList は SLOClass リソースの一覧を定義する
// +kubebuilder:object:root=true
type SLOClassList struct {
	// Kubernetes 型メタ情報を埋め込む
	metav1.TypeMeta `json:",inline"`
	// リスト標準メタデータ
	metav1.ListMeta `json:"metadata,omitempty"`
	// SLOClass アイテム一覧
	Items []SLOClass `json:"items"`
}

// DeepCopyObject は SLOClass のディープコピーを返す（runtime.Object インターフェース実装）
func (in *SLOClass) DeepCopyObject() runtime.Object {
	// コピー先を生成する
	out := new(SLOClass)
	// 値型フィールドをコピーする
	*out = *in
	// LastReconcileAt ポインタをディープコピーする
	if in.Status.LastReconcileAt != nil {
		// 新しい Time 値を生成してポインタを設定する
		t := *in.Status.LastReconcileAt
		// ポインタを設定する
		out.Status.LastReconcileAt = &t
	}
	// Conditions スライスをディープコピーする
	if in.Status.Conditions != nil {
		// スライスを新規アロケートする
		out.Status.Conditions = make([]metav1.Condition, len(in.Status.Conditions))
		// 各要素をコピーする
		copy(out.Status.Conditions, in.Status.Conditions)
	}
	// BurnRateWindows スライスをディープコピーする
	if in.Spec.BurnRateWindows != nil {
		// スライスを新規アロケートする
		out.Spec.BurnRateWindows = make([]BurnRateWindow, len(in.Spec.BurnRateWindows))
		// 各要素をコピーする
		copy(out.Spec.BurnRateWindows, in.Spec.BurnRateWindows)
	}
	// ServiceSelector マップをディープコピーする
	if in.Spec.ServiceSelector != nil {
		// マップを新規アロケートする
		out.Spec.ServiceSelector = make(map[string]string, len(in.Spec.ServiceSelector))
		// 各エントリをコピーする
		for k, v := range in.Spec.ServiceSelector {
			// キーと値をコピーする
			out.Spec.ServiceSelector[k] = v
		}
	}
	// コピーした SLOClass を返す
	return out
}

// DeepCopyObject は SLOClassList のディープコピーを返す（runtime.Object インターフェース実装）
func (in *SLOClassList) DeepCopyObject() runtime.Object {
	// コピー先を生成する
	out := new(SLOClassList)
	// 値型フィールドをコピーする
	*out = *in
	// Items スライスのディープコピーを実行する
	if in.Items != nil {
		// スライスを新規アロケートする
		out.Items = make([]SLOClass, len(in.Items))
		// 各 SLOClass を DeepCopy する（DeepCopyObject を再利用する）
		for i := range in.Items {
			// DeepCopyObject でコピーする
			copied := in.Items[i].DeepCopyObject().(*SLOClass)
			// コピー結果をスライスに設定する
			out.Items[i] = *copied
		}
	}
	// コピーした SLOClassList を返す
	return out
}
