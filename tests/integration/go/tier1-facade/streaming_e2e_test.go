// 本ファイルは tier1 の server-streaming RPC を実バイナリで検証する E2E。
//
// 設計正典:
//   docs/03_要件定義/20_機能要件/40_tier1_API契約IDL/01_Service_Invoke_API.md（InvokeStream）
//   docs/03_要件定義/20_機能要件/40_tier1_API契約IDL/03_PubSub_API.md（Subscribe）
//   docs/03_要件定義/20_機能要件/40_tier1_API契約IDL/10_Audit_Pii_API.md（Export）
//
// 検証目的:
//   HTTP/JSON gateway は server-streaming 非対応のため、binary を gRPC で叩いて
//   stream 経路が proto 契約通り動くことを実機で保証する。

package tier1facade

import (
	"context"
	"fmt"
	"io"
	"net"
	"os"
	"os/exec"
	"path/filepath"
	"strings"
	"testing"
	"time"

	auditv1 "github.com/k1s0/sdk-go/proto/v1/k1s0/tier1/audit/v1"
	commonv1 "github.com/k1s0/sdk-go/proto/v1/k1s0/tier1/common/v1"
	pubsubv1 "github.com/k1s0/sdk-go/proto/v1/k1s0/tier1/pubsub/v1"
	serviceinvokev1 "github.com/k1s0/sdk-go/proto/v1/k1s0/tier1/serviceinvoke/v1"
	"google.golang.org/grpc"
	"google.golang.org/grpc/credentials/insecure"
)

// startStatePodGRPC は cmd/state を起動し、gRPC client conn を返す。
func startStatePodGRPC(t *testing.T) (*grpc.ClientConn, func()) {
	t.Helper()
	root := repoRoot(t)
	out := filepath.Join(t.TempDir(), "k1s0-state")
	build := exec.Command("go", "build", "-o", out, "./cmd/state")
	build.Dir = filepath.Join(root, "src/tier1/go")
	build.Env = append(os.Environ(), "CGO_ENABLED=0")
	if outBytes, err := build.CombinedOutput(); err != nil {
		t.Fatalf("go build cmd/state: %v\n%s", err, outBytes)
	}
	grpcPort := findFreePort(t)
	cmd := exec.Command(out,
		"-listen", fmt.Sprintf(":%d", grpcPort),
		"-http-listen", "off",
	)
	cmd.Stdout = io.Discard
	cmd.Stderr = io.Discard
	if err := cmd.Start(); err != nil {
		t.Fatalf("start state binary: %v", err)
	}
	dialAddr := fmt.Sprintf("127.0.0.1:%d", grpcPort)

	deadline := time.Now().Add(5 * time.Second)
	for time.Now().Before(deadline) {
		c, err := net.DialTimeout("tcp", dialAddr, 200*time.Millisecond)
		if err == nil {
			_ = c.Close()
			break
		}
		time.Sleep(50 * time.Millisecond)
	}
	conn, err := grpc.NewClient(dialAddr, grpc.WithTransportCredentials(insecure.NewCredentials()))
	if err != nil {
		t.Fatalf("dial: %v", err)
	}
	cleanup := func() {
		_ = conn.Close()
		_ = cmd.Process.Signal(os.Interrupt)
		done := make(chan error, 1)
		go func() { done <- cmd.Wait() }()
		select {
		case <-done:
		case <-time.After(3 * time.Second):
			_ = cmd.Process.Kill()
			<-done
		}
	}
	return conn, cleanup
}

// startRustPodGRPC は Rust pod を起動し、gRPC client conn を返す（HTTP gateway は off）。
func startRustPodGRPC(t *testing.T, pod string) (*grpc.ClientConn, func()) {
	t.Helper()
	bin := rustBinaryPath(t, pod)
	grpcPort := findFreePort(t)
	cmd := exec.Command(bin)
	cmd.Env = append(os.Environ(),
		fmt.Sprintf("LISTEN_ADDR=[::]:%d", grpcPort),
		"TIER1_HTTP_LISTEN_ADDR=off",
		"TIER1_AUTH_MODE=off",
	)
	cmd.Stdout = io.Discard
	cmd.Stderr = io.Discard
	if err := cmd.Start(); err != nil {
		t.Fatalf("start %s: %v", pod, err)
	}
	dialAddr := fmt.Sprintf("127.0.0.1:%d", grpcPort)

	deadline := time.Now().Add(8 * time.Second)
	for time.Now().Before(deadline) {
		c, err := net.DialTimeout("tcp", dialAddr, 200*time.Millisecond)
		if err == nil {
			_ = c.Close()
			break
		}
		time.Sleep(80 * time.Millisecond)
	}
	conn, err := grpc.NewClient(dialAddr, grpc.WithTransportCredentials(insecure.NewCredentials()))
	if err != nil {
		t.Fatalf("dial: %v", err)
	}
	cleanup := func() {
		_ = conn.Close()
		_ = cmd.Process.Signal(os.Interrupt)
		done := make(chan error, 1)
		go func() { done <- cmd.Wait() }()
		select {
		case <-done:
		case <-time.After(3 * time.Second):
			_ = cmd.Process.Kill()
			<-done
		}
	}
	return conn, cleanup
}

// State Pod gRPC: ServiceInvoke.InvokeStream で in-memory echo backend のチャンク分割を確認。
func TestServiceInvokePod_GRPC_InvokeStream(t *testing.T) {
	if testing.Short() {
		t.Skip("skip binary integration test in -short mode")
	}
	conn, cleanup := startStatePodGRPC(t)
	defer cleanup()

	cli := serviceinvokev1.NewInvokeServiceClient(conn)
	ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
	defer cancel()

	// in-memory backend は echo response を返すため、Data がそのまま戻る。
	payload := []byte(strings.Repeat("X", 10*1024)) // 10 KiB → 4 KiB chunk で 3 chunk
	stream, err := cli.InvokeStream(ctx, &serviceinvokev1.InvokeRequest{
		AppId:       "echo-app",
		Method:      "ping",
		Data:        payload,
		ContentType: "application/octet-stream",
		Context:     &commonv1.TenantContext{TenantId: "T-stream"},
	})
	if err != nil {
		t.Fatalf("InvokeStream: %v", err)
	}
	totalBytes := 0
	chunkCount := 0
	eofReceived := false
	for {
		chunk, err := stream.Recv()
		if err == io.EOF {
			break
		}
		if err != nil {
			t.Fatalf("Recv: %v", err)
		}
		totalBytes += len(chunk.GetData())
		chunkCount++
		if chunk.GetEof() {
			eofReceived = true
			break
		}
	}
	if !eofReceived {
		t.Errorf("did not receive EOF chunk")
	}
	if totalBytes != len(payload) {
		t.Errorf("total bytes mismatch: got %d want %d", totalBytes, len(payload))
	}
	if chunkCount < 2 {
		t.Errorf("expected >=2 chunks for 10KiB payload, got %d", chunkCount)
	}
}

// State Pod gRPC: PubSub Subscribe stream が in-memory bus 経由で Publish を受信する。
func TestPubSubPod_GRPC_Subscribe(t *testing.T) {
	if testing.Short() {
		t.Skip("skip binary integration test in -short mode")
	}
	conn, cleanup := startStatePodGRPC(t)
	defer cleanup()

	cli := pubsubv1.NewPubSubServiceClient(conn)
	subCtx, subCancel := context.WithCancel(context.Background())
	defer subCancel()
	stream, err := cli.Subscribe(subCtx, &pubsubv1.SubscribeRequest{
		Topic:         "shipments",
		ConsumerGroup: "g-stream-test",
		Context:       &commonv1.TenantContext{TenantId: "T-stream-pub"},
	})
	if err != nil {
		t.Fatalf("Subscribe: %v", err)
	}

	// channel 確保のため少し待つ。
	time.Sleep(50 * time.Millisecond)

	// 別 goroutine で 2 件 publish。
	go func() {
		ctx, cancel := context.WithTimeout(context.Background(), 2*time.Second)
		defer cancel()
		for _, data := range [][]byte{[]byte("evt-1"), []byte("evt-2")} {
			_, err := cli.Publish(ctx, &pubsubv1.PublishRequest{
				Topic:       "shipments",
				Data:        data,
				ContentType: "application/octet-stream",
				Context:     &commonv1.TenantContext{TenantId: "T-stream-pub"},
			})
			if err != nil {
				t.Errorf("Publish: %v", err)
				return
			}
		}
	}()

	got := make([]string, 0, 2)
	for i := 0; i < 2; i++ {
		ev, err := stream.Recv()
		if err != nil {
			t.Fatalf("Recv (%d): %v", i, err)
		}
		got = append(got, string(ev.GetData()))
	}
	if got[0] != "evt-1" || got[1] != "evt-2" {
		t.Errorf("payload order: %v want [evt-1 evt-2]", got)
	}
}

// Audit Pod gRPC: Export streaming が Record 後の events を NDJSON / chunk として返す。
func TestAuditPod_GRPC_ExportStream(t *testing.T) {
	if testing.Short() {
		t.Skip("skip rust binary test in -short mode")
	}
	conn, cleanup := startRustPodGRPC(t, "audit")
	defer cleanup()

	cli := auditv1.NewAuditServiceClient(conn)
	ctx, cancel := context.WithTimeout(context.Background(), 8*time.Second)
	defer cancel()

	// 5 件 Record してから Export する。
	for i := 0; i < 5; i++ {
		_, err := cli.Record(ctx, &auditv1.RecordAuditRequest{
			Event: &auditv1.AuditEvent{
				Actor:    "stream-test",
				Action:   fmt.Sprintf("act.%d", i),
				Resource: fmt.Sprintf("res-%d", i),
				Outcome:  "SUCCESS",
			},
			Context: &commonv1.TenantContext{TenantId: "T-export"},
		})
		if err != nil {
			t.Fatalf("Record %d: %v", i, err)
		}
	}

	stream, err := cli.Export(ctx, &auditv1.ExportAuditRequest{
		Format:     auditv1.ExportFormat_EXPORT_FORMAT_NDJSON,
		ChunkBytes: 4096,
		Context:    &commonv1.TenantContext{TenantId: "T-export"},
	})
	if err != nil {
		t.Fatalf("Export: %v", err)
	}
	totalBytes := 0
	chunks := 0
	lastSeen := false
	for {
		chunk, err := stream.Recv()
		if err == io.EOF {
			break
		}
		if err != nil {
			t.Fatalf("Recv: %v", err)
		}
		totalBytes += len(chunk.GetData())
		chunks++
		if chunk.GetIsLast() {
			lastSeen = true
		}
		if chunks > 50 {
			t.Fatal("too many chunks, possibly infinite loop")
		}
	}
	if !lastSeen {
		t.Errorf("Export did not emit is_last=true marker")
	}
	if totalBytes == 0 {
		t.Errorf("Export streamed zero bytes for 5 events")
	}
}
