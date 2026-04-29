// 本ファイルは tier1 facade から t1-audit Pod へ AuditEvent を gRPC で送る emitter。
//
// 設計正典:
//   docs/03_要件定義/20_機能要件/40_tier1_API契約IDL/10_Audit_Pii_API.md
//   docs/04_概要設計/20_ソフトウェア方式設計/01_コンポーネント方式設計/01_tier1全体コンポーネント俯瞰.md
//     - DS-SW-COMP-007（t1-audit Pod、AuditService.Record gRPC）
//
// 役割:
//   audit.go の AuditInterceptor が呼ぶ AuditEmitter を gRPC client backed で実装し、
//   特権操作の event を t1-audit Pod の AuditService.Record に転送する。
//   非同期キュー + worker goroutine で fire-and-forget を担保し、handler パスに
//   audit emit のレイテンシが乗らないようにする。
//
// 利用例（cmd/state/main.go 等）:
//   emitter, err := common.NewGRPCAuditEmitter(ctx, "t1-audit:50001")
//   if err != nil { ... }
//   defer emitter.Close()
//   srv := grpc.NewServer(
//       grpc.ChainUnaryInterceptor(
//           common.AuthInterceptor(...),
//           common.RateLimitInterceptor(...),
//           common.ObservabilityInterceptor(),
//           common.AuditInterceptor(emitter),
//       ),
//   )
//
// Fail-soft 方針:
//   - キュー溢れ時は drop + warn ログ（handler 完了は止めない）
//   - gRPC 送信失敗時も drop + warn ログ（外部依存の不安定性が tier1 SLO を毀損しない）
//   - Close 時は in-flight queue を flush_timeout 内で送り切るよう試みる

package common

import (
	"context"
	"log"
	"sync"
	"time"

	"google.golang.org/grpc"
	"google.golang.org/grpc/credentials/insecure"
	"google.golang.org/protobuf/types/known/timestamppb"

	auditv1 "github.com/k1s0/sdk-go/proto/v1/k1s0/tier1/audit/v1"
	commonv1 "github.com/k1s0/sdk-go/proto/v1/k1s0/tier1/common/v1"
)

// gRPCAuditEmitter は AuditService.Record を gRPC で叩く emitter。
type gRPCAuditEmitter struct {
	// gRPC client へのコネクション。Close() で解放する。
	conn *grpc.ClientConn
	// AuditService client（生成 stub）。
	client auditv1.AuditServiceClient
	// 非同期送信キュー。channel 容量を超えた emit は drop。
	queue chan AuditEvent
	// worker goroutine 完了通知。
	done chan struct{}
	// shutdown flag（Close 後の Emit を no-op にする）。
	closed sync.Mutex
	isOpen bool
	// 送信タイムアウト（per-event）。
	sendTimeout time.Duration
}

// GRPCAuditEmitterConfig は構築時の任意パラメータ。
type GRPCAuditEmitterConfig struct {
	// Endpoint は t1-audit Pod の gRPC URL（"host:port"）。
	Endpoint string
	// QueueSize は非同期キュー容量。0 / 負値で既定 1024。
	QueueSize int
	// SendTimeout は per-event の gRPC 送信タイムアウト。0 で既定 2s。
	SendTimeout time.Duration
	// DialTimeout は接続確立タイムアウト。0 で既定 5s。
	DialTimeout time.Duration
	// DialOptions は追加 gRPC dial option（mTLS 認証情報など）。
	// 未指定時は insecure（dev / mTLS sidecar 経由を前提）。
	DialOptions []grpc.DialOption
}

// NewGRPCAuditEmitter は t1-audit Pod に接続する emitter を生成する。
// Close() で goroutine と connection を解放するため、defer で必ず呼ぶこと。
func NewGRPCAuditEmitter(ctx context.Context, cfg GRPCAuditEmitterConfig) (AuditEmitter, error) {
	if cfg.QueueSize <= 0 {
		cfg.QueueSize = 1024
	}
	if cfg.SendTimeout <= 0 {
		cfg.SendTimeout = 2 * time.Second
	}
	if cfg.DialTimeout <= 0 {
		cfg.DialTimeout = 5 * time.Second
	}
	dialCtx, cancel := context.WithTimeout(ctx, cfg.DialTimeout)
	defer cancel()
	opts := cfg.DialOptions
	if len(opts) == 0 {
		// dev / mesh-mTLS 経由を前提に、明示的な認証は呼出側設定に委ねる。
		opts = []grpc.DialOption{grpc.WithTransportCredentials(insecure.NewCredentials())}
	}
	conn, err := grpc.DialContext(dialCtx, cfg.Endpoint, opts...)
	if err != nil {
		return nil, err
	}
	e := &gRPCAuditEmitter{
		conn:        conn,
		client:      auditv1.NewAuditServiceClient(conn),
		queue:       make(chan AuditEvent, cfg.QueueSize),
		done:        make(chan struct{}),
		isOpen:      true,
		sendTimeout: cfg.SendTimeout,
	}
	go e.worker()
	return e, nil
}

// Emit は event を非同期キューに投入する。キュー満杯時は drop + warn ログ。
func (e *gRPCAuditEmitter) Emit(_ context.Context, ev AuditEvent) {
	e.closed.Lock()
	open := e.isOpen
	e.closed.Unlock()
	if !open {
		// Close 後の Emit は no-op。
		return
	}
	select {
	case e.queue <- ev:
		// 投入成功。
	default:
		// キュー満杯 → drop。fail-soft 方針（handler 結果に影響させない）。
		log.Printf("audit emitter: queue full, dropping event tenant=%q action=%q", ev.TenantID, ev.Action)
	}
}

// Close は queue を closing し、worker の終了を待つ。flushTimeout 内で送り切れない event は drop。
func (e *gRPCAuditEmitter) Close(flushTimeout time.Duration) error {
	e.closed.Lock()
	if !e.isOpen {
		e.closed.Unlock()
		return nil
	}
	e.isOpen = false
	close(e.queue)
	e.closed.Unlock()
	if flushTimeout <= 0 {
		flushTimeout = 5 * time.Second
	}
	select {
	case <-e.done:
		// worker 終了。
	case <-time.After(flushTimeout):
		// flush timeout: 残 event を drop して進む。
		log.Printf("audit emitter: flush timeout, some events may be dropped")
	}
	return e.conn.Close()
}

// worker は queue から event を取り出して AuditService.Record を逐次呼ぶ。
// 送信失敗（network / Pod 再起動）は warn ログに留め、event を drop する（fail-soft）。
func (e *gRPCAuditEmitter) worker() {
	defer close(e.done)
	for ev := range e.queue {
		e.send(ev)
	}
}

// send は単一 event を gRPC で送る。タイムアウトは sendTimeout。
func (e *gRPCAuditEmitter) send(ev AuditEvent) {
	ctx, cancel := context.WithTimeout(context.Background(), e.sendTimeout)
	defer cancel()
	// outcome を proto の SUCCESS / DENIED / ERROR に翻訳する（docs §「監査と痕跡」結果列挙）。
	outcome := "SUCCESS"
	switch ev.Result {
	case "denied":
		outcome = "DENIED"
	case "failure":
		outcome = "ERROR"
	}
	// attributes に trace_id / span_id / code を詰める（docs 必須フィールド）。
	attrs := make(map[string]string, 3)
	if ev.TraceID != "" {
		attrs["trace_id"] = ev.TraceID
	}
	if ev.SpanID != "" {
		attrs["span_id"] = ev.SpanID
	}
	if ev.Code != "" {
		attrs["code"] = ev.Code
	}
	req := &auditv1.RecordAuditRequest{
		Event: &auditv1.AuditEvent{
			Timestamp:  timestamppb.Now(),
			Actor:      ev.Actor,
			Action:     ev.Action,
			Resource:   ev.Resource,
			Outcome:    outcome,
			Attributes: attrs,
		},
		Context: &commonv1.TenantContext{
			TenantId: ev.TenantID,
			Subject:  ev.Actor,
		},
	}
	if _, err := e.client.Record(ctx, req); err != nil {
		// 失敗は warn ログのみ（fail-soft）。再送リトライ / DLQ 連携は次セッションで検討。
		log.Printf("audit emitter: gRPC Record failed: tenant=%q action=%q err=%v",
			ev.TenantID, ev.Action, err)
	}
}
