// 本ファイルは LogService / TelemetryService の in-process gRPC 結線テスト。
//
// 試験戦略:
//   bufconn で in-memory Listener を構築し、本番と同じ Register hook で gRPC server と
//   client を結ぶ。otel.NewStdoutBundle で構築した stdout JSON Lines emitter を bytes.Buffer
//   にバインドし、proto serialization → handler → emitter → stdout payload までを通す。
//   OTel SDK / Collector を介さないため CI 上で外部依存なしに実行できる。
//
// 本テストが PASS すれば「LogService.Send / BulkSend / TelemetryService.EmitMetric / EmitSpan
// が gRPC 経由で実値（JSON Line）を出力する」が単体テストではなく実 gRPC レイヤを通した
// 形で証明される（cmd/state/main.go の OTel 結線が動作している前提を整える）。

package state

import (
	// 出力バッファ。
	"bytes"
	// 全 RPC で context を伝搬する。
	"context"
	// JSON 出力のパース。
	"encoding/json"
	// bufconn の net.Conn 型。
	"net"
	// 改行で分割。
	"strings"
	// テストハーネス。
	"testing"

	// OTel emitter 構築。
	"github.com/k1s0/k1s0/src/tier1/go/internal/otel"
	// proto stub。
	logv1 "github.com/k1s0/sdk-go/proto/v1/k1s0/tier1/log/v1"
	telemetryv1 "github.com/k1s0/sdk-go/proto/v1/k1s0/tier1/telemetry/v1"
	// gRPC server / client。
	"google.golang.org/grpc"
	// 認証なし credential。
	"google.golang.org/grpc/credentials/insecure"
	// bufconn の Listener 型。
	"google.golang.org/grpc/test/bufconn"
)

// startStdoutObservabilityServer は cmd/state/main.go と同じ構成（stdout emitter 注入）で
// gRPC server を起動する。出力先は bytes.Buffer に差し替えて assertion 可能にする。
func startStdoutObservabilityServer(t *testing.T) (logv1.LogServiceClient, telemetryv1.TelemetryServiceClient, *bytes.Buffer, func()) {
	// テストヘルパであることをマーク。
	t.Helper()
	// 1 MiB バッファの bufconn を生成する。
	lis := bufconn.Listen(1024 * 1024)
	// 出力先 buffer を構築する（複数 emitter で共有する）。
	buf := &bytes.Buffer{}
	// stdout emitter を buffer 出力で構築する。
	bundle := otel.NewStdoutBundle(buf)
	// Deps に 3 emitter を注入する（Dapr adapter は不要、Log/Telemetry のみテスト対象）。
	deps := Deps{
		// Log emitter。
		LogEmitter: bundle.LogEmitter,
		// Metric emitter。
		MetricEmitter: bundle.MetricEmitter,
		// Trace emitter。
		TraceEmitter: bundle.TraceEmitter,
	}
	// gRPC server を生成する。
	srv := grpc.NewServer()
	// 本番の Register hook を使う（state.Register は 7 service 全件を登録するが、Dapr adapter
	// 未注入の State / PubSub / Binding / Invoke / Feature は呼ばないため副作用なし）。
	Register(deps)(srv)
	// 別 goroutine で listen ループを回す。
	go func() {
		// listen 失敗（bufconn 終了）は test 終了時の自然停止なので無視する。
		_ = srv.Serve(lis)
	}()
	// bufconn dialer を構築する。
	dialer := func(context.Context, string) (net.Conn, error) {
		// Conn を取得する。
		return lis.Dial()
	}
	// gRPC client を bufconn 越しに接続する。
	conn, err := grpc.NewClient(
		// passthrough scheme。
		"passthrough://bufnet",
		// dialer を注入する。
		grpc.WithContextDialer(dialer),
		// TLS なし。
		grpc.WithTransportCredentials(insecure.NewCredentials()),
	)
	// dial 失敗は即座に Fatal。
	if err != nil {
		// fatal で test を停止する。
		t.Fatalf("grpc.NewClient failed: %v", err)
	}
	// LogService と TelemetryService の typed client を生成する。
	lc := logv1.NewLogServiceClient(conn)
	// Telemetry client。
	tc := telemetryv1.NewTelemetryServiceClient(conn)
	// cleanup 関数。
	cleanup := func() {
		// client conn を閉じる。
		_ = conn.Close()
		// server を停止する。
		srv.Stop()
		// listener を閉じる。
		_ = lis.Close()
	}
	// 4 値を返却する。
	return lc, tc, buf, cleanup
}

// gRPC client から LogService.Send / BulkSend を呼び、stdout JSON Lines が
// 期待通り buffer に書き出されることを検証する。
func TestLogService_StdoutEmitter_OverGRPC(t *testing.T) {
	// bufconn server を起動する。
	lc, _, buf, cleanup := startStdoutObservabilityServer(t)
	// テスト終了時に cleanup する。
	defer cleanup()
	// Background context を使う。
	ctx := context.Background()

	// 1. Send: 1 件の log を送信する。
	if _, err := lc.Send(ctx, &logv1.SendLogRequest{
		// log entry。
		Entry: &logv1.LogEntry{
			// 重大度 ERROR。
			Severity: logv1.Severity_ERROR,
			// 本文。
			Body: "DB connection lost",
			// 属性。
			Attributes: map[string]string{"service.name": "tier1"},
		},
	}); err != nil {
		// fatal。
		t.Fatalf("Send failed: %v", err)
	}

	// 2. BulkSend: 3 件まとめて送信する。
	bulkResp, err := lc.BulkSend(ctx, &logv1.BulkSendLogRequest{
		// 3 entries。
		Entries: []*logv1.LogEntry{
			// 1 件目。
			{Body: "e1", Severity: logv1.Severity_INFO},
			// 2 件目。
			{Body: "e2", Severity: logv1.Severity_INFO},
			// 3 件目。
			{Body: "e3", Severity: logv1.Severity_INFO},
		},
	})
	// BulkSend 失敗は test 失敗。
	if err != nil {
		// fatal。
		t.Fatalf("BulkSend failed: %v", err)
	}
	// Accepted は 3 のはず。
	if bulkResp.GetAccepted() != 3 {
		// 不一致は test 失敗。
		t.Fatalf("Accepted mismatch: got %d", bulkResp.GetAccepted())
	}

	// 3. buffer の内容を JSON Lines として解析する（4 行のはず）。
	lines := splitNonEmptyLines(buf.String())
	// 4 行（Send 1 + BulkSend 3）が出ているはず。
	if len(lines) != 4 {
		// 不一致は test 失敗。
		t.Fatalf("expected 4 stdout lines, got %d: %s", len(lines), buf.String())
	}
	// 1 行目（Send）が ERROR severity で本文が "DB connection lost" であることを確認する。
	first := decodeStdoutLine(t, lines[0])
	// kind 確認。
	if first["kind"] != "log" {
		// 不一致は test 失敗。
		t.Fatalf("first line kind mismatch: %v", first["kind"])
	}
	// severity 確認。
	if first["severity"] != "ERROR" {
		// 不一致は test 失敗。
		t.Fatalf("first line severity mismatch: %v", first["severity"])
	}
	// body 確認。
	if first["body"] != "DB connection lost" {
		// 不一致は test 失敗。
		t.Fatalf("first line body mismatch: %v", first["body"])
	}
}

// gRPC client から TelemetryService.EmitMetric / EmitSpan を呼び、stdout JSON Lines
// に期待通りのスキーマで出力されることを検証する。
func TestTelemetryService_StdoutEmitter_OverGRPC(t *testing.T) {
	// bufconn server を起動する。
	_, tc, buf, cleanup := startStdoutObservabilityServer(t)
	// テスト終了時に cleanup する。
	defer cleanup()
	// Background context を使う。
	ctx := context.Background()

	// 1. EmitMetric: counter / gauge / histogram を一括送信する。
	if _, err := tc.EmitMetric(ctx, &telemetryv1.EmitMetricRequest{
		// 3 メトリクス。
		Metrics: []*telemetryv1.Metric{
			// counter。
			{Name: "k1s0.invoke.total", Kind: telemetryv1.MetricKind_COUNTER, Value: 1, Labels: map[string]string{"status": "ok"}},
			// gauge。
			{Name: "k1s0.invoke.queue_depth", Kind: telemetryv1.MetricKind_GAUGE, Value: 42},
			// histogram。
			{Name: "k1s0.invoke.duration_ms", Kind: telemetryv1.MetricKind_HISTOGRAM, Value: 12.5},
		},
	}); err != nil {
		// fatal。
		t.Fatalf("EmitMetric failed: %v", err)
	}

	// 2. EmitSpan: 1 件の span を送信する。
	if _, err := tc.EmitSpan(ctx, &telemetryv1.EmitSpanRequest{
		// 1 span。
		Spans: []*telemetryv1.Span{
			// span。
			{
				// trace_id。
				TraceId: "0123456789abcdef0123456789abcdef",
				// span_id。
				SpanId: "0123456789abcdef",
				// span 名。
				Name: "GET /api/v1/foo",
			},
		},
	}); err != nil {
		// fatal。
		t.Fatalf("EmitSpan failed: %v", err)
	}

	// 3. buffer の内容を JSON Lines として解析する（4 行のはず）。
	lines := splitNonEmptyLines(buf.String())
	// 3 metric + 1 span = 4 行のはず。
	if len(lines) != 4 {
		// 不一致は test 失敗。
		t.Fatalf("expected 4 stdout lines, got %d: %s", len(lines), buf.String())
	}
	// 1 行目（counter）の kind / metric_kind / value を確認する。
	m0 := decodeStdoutLine(t, lines[0])
	// kind 確認。
	if m0["kind"] != "metric" {
		// 不一致は test 失敗。
		t.Fatalf("metric line 0 kind mismatch: %v", m0["kind"])
	}
	// metric_kind 確認。
	if m0["metric_kind"] != "counter" {
		// 不一致は test 失敗。
		t.Fatalf("metric line 0 metric_kind mismatch: %v", m0["metric_kind"])
	}
	// 4 行目は span。
	span := decodeStdoutLine(t, lines[3])
	// kind 確認。
	if span["kind"] != "span" {
		// 不一致は test 失敗。
		t.Fatalf("span line kind mismatch: %v", span["kind"])
	}
	// trace_id 確認。
	if span["trace_id"] != "0123456789abcdef0123456789abcdef" {
		// 不一致は test 失敗。
		t.Fatalf("span line trace_id mismatch: %v", span["trace_id"])
	}
}

// splitNonEmptyLines は string を改行で分割し、空行を除いた slice を返す。
func splitNonEmptyLines(s string) []string {
	// 改行で分割する。
	parts := strings.Split(s, "\n")
	// 空行を除いた slice を準備する。
	out := make([]string, 0, len(parts))
	// 1 件ずつ filter する。
	for _, p := range parts {
		// 空行は skip する。
		if p == "" {
			// 次の iteration に進む。
			continue
		}
		// 結果に追加する。
		out = append(out, p)
	}
	// 結果を返す。
	return out
}

// decodeStdoutLine は JSON 1 行を map にデコードする（test 用 helper）。
func decodeStdoutLine(t *testing.T, line string) map[string]interface{} {
	// テストヘルパであることをマーク。
	t.Helper()
	// 結果格納先 map。
	var m map[string]interface{}
	// JSON unmarshal を実行する。
	if err := json.Unmarshal([]byte(line), &m); err != nil {
		// 失敗は test 停止。
		t.Fatalf("decode failed: %v\nline: %s", err, line)
	}
	// 結果を返す。
	return m
}
