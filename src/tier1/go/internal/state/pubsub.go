// 本ファイルは t1-state Pod の PubSubService 3 RPC ハンドラ実装。
//
// 設計正典:
//   docs/03_要件定義/20_機能要件/40_tier1_API契約IDL/03_PubSub_API.md
//
// scope（リリース時点 placeholder）: 実 Dapr Pub/Sub（Kafka）結線は plan 04-05。

package state

// 標準 / 内部パッケージ。
import (
	// context 伝搬。
	"context"
	// BulkPublish entry 失敗時の error_code 整形用。
	"fmt"
	// topic 形式検証用の事前コンパイル正規表現。
	"regexp"
	// FR-T1-PUBSUB-001 「published_at」自動付与で RFC 3339 タイムスタンプを生成する。
	"time"

	// Dapr adapter。
	"github.com/k1s0/k1s0/src/tier1/go/internal/adapter/dapr"
	// 共通 idempotency cache（共通規約 §「冪等性と再試行」）。
	"github.com/k1s0/k1s0/src/tier1/go/internal/common"
	// FR-T1-PUBSUB-002 「event_id は tier1 が UUID v7 で自動生成」。
	"github.com/google/uuid"
	// SDK 生成 stub の PubSubService 型。
	pubsubv1 "github.com/k1s0/sdk-go/proto/v1/k1s0/tier1/pubsub/v1"
	// gRPC エラーコード。
	"google.golang.org/grpc/codes"
	// FR-T1-PUBSUB-001 「trace_id 自動付与」用に incoming gRPC metadata を読む。
	"google.golang.org/grpc/metadata"
	// gRPC ステータスエラー。
	"google.golang.org/grpc/status"
)

// pubsubTopicRegex は PubSub backend が共通で許容する topic 名の正規表現。
// docs §「PubSub テナント prefix の物理表現」と整合: Kafka / GCP Pub/Sub /
// NATS / Redis Streams のいずれも `[a-zA-Z0-9._-]+` のみ。tier1 はテナント
// prefix を「ドット区切り」で付与するため、本 regex は prefix 付与「前」の
// 論理 topic 名と prefix 付与「後」の物理 topic 名 双方に適用できる。
var pubsubTopicRegex = regexp.MustCompile(`^[a-zA-Z0-9._-]+$`)

// pubsubMaxEventBytes は FR-T1-PUBSUB-005 受け入れ基準「イベントサイズ上限 1MB
// （Kafka メッセージ上限に合わせる）」。Strimzi Kafka の既定 message.max.bytes は
// 1 MiB（1_048_588 バイト）相当のため、handler 段でこの値で弾いて Kafka 側で
// rejected を発生させない。
const pubsubMaxEventBytes = 1 * 1024 * 1024

// pubsubMetaKeyEventID / TraceID / PublishedAt / TenantID は FR-T1-PUBSUB-001 で
// tier1 が自動付与する metadata キー名。subscriber 側 SDK は同名キーを取り出す。
const (
	pubsubMetaKeyEventID     = "event_id"
	pubsubMetaKeyTraceID     = "trace_id"
	pubsubMetaKeyPublishedAt = "published_at"
	pubsubMetaKeyTenantID    = "tenant_id"
)

// enrichPubSubMetadata は handler 段で event_id / trace_id / published_at / tenant_id を
// 自動付与する（FR-T1-PUBSUB-001 / FR-T1-PUBSUB-002）。
//
// 動作:
//   - 元 metadata を破壊せず、不在キーのみ補完する（呼出側が明示指定した場合は尊重）
//   - event_id: UUID v7（時系列ソート可能、共通規約 §「冪等性と再試行」と整合）
//   - trace_id: incoming gRPC metadata の traceparent から W3C Trace Context の
//     trace-id 部（最初の "-" 区切り後 32 文字）を抽出
//   - published_at: 現在時刻の RFC 3339 表現
//   - tenant_id: handler が requireTenantIDFromCtx で確定済みの tid を入れる
func enrichPubSubMetadata(ctx context.Context, original map[string]string, tenantID string) map[string]string {
	// nil-safe な copy を作る。
	out := make(map[string]string, len(original)+4)
	for k, v := range original {
		out[k] = v
	}
	// event_id: UUID v7 を新規生成（既存値があれば尊重）。
	if _, ok := out[pubsubMetaKeyEventID]; !ok {
		// NewV7 が rand 失敗する可能性は実用上ゼロだが、フォールバックは plain UUID v4 にする。
		if id, err := uuid.NewV7(); err == nil {
			out[pubsubMetaKeyEventID] = id.String()
		} else {
			out[pubsubMetaKeyEventID] = uuid.NewString()
		}
	}
	// trace_id: incoming metadata の traceparent から抽出する（W3C Trace Context）。
	if _, ok := out[pubsubMetaKeyTraceID]; !ok {
		if md, mok := metadata.FromIncomingContext(ctx); mok {
			if vs := md.Get("traceparent"); len(vs) > 0 {
				if tid := extractTraceIDFromTraceparent(vs[0]); tid != "" {
					out[pubsubMetaKeyTraceID] = tid
				}
			}
		}
	}
	// published_at: 現在時刻 RFC 3339（既存値があれば尊重しない、tier1 発行時刻を厳密に記録）。
	out[pubsubMetaKeyPublishedAt] = time.Now().UTC().Format(time.RFC3339Nano)
	// tenant_id: 必ず tier1 確定値を入れる（呼出側自己宣言は拒否、共通規約 L1）。
	out[pubsubMetaKeyTenantID] = tenantID
	return out
}

// extractTraceIDFromTraceparent は W3C Trace Context の traceparent から
// trace-id 部（32 文字 hex）を取り出す。形式: "00-<trace-id>-<parent-id>-<flags>"。
// 不正形式の場合は空文字を返す。
func extractTraceIDFromTraceparent(traceparent string) string {
	// "-" で 4 セグメントに分割する。
	parts := splitN(traceparent, '-', 4)
	if len(parts) < 4 {
		return ""
	}
	// 2 番目（index 1）が trace-id。32 文字 hex 必須。
	tid := parts[1]
	if len(tid) != 32 {
		return ""
	}
	return tid
}

// splitN は strings.Split と同等だが strings 依存を避けるための軽量版。
func splitN(s string, sep byte, n int) []string {
	out := make([]string, 0, n)
	start := 0
	for i := 0; i < len(s) && len(out) < n-1; i++ {
		if s[i] == sep {
			out = append(out, s[start:i])
			start = i + 1
		}
	}
	out = append(out, s[start:])
	return out
}

// validatePubSubTopic は handler 段で topic 名を事前検証する。
// 不正値は backend 越しに 500 を返さず、InvalidArgument（HTTP 400）として弾く。
func validatePubSubTopic(topic string) error {
	// 空 topic は単独で扱う（より具体的なメッセージを返すため）。
	if topic == "" {
		return status.Error(codes.InvalidArgument, "tier1/pubsub: topic required")
	}
	// 形式検証: Kafka 互換 regex。
	if !pubsubTopicRegex.MatchString(topic) {
		return status.Error(codes.InvalidArgument,
			"tier1/pubsub: invalid topic name (must match [a-zA-Z0-9._-]+)")
	}
	return nil
}

// pubsubHandler は PubSubService の handler 実装。
type pubsubHandler struct {
	// 将来 RPC 用埋め込み。
	pubsubv1.UnimplementedPubSubServiceServer
	// adapter 集合。
	deps Deps
	// 冪等性 cache（共通規約 §「冪等性と再試行」: 同一 idempotency_key の再試行で
	// 副作用を発生させず初回と同じレスポンスを返す）。nil の場合は dedup なし。
	idempotency common.IdempotencyCache
}

// Publish は単発 Publish。
// 共通規約 §「冪等性と再試行」: idempotency_key 指定時は同一キーの再試行で副作用を
// 重複させず、初回と同じレスポンスを返す（24h TTL の cache でレスポンスを保持）。
func (h *pubsubHandler) Publish(ctx context.Context, req *pubsubv1.PublishRequest) (*pubsubv1.PublishResponse, error) {
	// 入力 nil 防御。
	if req == nil {
		// 不正引数返却。
		return nil, status.Error(codes.InvalidArgument, "tier1/pubsub: nil request")
	}
	// NFR-E-AC-003: tenant_id 越境防止のため必須検証。
	tid, err := requireTenantIDFromCtx(ctx, req.GetContext(), "PubSub.Publish")
	if err != nil {
		return nil, err
	}
	// topic 名は Kafka / GCP Pub/Sub / NATS / Redis Streams 等で
	// `[a-zA-Z0-9._-]+` のみ許可。非 ASCII / 制御文字を含む topic は backend が
	// "invalid topic" として 500 を返してしまうため、handler で InvalidArgument に
	// 変換する（docs §「PubSub テナント prefix の物理表現」と整合）。
	if err := validatePubSubTopic(req.GetTopic()); err != nil {
		return nil, err
	}
	// FR-T1-PUBSUB-005: イベントサイズ上限 1MB を handler で強制する。
	// Kafka 側が rejected を返すのを待たず、ResourceExhausted（HTTP 429）として弾く。
	if len(req.GetData()) > pubsubMaxEventBytes {
		return nil, status.Errorf(codes.ResourceExhausted,
			"tier1/pubsub: data size %d exceeds maximum %d (1 MiB)", len(req.GetData()), pubsubMaxEventBytes)
	}
	// FR-T1-PUBSUB-001 / 002: event_id / trace_id / published_at / tenant_id を metadata に自動付与する。
	enriched := enrichPubSubMetadata(ctx, req.GetMetadata(), tid)
	// 実 Publish 実行クロージャ。idempotency cache hit 時は呼ばれない。
	doPublish := func() (interface{}, error) {
		areq := dapr.PublishRequest{
			Component:      "pubsub-kafka",
			Topic:          req.GetTopic(),
			Data:           req.GetData(),
			ContentType:    req.GetContentType(),
			IdempotencyKey: req.GetIdempotencyKey(),
			Metadata:       enriched,
			TenantID:       tid,
		}
		aresp, err := h.deps.PubSubAdapter.Publish(ctx, areq)
		if err != nil {
			return nil, translatePubSubErr(err, "Publish")
		}
		return &pubsubv1.PublishResponse{Offset: aresp.Offset}, nil
	}
	// 冪等性キー + cache が両方ある場合のみ dedup を適用する。
	// 空キーや cache 未注入時は通常呼出（後方互換 / dev 経路）。
	idempKey := common.IdempotencyKey(tid, "PubSub.Publish", req.GetIdempotencyKey())
	if idempKey == "" || h.idempotency == nil {
		resp, err := doPublish()
		if err != nil {
			return nil, err
		}
		return resp.(*pubsubv1.PublishResponse), nil
	}
	resp, err := h.idempotency.GetOrCompute(ctx, idempKey, doPublish)
	if err != nil {
		return nil, err
	}
	return resp.(*pubsubv1.PublishResponse), nil
}

// BulkPublish は複数 message を順次 Publish する。
// Dapr SDK は単一 PublishEvent しか提供しないため、ループで逐次発行する。
//
// docs §「PubSub API」: 「配列内の各エントリに個別の結果を返す（部分成功あり）」
// → 各 entry は独立した結果（成功 = offset、失敗 = error_code + メッセージ）として
// `BulkPublishEntry` に詰めて返す。1 件失敗で全体停止せず、後続 entry も処理する。
// tenant_id 不在のように batch 全体に響く前提違反は最初に弾き、entry 個別の不正
// （topic 形式 / adapter 側エラー等）は entry 結果に格納して継続する。
func (h *pubsubHandler) BulkPublish(ctx context.Context, req *pubsubv1.BulkPublishRequest) (*pubsubv1.BulkPublishResponse, error) {
	if req == nil {
		return nil, status.Error(codes.InvalidArgument, "tier1/pubsub: nil request")
	}
	results := make([]*pubsubv1.BulkPublishEntry, 0, len(req.GetEntries()))
	for i, entry := range req.GetEntries() {
		// 各 entry は PublishRequest（topic / data / content_type / idempotency_key /
		// metadata / context）。BulkPublishRequest 側で topic を共通化しているため
		// entry.topic と齟齬がある場合は entry を優先する。
		topic := entry.GetTopic()
		if topic == "" {
			topic = req.GetTopic()
		}
		// NFR-E-AC-003: 各 entry も tenant_id 越境防止のため必須検証。tenant 越境は
		// 「batch 全体の前提違反」のため、entry 結果ではなく即時 InvalidArgument で
		// 弾く（部分成功で抜けると security audit が破綻するため）。
		entTid, err := requireTenantIDFromCtx(ctx, entry.GetContext(), "PubSub.BulkPublish")
		if err != nil {
			return nil, err
		}
		// topic 形式の事前検証（Kafka 規約 [a-zA-Z0-9._-]+）。entry レベルの不正なので
		// entry 結果に格納して次へ進む。
		if verr := validatePubSubTopic(topic); verr != nil {
			results = append(results, &pubsubv1.BulkPublishEntry{
				EntryIndex: int32(i),
				ErrorCode:  "InvalidArgument: invalid topic name",
			})
			continue
		}
		// FR-T1-PUBSUB-005: 1 entry のサイズ上限を 1 MiB で強制する。entry レベルの
		// 上限超過は entry 結果に格納し、後続 entry の処理は継続する（部分成功）。
		if len(entry.GetData()) > pubsubMaxEventBytes {
			results = append(results, &pubsubv1.BulkPublishEntry{
				EntryIndex: int32(i),
				ErrorCode: fmt.Sprintf("ResourceExhausted: data size %d exceeds maximum %d (1 MiB)",
					len(entry.GetData()), pubsubMaxEventBytes),
			})
			continue
		}
		// FR-T1-PUBSUB-001 / 002: 各 entry にも event_id / trace_id / published_at /
		// tenant_id を自動付与する（entry ごとに event_id は別 UUID v7）。
		entMeta := enrichPubSubMetadata(ctx, entry.GetMetadata(), entTid)
		areq := dapr.PublishRequest{
			Component:      "pubsub-kafka",
			Topic:          topic,
			Data:           entry.GetData(),
			ContentType:    entry.GetContentType(),
			IdempotencyKey: entry.GetIdempotencyKey(),
			Metadata:       entMeta,
			TenantID:       entTid,
		}
		aresp, perr := h.deps.PubSubAdapter.Publish(ctx, areq)
		if perr != nil {
			// adapter 側 error も entry 個別の失敗として扱う（部分成功）。原 gRPC
			// code が取れる場合はそれを使い、無い場合は Internal 相当。
			code := "Internal"
			if st, ok := status.FromError(perr); ok && st.Code() != codes.Unknown {
				code = st.Code().String()
			}
			results = append(results, &pubsubv1.BulkPublishEntry{
				EntryIndex: int32(i),
				ErrorCode:  fmt.Sprintf("%s: %s", code, perr.Error()),
			})
			continue
		}
		results = append(results, &pubsubv1.BulkPublishEntry{
			EntryIndex: int32(i),
			Offset:     aresp.Offset,
		})
	}
	return &pubsubv1.BulkPublishResponse{Results: results}, nil
}

// Subscribe は server-streaming RPC。Dapr Subscribe で得た subscription から
// 逐次イベントを受信し、proto Event として gRPC stream クライアントへ転送する。
//
// stream context が cancel されると subscription を Close し関数を戻す。
// 送信成功後に ev.Ack()、stream.Send 失敗時は ev.Retry() を呼んで Dapr 側で再配信させる。
func (h *pubsubHandler) Subscribe(req *pubsubv1.SubscribeRequest, stream pubsubv1.PubSubService_SubscribeServer) error {
	if req == nil {
		return status.Error(codes.InvalidArgument, "tier1/pubsub: nil request")
	}
	// NFR-E-AC-003: tenant_id 越境防止のため必須検証。
	tid, terr := requireTenantIDFromCtx(stream.Context(), req.GetContext(), "PubSub.Subscribe")
	if terr != nil {
		return terr
	}
	ctx := stream.Context()
	sub, err := h.deps.PubSubAdapter.Subscribe(ctx, dapr.SubscribeAdapterRequest{
		Component:     "pubsub-kafka",
		Topic:         req.GetTopic(),
		ConsumerGroup: req.GetConsumerGroup(),
		TenantID:      tid,
	})
	if err != nil {
		return status.Errorf(codes.Internal, "tier1/pubsub: Subscribe failed: %v", err)
	}
	defer func() { _ = sub.Close() }()

	for {
		// stream cancel チェック（Receive が block する前に context 状況を確認）。
		if err := ctx.Err(); err != nil {
			return err
		}
		ev, err := sub.Receive(ctx)
		if err != nil {
			// adapter 側で「subscription closed」を通常終了として返す場合は io.EOF など。
			// 単純化のため、エラーは Internal として返却。
			return status.Errorf(codes.Internal, "tier1/pubsub: Subscribe receive: %v", err)
		}
		if ev == nil {
			// イベント無しは無視して次へ。
			continue
		}
		out := &pubsubv1.Event{
			Topic:       ev.Topic,
			Data:        ev.Data,
			ContentType: ev.ContentType,
			Offset:      ev.Offset,
			Metadata:    ev.Metadata,
		}
		if err := stream.Send(out); err != nil {
			// クライアントへの転送失敗 → Dapr 側で再配信させる。
			if ev.Retry != nil {
				_ = ev.Retry()
			}
			return status.Errorf(codes.Internal, "tier1/pubsub: stream.Send: %v", err)
		}
		// 転送成功 → ack。
		if ev.Ack != nil {
			_ = ev.Ack()
		}
	}
}

// translatePubSubErr は PubSub 用エラー翻訳。
func translatePubSubErr(err error, rpc string) error {
	// dapr / Kafka adapter が返す gRPC status code を尊重する（FailedPrecondition →
	// 409 / InvalidArgument → 400 / Unavailable → 503 等を HTTP layer に正しく伝える）。
	if st, ok := status.FromError(err); ok && st.Code() != codes.Unknown && st.Code() != codes.OK {
		return status.Errorf(st.Code(), "tier1/pubsub: %s adapter error: %s", rpc, st.Message())
	}
	// その他 → Internal。
	return status.Errorf(codes.Internal, "tier1/pubsub: %s adapter error: %v", rpc, err)
}
