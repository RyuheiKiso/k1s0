// src/sdk/go/k1s0/test-fixtures/mock_builder.go
//
// k1s0 Go SDK test-fixtures: Mock builder fluent API（領域 3、ADR-TEST-010 §3）。
// リリース時点で 3 service（State / Audit / PubSub）の builder を提供。
// 採用初期で +3 (Workflow / Decision / Secret)、運用拡大時で残 6 service を追加する。
package testfixtures

// MockBuilderRoot は 12 service の mock builder への entry point
type MockBuilderRoot struct {
	// defaultTenant は WithTenant 未指定時の既定 tenant
	defaultTenant string
}

// newMockBuilderRoot は Setup 内から呼ばれる constructor
func newMockBuilderRoot(defaultTenant string) *MockBuilderRoot {
	return &MockBuilderRoot{defaultTenant: defaultTenant}
}

// State は State service の mock builder を返す（fluent chain entry）
func (r *MockBuilderRoot) State() *StateMockBuilder {
	return &StateMockBuilder{tenant: r.defaultTenant}
}

// Audit は Audit service の mock builder を返す
func (r *MockBuilderRoot) Audit() *AuditMockBuilder {
	return &AuditMockBuilder{tenant: r.defaultTenant}
}

// PubSub は PubSub service の mock builder を返す
func (r *MockBuilderRoot) PubSub() *PubSubMockBuilder {
	return &PubSubMockBuilder{tenant: r.defaultTenant}
}

// Workflow は採用初期で実装する Workflow service mock builder（panic で警告）
func (r *MockBuilderRoot) Workflow() *workflowMockBuilder {
	panic("ADR-TEST-010 PHASE: Workflow mock builder は採用初期で実装、リリース時点未対応")
}

// Decision は採用初期で実装
func (r *MockBuilderRoot) Decision() *decisionMockBuilder {
	panic("ADR-TEST-010 PHASE: Decision mock builder は採用初期で実装、リリース時点未対応")
}

// Secret は採用初期で実装
func (r *MockBuilderRoot) Secret() *secretMockBuilder {
	panic("ADR-TEST-010 PHASE: Secret mock builder は採用初期で実装、リリース時点未対応")
}

// Pii / Feature / Telemetry / Log / Binding / Invoke は運用拡大時で実装（同パタン）
func (r *MockBuilderRoot) Pii() *unimplementedMockBuilder {
	panic("ADR-TEST-010 PHASE: Pii mock builder は運用拡大時で実装")
}

// State / Audit / PubSub の builder fluent chain ----------------------

// StateMockBuilder は State service mock data の fluent builder
type StateMockBuilder struct {
	tenant string
	key    string
	value  []byte
	ttl    int
}

// WithTenant は tenant 指定で builder を更新（fluent chain）
func (b *StateMockBuilder) WithTenant(tenant string) *StateMockBuilder {
	b.tenant = tenant
	return b
}

// WithKey は key 指定
func (b *StateMockBuilder) WithKey(key string) *StateMockBuilder {
	b.key = key
	return b
}

// WithValue は value 指定（byte slice）
func (b *StateMockBuilder) WithValue(value []byte) *StateMockBuilder {
	b.value = value
	return b
}

// WithTTL は TTL 秒指定
func (b *StateMockBuilder) WithTTL(seconds int) *StateMockBuilder {
	b.ttl = seconds
	return b
}

// Build は最終的な StateEntry struct を返す（採用初期で contracts/proto 生成型に置換）
func (b *StateMockBuilder) Build() *StateEntry {
	return &StateEntry{
		Tenant: b.tenant,
		Key:    b.key,
		Value:  b.value,
		TTL:    b.ttl,
	}
}

// StateEntry は State service の wire 形式（採用初期で contracts/proto 型に置換）
type StateEntry struct {
	Tenant string
	Key    string
	Value  []byte
	TTL    int
}

// AuditMockBuilder は Audit service mock data の fluent builder
type AuditMockBuilder struct {
	tenant     string
	entryCount int
	startSeq   int
}

// WithTenant は tenant 指定
func (b *AuditMockBuilder) WithTenant(tenant string) *AuditMockBuilder {
	b.tenant = tenant
	return b
}

// WithEntries は entry 件数指定（hash chain で連結された N 件を生成）
func (b *AuditMockBuilder) WithEntries(n int) *AuditMockBuilder {
	b.entryCount = n
	return b
}

// WithSequence は開始 sequence 番号指定
func (b *AuditMockBuilder) WithSequence(seq int) *AuditMockBuilder {
	b.startSeq = seq
	return b
}

// Build は AuditEntry の slice を返す（採用初期で hash chain 計算 + contracts/proto 型に置換）
func (b *AuditMockBuilder) Build() []*AuditEntry {
	entries := make([]*AuditEntry, b.entryCount)
	for i := 0; i < b.entryCount; i++ {
		entries[i] = &AuditEntry{
			Tenant:   b.tenant,
			Sequence: b.startSeq + i,
			// 採用初期で prev_id chain を SHA-256 で計算する
			PrevID: "",
		}
	}
	return entries
}

// AuditEntry は Audit service の wire 形式（採用初期で contracts/proto 型に置換）
type AuditEntry struct {
	Tenant   string
	Sequence int
	PrevID   string
}

// PubSubMockBuilder は PubSub service mock data の fluent builder
type PubSubMockBuilder struct {
	tenant      string
	topic       string
	messages    int
	delayMs     int
}

// WithTenant は tenant 指定
func (b *PubSubMockBuilder) WithTenant(tenant string) *PubSubMockBuilder {
	b.tenant = tenant
	return b
}

// WithTopic は topic 指定
func (b *PubSubMockBuilder) WithTopic(topic string) *PubSubMockBuilder {
	b.topic = topic
	return b
}

// WithMessages は publish する message 数
func (b *PubSubMockBuilder) WithMessages(n int) *PubSubMockBuilder {
	b.messages = n
	return b
}

// WithDelayMs は publish 間隔（ミリ秒）
func (b *PubSubMockBuilder) WithDelayMs(ms int) *PubSubMockBuilder {
	b.delayMs = ms
	return b
}

// Build は PubSub message の slice を返す
func (b *PubSubMockBuilder) Build() []*PubSubMessage {
	msgs := make([]*PubSubMessage, b.messages)
	for i := 0; i < b.messages; i++ {
		msgs[i] = &PubSubMessage{
			Tenant: b.tenant,
			Topic:  b.topic,
			SeqID:  i,
		}
	}
	return msgs
}

// PubSubMessage は PubSub service の wire 形式
type PubSubMessage struct {
	Tenant string
	Topic  string
	SeqID  int
}

// 採用初期で実装する builder の placeholder type ----------------------

type workflowMockBuilder struct{}
type decisionMockBuilder struct{}
type secretMockBuilder struct{}
type unimplementedMockBuilder struct{}
