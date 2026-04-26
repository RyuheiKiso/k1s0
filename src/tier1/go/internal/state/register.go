// 本ファイルは t1-state Pod が gRPC server に登録する 5 サービスのオーケストレータ。
//
// 設計正典:
//   docs/04_概要設計/20_ソフトウェア方式設計/01_コンポーネント方式設計/01_tier1全体コンポーネント俯瞰.md
//     - DS-SW-COMP-005（t1-state: 5 API Router Pod）
//   docs/04_概要設計/20_ソフトウェア方式設計/01_コンポーネント方式設計/02_Daprファサード層コンポーネント.md
//     - DS-SW-COMP-020（5 モジュールパイプライン: API Router → Policy → Dapr Adapter → Log/Telemetry）
//
// 役割:
//   common.Pod.Register hook に渡される func(*grpc.Server) を提供する。
//   5 つの公開 API（ServiceInvoke / State / PubSub / Binding / Feature）の handler 実装を
//   gRPC server に登録する。各 handler は internal/adapter/dapr/ への委譲のみを行う。
//
// scope（リリース時点 最小骨格）:
//   各 handler は adapter を呼び出すが、adapter は ErrNotWired を返すため、
//   利用側は codes.Unimplemented を受け取る。実 Dapr backend 結線は plan 04-04 〜 04-13。

// Package state は t1-state Pod が登録する 5 公開 API のハンドラを提供する。
package state

// 標準 / 内部パッケージを import する。
import (
	// Dapr adapter（本リリース時点 placeholder）。
	"github.com/k1s0/k1s0/src/tier1/go/internal/adapter/dapr"
	// SDK 生成 stub の各 service registration 関数を import する（公開 12 API のうち 5 件）。
	bindingv1 "github.com/k1s0/sdk-go/proto/v1/k1s0/tier1/binding/v1"
	featurev1 "github.com/k1s0/sdk-go/proto/v1/k1s0/tier1/feature/v1"
	pubsubv1 "github.com/k1s0/sdk-go/proto/v1/k1s0/tier1/pubsub/v1"
	serviceinvokev1 "github.com/k1s0/sdk-go/proto/v1/k1s0/tier1/serviceinvoke/v1"
	statev1 "github.com/k1s0/sdk-go/proto/v1/k1s0/tier1/state/v1"
	// gRPC server 型。
	"google.golang.org/grpc"
)

// Deps は t1-state Pod の handler 群が依存する adapter 集合。
// main.go から注入され、各 handler は本 struct のフィールドを参照する。
type Deps struct {
	// Dapr State building block アダプタ。
	StateAdapter dapr.StateAdapter
	// Dapr Pub/Sub building block アダプタ。
	PubSubAdapter dapr.PubSubAdapter
	// Dapr Output Binding building block アダプタ。
	BindingAdapter dapr.BindingAdapter
	// Dapr Service Invocation building block アダプタ。
	InvokeAdapter dapr.InvokeAdapter
	// Feature Flag（flagd 直結）アダプタ。
	FeatureAdapter dapr.FeatureAdapter
}

// NewDepsFromClient は単一の Dapr Client から 5 つのアダプタを構築する。
// main.go の起動シーケンスで使用される（lazy 不可、健全性確認前に初期化必須）。
func NewDepsFromClient(client *dapr.Client) Deps {
	// 各 adapter を Client 共有で生成する。
	return Deps{
		// State Management（Valkey）。
		StateAdapter: dapr.NewStateAdapter(client),
		// Pub/Sub（Kafka）。
		PubSubAdapter: dapr.NewPubSubAdapter(client),
		// Output Binding（外部 HTTP / SMTP / S3）。
		BindingAdapter: dapr.NewBindingAdapter(client),
		// Service Invocation（tier1 内部 gRPC 呼出含む）。
		InvokeAdapter: dapr.NewInvokeAdapter(client),
		// Feature Flag（flagd）。
		FeatureAdapter: dapr.NewFeatureAdapter(client),
	}
}

// Register は 5 つの公開 API handler を gRPC server に登録する hook を返す。
// common.Pod.Register に直接渡せるシグネチャ（func(*grpc.Server)）に整形する。
func Register(deps Deps) func(*grpc.Server) {
	// closure で deps を捕捉し、registerAll を返す。
	return func(srv *grpc.Server) {
		// ServiceInvokeService（FR-T1-INVOKE-001〜005）登録。
		serviceinvokev1.RegisterInvokeServiceServer(srv, &invokeHandler{deps: deps})
		// StateService（FR-T1-STATE-001〜005）登録。
		statev1.RegisterStateServiceServer(srv, &stateHandler{deps: deps})
		// PubSubService（FR-T1-PUBSUB-001〜005）登録。
		pubsubv1.RegisterPubSubServiceServer(srv, &pubsubHandler{deps: deps})
		// BindingService（FR-T1-BINDING-001〜004）登録。
		bindingv1.RegisterBindingServiceServer(srv, &bindingHandler{deps: deps})
		// FeatureService（FR-T1-FEATURE-001〜004）登録。
		featurev1.RegisterFeatureServiceServer(srv, &featureHandler{deps: deps})
	}
}
