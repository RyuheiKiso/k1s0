// 本ファイルは k1s0 Go SDK の高水準ファサード Client。
//
// docs 正典:
//   docs/05_実装/10_ビルド設計/20_Go_module分離戦略/01_Go_module分離戦略.md
//   README.md のサンプルコードに準拠（k1s0.State.Save / k1s0.PubSub.Publish 等の動詞統一）
//
// scope（リリース時点 最小、3 代表 service）:
//   - Client.State() → StateClient（Get / Save / Delete）
//   - Client.PubSub() → PubSubClient（Publish）
//   - Client.Secrets() → SecretsClient（Get / Rotate）
//   - Client.Raw() → 12 service すべての生成 stub クライアントへの直接アクセス
//
// 残り 9 service（Invoke / Workflow / Log / Telemetry / Decision / Audit / Pii / Feature /
// Binding）は本リリース時点 では Raw() 経由で利用、動詞統一 facade はロードマップ #8 後続で追加。

// Package k1s0 は tier1 公開 12 API への高水準クライアントを提供する。
package k1s0

// 標準 / 内部 import。
import (
	// gRPC 接続設定。
	"crypto/tls"
	// context 伝搬。
	"context"
	// gRPC ランタイム。
	"google.golang.org/grpc"
	// gRPC 認証情報（TLS / insecure）。
	"google.golang.org/grpc/credentials"
	// 平文 gRPC（local-stack / dev 用）。
	"google.golang.org/grpc/credentials/insecure"

	// SDK 生成 stub の 12 service クライアントを参照する。
	auditv1 "github.com/k1s0/sdk-go/proto/v1/k1s0/tier1/audit/v1"
	bindingv1 "github.com/k1s0/sdk-go/proto/v1/k1s0/tier1/binding/v1"
	decisionv1 "github.com/k1s0/sdk-go/proto/v1/k1s0/tier1/decision/v1"
	featurev1 "github.com/k1s0/sdk-go/proto/v1/k1s0/tier1/feature/v1"
	logv1 "github.com/k1s0/sdk-go/proto/v1/k1s0/tier1/log/v1"
	piiv1 "github.com/k1s0/sdk-go/proto/v1/k1s0/tier1/pii/v1"
	pubsubv1 "github.com/k1s0/sdk-go/proto/v1/k1s0/tier1/pubsub/v1"
	secretsv1 "github.com/k1s0/sdk-go/proto/v1/k1s0/tier1/secrets/v1"
	serviceinvokev1 "github.com/k1s0/sdk-go/proto/v1/k1s0/tier1/serviceinvoke/v1"
	statev1 "github.com/k1s0/sdk-go/proto/v1/k1s0/tier1/state/v1"
	telemetryv1 "github.com/k1s0/sdk-go/proto/v1/k1s0/tier1/telemetry/v1"
	workflowv1 "github.com/k1s0/sdk-go/proto/v1/k1s0/tier1/workflow/v1"
)

// Config は Client 初期化時に渡す設定。
type Config struct {
	// gRPC 接続先（例: "tier1-state.tier1-facade.svc.cluster.local:50001"）。
	Target string
	// テナント ID（全 RPC の TenantContext.tenant_id に自動付与）。
	TenantID string
	// 主体識別子（subject）。
	Subject string
	// TLS を使うか（本番は true 必須、dev は false で平文）。
	UseTLS bool
	// 追加の DialOption（OTel interceptor / retry 等を渡す用途）。
	DialOptions []grpc.DialOption
}

// Client は 12 service へのアクセス起点。
type Client struct {
	// gRPC 接続。Close で解放する。
	conn *grpc.ClientConn
	// 自動付与する TenantContext 情報。
	cfg Config
	// 12 service の生成 stub クライアント（Raw() で全件アクセス可）。
	raw RawClients
	// 動詞統一 facade（12 公開 + 2 admin の 14 service）。
	state         *StateClient
	pubsub        *PubSubClient
	secrets       *SecretsClient
	log           *LogClient
	workflow      *WorkflowClient
	decision      *DecisionClient
	decisionAdmin *DecisionAdminClient
	audit         *AuditClient
	pii           *PiiClient
	feature       *FeatureClient
	featureAdmin  *FeatureAdminClient
	binding       *BindingClient
	invoke        *InvokeClient
	telemetry     *TelemetryClient
}

// RawClients は 12 service すべての生成 stub クライアントを保持する。
// 高水準 facade が未実装の service にアクセスしたい場合は Client.Raw() 経由で本構造体を使う。
type RawClients struct {
	// 各 service の生成 client（README サンプルでは facade 経由を推奨）。
	Audit         auditv1.AuditServiceClient
	Binding       bindingv1.BindingServiceClient
	Decision      decisionv1.DecisionServiceClient
	DecisionAdmin decisionv1.DecisionAdminServiceClient
	Feature       featurev1.FeatureServiceClient
	FeatureAdmin  featurev1.FeatureAdminServiceClient
	Log           logv1.LogServiceClient
	Pii           piiv1.PiiServiceClient
	PubSub        pubsubv1.PubSubServiceClient
	Secrets       secretsv1.SecretsServiceClient
	ServiceInvoke serviceinvokev1.InvokeServiceClient
	State         statev1.StateServiceClient
	Telemetry     telemetryv1.TelemetryServiceClient
	Workflow      workflowv1.WorkflowServiceClient
}

// New は Config から Client を生成する。
func New(ctx context.Context, cfg Config) (*Client, error) {
	// DialOption を組み立てる。
	opts := []grpc.DialOption{}
	// TLS 有無で transport credentials を切り替える。
	if cfg.UseTLS {
		// production: TLS 1.3 strict（証明書は SPIRE / cert-manager 経由で配布）
		opts = append(opts, grpc.WithTransportCredentials(credentials.NewTLS(&tls.Config{MinVersion: tls.VersionTLS13})))
	} else {
		// dev: 平文。
		opts = append(opts, grpc.WithTransportCredentials(insecure.NewCredentials()))
	}
	// 利用者の追加 DialOption（OTel interceptor 等）を末尾に追加。
	opts = append(opts, cfg.DialOptions...)
	// gRPC 接続を確立する。
	conn, err := grpc.NewClient(cfg.Target, opts...)
	// 接続失敗時は呼び出し元に error を返す。
	if err != nil {
		// caller に伝搬する。
		return nil, err
	}
	// Client インスタンスを構築する。
	c := &Client{conn: conn, cfg: cfg}
	// RawClients に 12 service の生成 client を詰める。
	c.raw = RawClients{
		Audit:         auditv1.NewAuditServiceClient(conn),
		Binding:       bindingv1.NewBindingServiceClient(conn),
		Decision:      decisionv1.NewDecisionServiceClient(conn),
		DecisionAdmin: decisionv1.NewDecisionAdminServiceClient(conn),
		Feature:       featurev1.NewFeatureServiceClient(conn),
		FeatureAdmin:  featurev1.NewFeatureAdminServiceClient(conn),
		Log:           logv1.NewLogServiceClient(conn),
		Pii:           piiv1.NewPiiServiceClient(conn),
		PubSub:        pubsubv1.NewPubSubServiceClient(conn),
		Secrets:       secretsv1.NewSecretsServiceClient(conn),
		ServiceInvoke: serviceinvokev1.NewInvokeServiceClient(conn),
		State:         statev1.NewStateServiceClient(conn),
		Telemetry:     telemetryv1.NewTelemetryServiceClient(conn),
		Workflow:      workflowv1.NewWorkflowServiceClient(conn),
	}
	// 動詞統一 facade を初期化する。
	c.state = &StateClient{client: c}
	c.pubsub = &PubSubClient{client: c}
	c.secrets = &SecretsClient{client: c}
	c.log = &LogClient{client: c}
	c.workflow = &WorkflowClient{client: c}
	c.decision = &DecisionClient{client: c}
	c.audit = &AuditClient{client: c}
	c.pii = &PiiClient{client: c}
	c.feature = &FeatureClient{client: c}
	c.binding = &BindingClient{client: c}
	c.invoke = &InvokeClient{client: c}
	c.telemetry = &TelemetryClient{client: c}
	c.decisionAdmin = &DecisionAdminClient{client: c}
	c.featureAdmin = &FeatureAdminClient{client: c}
	// 構築済 Client を返却する。
	return c, nil
}

// Close は gRPC 接続を解放する。Pod 終了時に呼ぶ。
func (c *Client) Close() error {
	// nil ガード。
	if c == nil || c.conn == nil {
		// 何もせず nil を返却する。
		return nil
	}
	// gRPC 接続を閉じる。
	return c.conn.Close()
}

// State は StateClient（Get / Save / Delete の動詞統一 facade）を返す。
func (c *Client) State() *StateClient {
	// 初期化済の facade を返却する。
	return c.state
}

// PubSub は PubSubClient（Publish の動詞統一 facade）を返す。
func (c *Client) PubSub() *PubSubClient {
	// 初期化済の facade を返却する。
	return c.pubsub
}

// Secrets は SecretsClient（Get / Rotate の動詞統一 facade）を返す。
func (c *Client) Secrets() *SecretsClient {
	// 初期化済の facade を返却する。
	return c.secrets
}

// Raw は 12 service すべての生成 stub クライアント集合を返す。
// 動詞統一 facade が未実装の service（Workflow / Log / Telemetry / Decision / Audit /
// Pii / Feature / Binding / Invoke）はこちらを使う。
func (c *Client) Raw() RawClients {
	// 構造体をそのまま返却する（コピー、内部の client は同 conn を共有するため安全）。
	return c.raw
}
