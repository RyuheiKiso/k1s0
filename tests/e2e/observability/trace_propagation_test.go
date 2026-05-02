// 本ファイルは観測性 E2E の検証 1（OTLP trace 貫通）。
// 設計正典: ADR-TEST-006（観測性 E2E を 5 検証で構造化）
// 関連 ADR: ADR-OBS-001（Grafana LGTM）/ ADR-OBS-002（OpenTelemetry Collector）
//
// 検証対象（リリース時点での最小成立形）:
//   1. OTLP gRPC exporter 経由で Tempo に span を送信できる
//   2. Tempo HTTP API（/api/traces/<trace-id>）で送信した span を取得できる
//   3. span が想定された service.name 属性を保持している
//
// tier1→2→3 を貫通する trace_id 連続性の検証は採用初期で本ファイルを拡張する。
// 現在は「自前 1 span を Tempo に送って取得できる」最小経路の動作確認に留める。
//
// 前提:
//   K1S0_TEMPO_OTLP_TARGET=localhost:4317  （OTel Collector の OTLP gRPC endpoint）
//   K1S0_TEMPO_HTTP_TARGET=http://localhost:3200  （Tempo の HTTP API endpoint）
//   いずれも tools/local-stack/up.sh --observability で起動する。
package observability

import (
	"context"
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"os"
	"testing"
	"time"

	// OTel SDK（trace 生成 / OTLP exporter / SpanProcessor）
	"go.opentelemetry.io/otel"
	"go.opentelemetry.io/otel/exporters/otlp/otlptrace/otlptracegrpc"
	"go.opentelemetry.io/otel/sdk/resource"
	sdktrace "go.opentelemetry.io/otel/sdk/trace"
	semconv "go.opentelemetry.io/otel/semconv/v1.26.0"
	"go.opentelemetry.io/otel/trace"

	// gRPC（OTLP exporter の transport）
	"google.golang.org/grpc"
	"google.golang.org/grpc/credentials/insecure"
)

// TestOTLPTracePropagation は最小の trace 貫通検証。
// OTLP gRPC exporter で span を送信 → Tempo HTTP API で取得 → span 数 ≥ 1 を assert。
func TestOTLPTracePropagation(t *testing.T) {
	// OTLP exporter の endpoint（OTel Collector または直接 Tempo）
	otlpTarget := os.Getenv("K1S0_TEMPO_OTLP_TARGET")
	if otlpTarget == "" {
		t.Skip("K1S0_TEMPO_OTLP_TARGET 未設定: tools/local-stack/up.sh --observability で起動した OTel Collector / Tempo の OTLP gRPC endpoint を指定（例: localhost:4317）")
	}
	// Tempo HTTP API の endpoint（trace 取得用）
	tempoHTTP := os.Getenv("K1S0_TEMPO_HTTP_TARGET")
	if tempoHTTP == "" {
		t.Skip("K1S0_TEMPO_HTTP_TARGET 未設定: Tempo の HTTP API endpoint（例: http://localhost:3200）を指定")
	}

	// 全体タイムアウト 60 秒（Tempo の取り込み遅延を見込む）
	ctx, cancel := context.WithTimeout(context.Background(), 60*time.Second)
	defer cancel()

	// OTLP gRPC exporter を構築（dev: TLS なし）
	exporter, err := otlptracegrpc.New(
		ctx,
		otlptracegrpc.WithEndpoint(otlpTarget),
		otlptracegrpc.WithDialOption(grpc.WithTransportCredentials(insecure.NewCredentials())),
	)
	if err != nil {
		t.Fatalf("otlptracegrpc.New(target=%s): %v", otlpTarget, err)
	}
	// test 終了時に exporter を確実にシャットダウン
	defer func() {
		shutdownCtx, shutdownCancel := context.WithTimeout(context.Background(), 5*time.Second)
		defer shutdownCancel()
		_ = exporter.Shutdown(shutdownCtx)
	}()

	// service.name 等を span に付ける Resource
	res, err := resource.New(
		ctx,
		resource.WithAttributes(
			// 採用検討者がこの span を「e2e test 起源」と識別できるよう固定値
			semconv.ServiceName("k1s0-e2e-trace-propagation"),
			semconv.ServiceVersion("e2e-test"),
		),
	)
	if err != nil {
		t.Fatalf("resource.New: %v", err)
	}

	// TracerProvider を構築（同期 BatchSpanProcessor、test では即時 flush）
	tp := sdktrace.NewTracerProvider(
		sdktrace.WithBatcher(exporter, sdktrace.WithBatchTimeout(1*time.Second)),
		sdktrace.WithResource(res),
		// 全 span を sampling（test では cardinality 心配無用）
		sdktrace.WithSampler(sdktrace.AlwaysSample()),
	)
	// test 終了時に TracerProvider を shutdown して残り span を flush
	defer func() {
		shutdownCtx, shutdownCancel := context.WithTimeout(context.Background(), 5*time.Second)
		defer shutdownCancel()
		_ = tp.Shutdown(shutdownCtx)
	}()
	// global TracerProvider に設定（OTel API 経由の span 生成で使われる）
	otel.SetTracerProvider(tp)

	// span を 1 つ生成して終了させる
	tracer := tp.Tracer("k1s0-e2e-trace-propagation")
	spanCtx, span := tracer.Start(ctx, "e2e.trace_propagation.smoke")
	// span 生成直後の trace_id を取得（同 test 末尾の Tempo Query で使う）
	spanContextValue := trace.SpanContextFromContext(spanCtx)
	if !spanContextValue.IsValid() {
		t.Fatalf("生成 span の SpanContext が無効: trace_id=%v span_id=%v", spanContextValue.TraceID(), spanContextValue.SpanID())
	}
	traceID := spanContextValue.TraceID().String()
	t.Logf("送信 trace_id=%s", traceID)
	// 短い処理時間を持たせて span を終了
	time.Sleep(50 * time.Millisecond)
	span.End()

	// 強制 flush（BatchSpanProcessor の queue を空にしてから Tempo に送り終わる）
	flushCtx, flushCancel := context.WithTimeout(ctx, 10*time.Second)
	defer flushCancel()
	if err := tp.ForceFlush(flushCtx); err != nil {
		t.Fatalf("ForceFlush: %v", err)
	}

	// Tempo の取り込み遅延（ingester → block）を考慮し最大 30 秒 polling で trace 取得を試みる
	// Tempo HTTP API: GET /api/traces/<trace-id> → 200 OK で trace JSON を返す
	traceURL := fmt.Sprintf("%s/api/traces/%s", tempoHTTP, traceID)
	deadline := time.Now().Add(30 * time.Second)
	var lastErr error
	for time.Now().Before(deadline) {
		// HTTP GET（短い timeout で polling）
		req, err := http.NewRequestWithContext(ctx, http.MethodGet, traceURL, nil)
		if err != nil {
			t.Fatalf("http.NewRequest: %v", err)
		}
		client := &http.Client{Timeout: 5 * time.Second}
		resp, err := client.Do(req)
		if err != nil {
			// network エラーは polling 継続、最後の error を log
			lastErr = err
			time.Sleep(2 * time.Second)
			continue
		}
		// 404 は trace 未到達（取り込み遅延）、polling 継続
		if resp.StatusCode == http.StatusNotFound {
			_ = resp.Body.Close()
			time.Sleep(2 * time.Second)
			continue
		}
		// 200 以外（500 等）は致命的エラー
		if resp.StatusCode != http.StatusOK {
			body, _ := io.ReadAll(resp.Body)
			_ = resp.Body.Close()
			t.Fatalf("Tempo Query 異常 status=%d body=%s", resp.StatusCode, string(body))
		}
		// 200 OK: trace 取得成功
		body, err := io.ReadAll(resp.Body)
		_ = resp.Body.Close()
		if err != nil {
			t.Fatalf("response body 読み取り失敗: %v", err)
		}
		// JSON 構造を簡易 parse（batches[].resource / batches[].scopeSpans[] の有無を確認）
		var got struct {
			Batches []map[string]any `json:"batches"`
		}
		if err := json.Unmarshal(body, &got); err != nil {
			t.Fatalf("Tempo response JSON parse 失敗: %v body=%s", err, string(body))
		}
		// batches が空なら span が記録されていない
		if len(got.Batches) == 0 {
			t.Fatalf("Tempo Query: batches 空（trace_id=%s が記録されていない）", traceID)
		}
		t.Logf("Tempo Query 成功: trace_id=%s batches=%d", traceID, len(got.Batches))
		return
	}
	// polling timeout（取り込みが追いつかない / ingester 経路に問題）
	t.Fatalf("Tempo Query timeout (30s): trace_id=%s last_err=%v", traceID, lastErr)
}
