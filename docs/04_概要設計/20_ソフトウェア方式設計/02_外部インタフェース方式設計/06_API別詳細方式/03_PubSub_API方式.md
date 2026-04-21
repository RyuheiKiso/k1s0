# 03. PubSub API 方式

本ファイルは IPA 共通フレーム 2013 の 7.1.2.2「外部及びコンポーネント間のインタフェース方式設計（外部側）」に対応し、tier1 11 API のうち非同期メッセージング（Publish / Subscribe）を担う PubSub API の詳細方式を定める。共通契約は [00_API共通規約方式.md](00_API共通規約方式.md)（DS-SW-EIF-200〜211）を参照し、本ファイルは PubSub 固有の Protobuf 定義・CloudEvents 準拠・トピック命名強制・Consumer Group・DLQ・Ordering・Schema Registry・SLO 内訳の 8 点を深掘りする。

## 本ファイルの位置付け

PubSub API は tier2 / tier3 が非同期イベント駆動でサービス間を疎結合するための唯一の正規経路である。親ファイル [01_tier1_11API方式概要.md](../01_tier1_11API方式概要.md) の DS-SW-EIF-001 で Apache Kafka（Strimzi、KRaft）をバックエンドに採用し、Go ファサード（`facade-pubsub` Pod）が Dapr PubSub コンポーネント経由でアクセスする構成を確定させた。業務アプリから見ると「トピック名と payload を指定するだけで、テナント分離・スキーマ検証・at-least-once 配信・DLQ・ordering がすべて付随する」抽象として機能する。

この抽象が崩れる代表シナリオは 3 つある。1 つ目はトピック名が自由すぎてテナント間の event 混入が発生するシナリオ、2 つ目は Consumer Group が共有されて他アプリのイベントを誤処理するシナリオ、3 つ目は消費側の連続失敗で無限ループに陥り消費しきれないラグが溜まるシナリオである。本ファイルは 3 つのリスクを tier1 側で強制的に閉じる仕組みを明示する。

PubSub は「at-least-once セマンティクス」を基底とする。exactly-once は Kafka Transaction + Consumer Transactional Read で理論上達成可能だが、Dapr の PubSub コンポーネントが at-least-once に限定されるため本方式もそれに追従する。exactly-once が必須な業務（Saga 補償、会計仕訳）は Workflow API で Idempotency Key + Saga パターンを組み合わせる設計とし、PubSub 側での重複受信は業務側で冪等に処理する責務分担を原則とする。

## Protobuf Service 定義

PubSub API は Publish と Subscribe の 2 操作を持つが、Subscribe はサーバサイドストリーミング（push 型）で設計する。従来の pull 型（poll）はクライアント実装が複雑化し、Dapr の Subscribe API とも不整合となる。

**設計項目 DS-SW-EIF-260 Protobuf Service 定義**

`protos/k1s0/public/pubsub/v1/pubsub.proto` に以下を定義する。

```protobuf
service PubSub {
  rpc Publish(PublishRequest) returns (PublishResponse);
  // サーバサイドストリーミング。クライアント接続中はサーバから event を push 配信
  rpc Subscribe(SubscribeRequest) returns (stream SubscribeResponse);
  // 明示的 ack（Subscribe 受信後の消費確定通知）
  rpc Ack(AckRequest) returns (AckResponse);
  // 明示的 nack（消費失敗通知、DLQ 送付トリガ）
  rpc Nack(NackRequest) returns (NackResponse);
}

message PublishRequest {
  // Dapr PubSub コンポーネント名（Phase 1a は "kafka-primary" 固定）
  string pubsub_name = 1;
  // トピック名（DS-SW-EIF-262 の命名規約を強制）
  string topic = 2;
  // payload
  bytes data = 3;
  // Content-Type（例 "application/cloudevents+json"）
  string data_content_type = 4;
  // 追加属性（CloudEvents extensions として同梱）
  map<string, string> metadata = 5;
  // 業務相関 ID（ordering key にも使用、DS-SW-EIF-265）
  string correlation_id = 6;
  // ordering key 明示指定（未指定時は correlation_id 使用）
  string partition_key = 7;
}

message SubscribeRequest {
  string pubsub_name = 1;
  string topic = 2;
  // Consumer Group 名（DS-SW-EIF-264 の命名規約を強制）
  string consumer_group = 3;
  // 並列取得数（1〜100、Kafka partition 数を超えて指定しても上限は partition 数）
  int32 max_in_flight = 4;
}

message SubscribeResponse {
  string event_id = 1;          // CloudEvents id
  string event_type = 2;        // CloudEvents type
  string event_source = 3;      // CloudEvents source
  bytes data = 4;
  string data_content_type = 5;
  google.protobuf.Timestamp event_time = 6;
  map<string, string> metadata = 7;
  // tier1 側で採番する ack 用 token（DS-SW-EIF-267）
  string delivery_token = 8;
}
```

Ack / Nack は `delivery_token` で個別 event を識別する。自動 ack（Subscribe ストリーム側で即時 ack）のクライアントモードも SDK 側でオプション提供するが、業務側で失敗時のロールバック処理を考慮するなら明示 Ack / Nack 運用を推奨する。

## CloudEvents v1.0 準拠

Event のペイロード形式を tier1 独自仕様にすると、外部 SaaS 連携（将来の Webhook 受信、他社イベントバス接続）で毎回変換層が必要になる。CloudEvents v1.0（CNCF 標準）に準拠することで相互運用性を確保する。

**設計項目 DS-SW-EIF-261 CloudEvents v1.0 準拠と必須属性**

PubSub の event は CloudEvents v1.0 の structured content mode（`application/cloudevents+json`）を第一選択とし、binary content mode（Kafka headers に属性配置）も Dapr 側で同等に扱う。必須属性とその生成責任は以下のとおり。

| 属性 | 生成責任 | 値例 | 根拠 |
|------|---------|------|------|
| `id` | クライアント SDK（UUID v4） | `550e8400-e29b-41d4-a716-446655440000` | CloudEvents 必須、重複検出にも使用 |
| `source` | クライアント SDK | `//k1s0/t0001/order-api` | app-id と対称、同一テナント内で一意 |
| `specversion` | SDK 固定 | `1.0` | CloudEvents v1.0 固定 |
| `type` | 業務 | `com.jtc.order.created.v1` | 逆 DNS + 業務イベント名 + major version |
| `time` | SDK 自動（UTC） | `2026-04-21T10:15:30Z` | RFC 3339、Publish 時刻 |
| `datacontenttype` | 業務 | `application/json` | 内容の MIME type |
| `subject` | 業務（任意） | `order/12345` | aggregate id の可読表現 |
| `k1s0tenantid`（ext） | tier1 自動付与 | `t0001` | DS-SW-EIF-004 と整合、JWT から抽出 |
| `k1s0traceparent`（ext） | tier1 自動付与 | W3C traceparent 形式 | 分散トレース連結 |

`k1s0tenantid` は tier1 が強制付与し、クライアントが自己申告した値があっても JWT 由来で上書きする。これにより悪意あるクライアントの tenant 詐称を Publish 時点で遮断する。

## トピック命名強制

トピック名が自由だとテナント間・ドメイン間で event が混在し、Consumer Group 設定ミスで他テナントの event を誤処理するリスクが生じる。tier1 側で命名規約を強制する。

**設計項目 DS-SW-EIF-262 トピック命名規約**

トピック名は `<tenant_id>.<domain>.<event_type>.v<major>` 形式に強制する。例: `t0001.order.created.v1`、`t0002.inventory.stock_adjusted.v2`。構成要素と根拠は以下のとおり。

| セグメント | 形式 | 根拠 |
|-----------|------|------|
| `tenant_id` | `t` + 4 桁数字 | テナント境界の物理分離 |
| `domain` | 小文字英数字 3〜15 文字 | ビジネスドメイン（order / billing / inventory 等） |
| `event_type` | 小文字英数字アンダースコア 3〜40 文字 | 業務イベント名（created / updated / cancelled 等） |
| `v<major>` | `v` + 数字 | スキーマ破壊的変更時に v2 を並列運用 |

規約違反のトピック名で Publish した場合、tier1 は `PUBSUB_TOPIC_INVALID_NAME` を返して処理を拒否する。Subscribe 側も同様で、自テナント `tenant_id` と一致しないトピックへの Subscribe は `AUTH_TENANT_MISMATCH` を返す。この二重強制により、クライアント側のバグや設定ミスがあっても物理的にテナント間 event 漏洩が発生しない。

トピックの事前作成は `facade-pubsub` が Dapr PubSub コンポーネント経由で Kafka API を呼んで自動作成する。partition 数の初期値は 12（業界ベンチマーク 10〜20 の中央値）、replication factor は 3（Phase 1b で Strimzi 3 broker 前提）。partition 数は後から増やせるが減らせないため、初期値は慎重に設定する。

## Consumer Group 分離

複数の Subscribe クライアントが同一 Consumer Group を使うと、Kafka は各 event を 1 つのコンシューマにのみ配信する（負荷分散）。これは意図した挙動だが、別アプリが同じトピックを独立消費したい場合は Consumer Group を分離する必要がある。命名規約を強制して衝突を防ぐ。

**設計項目 DS-SW-EIF-263 Consumer Group 命名規約**

Consumer Group 名は `<app_id>.<tenant_id>` 形式に強制する。例: `order-worker.t0001`、`billing-worker.t0002`。app_id は JWT claim の `app_id`（Keycloak client_id）から tier1 が自動取得し、クライアント指定値は無視する。これにより「同一アプリ内で同一 Consumer Group」が自動成立し、アプリ間で Consumer Group が衝突しない。

同一アプリ内で複数 Consumer Group が必要なケース（例: 読み込み専用ダッシュボードとワーカの分離）は、app_id のサブ識別子として `<app_id>-<suffix>.<tenant_id>` の拡張命名を許容する。suffix は `[a-z0-9\-]{1,20}` の範囲で、Keycloak 側で事前登録された suffix のみ許容する（任意 suffix を許すと結局衝突が起きる）。

## at-least-once と DLQ

Kafka は at-least-once が自然だが、ack 忘れによるメッセージ滞留や連続失敗による無限ループが発生する。DLQ（Dead Letter Queue）で失敗トリアージを自動化する。

**設計項目 DS-SW-EIF-264 DLQ 方式**

Subscribe で受信した event に対し、クライアントから 30 秒以内に Ack / Nack が来なければ `facade-pubsub` は自動 Nack として扱い、該当 event を再配信キューに戻す。同一 event が 3 回 Nack された場合（delivery_token 単位でカウント）、`<topic>.dlq` トピック（例 `t0001.order.created.v1.dlq`）に転送する。元 event の CloudEvents 属性はすべて保持し、追加で以下の属性を付与する。

| 属性 | 内容 |
|------|------|
| `k1s0dlqreason` | 最終 Nack 時のエラーメッセージ（クライアント申告） |
| `k1s0dlqattempts` | 失敗回数（常に 3） |
| `k1s0dlqoriginaltopic` | 元トピック名 |
| `k1s0dlqfirstfailedat` | 最初の Nack 時刻（RFC 3339） |
| `k1s0dlqlastfailedat` | 最後の Nack 時刻（RFC 3339） |

根拠: 3 回失敗の閾値は「一時的障害（通常 1〜2 回で自己回復）」と「恒久的 bug」を分離する経験値で、Service Invoke のサーキットブレーカ閾値 5 連続と比べて PubSub は保守的に 3 に設定する（PubSub は失敗時の業務影響が波及しやすいため）。DLQ トピックは手動調査用であり、自動再処理は行わない。SRE は DLQ 件数を Prometheus `k1s0_pubsub_dlq_events_total` で監視し、閾値超過で PagerDuty alert する。

## Ordering

一部業務では event の順序保証が必須（例: 注文 created → updated → cancelled の順）。Kafka は partition 内で順序保証されるため、同一 aggregate 単位で同一 partition に配置する必要がある。

**設計項目 DS-SW-EIF-265 Ordering とパーティションキー**

Publish 時の `partition_key` フィールドでパーティションキーを明示指定する。デフォルトは `correlation_id` を使用する。tier1 は内部的に `<tenant_id>:<partition_key>` の SHA-256 を Kafka partition 数（初期 12）で modulo して partition 選択する。tenant_id を前置する理由は、異なるテナントで同一 aggregate_id が衝突しても別 partition に配置するためである。

ordering が不要な業務（ログ・メトリクス類）は partition_key を未指定にすることで Kafka の round-robin 配信（ランダム partition）となり、スループットが最大化される。クライアント SDK はドキュメントで両モードの使い分けを明示する。

partition 数を Phase 途中で増加させると hash modulo が変わり、同一 aggregate の event が異なる partition に分散する順序破壊が発生する。partition 数の変更は major version（`.v2` トピック並列運用）を伴う破壊的変更として扱い、移行は DS-SW-EIF-210 のセマンティック非後方互換ゲートで承認を要する。

## Schema Registry 連携

event の payload スキーマが自由だと、消費側が payload をパースできず実行時エラーになる。Buf Schema Registry（BSR）でトピックごとに Protobuf スキーマを登録し、Publish 時に tier1 側で強制検証する。

**設計項目 DS-SW-EIF-266 Buf Schema Registry 連携**

各トピック `<tenant_id>.<domain>.<event_type>.v<major>` に対応する Protobuf message を BSR に登録する。BSR 上のパス規約は `buf.build/k1s0/<tenant_id>/<domain>/<event_type>/v<major>` とする。`facade-pubsub` は Publish 時に以下を実施する。

1. 受信 payload の `data_content_type` を確認。
2. `application/protobuf` の場合は BSR から対応 schema を取得し、payload を schema でデシリアライズして検証。
3. `application/cloudevents+json` の場合は BSR 登録 schema の JSON マッピングで検証。
4. schema 不一致は `PUBSUB_SCHEMA_MISMATCH` を返して Publish を拒否。

検証のレイテンシ加算は p99 5ms（BSR schema のローカルキャッシュ前提、cache miss 時は 30ms）。cache は `facade-pubsub` Pod 内で 5 分 TTL、BSR 更新時は Subscribe で cache invalidate する（BSR の webhook 経由）。schema 不一致を Publish 時点で弾くことで、消費側の実行時エラーを構造的に封じる。

Phase 1a では BSR 連携を optional とし、警告ログのみ出力する運用から開始する。Phase 1b 以降は mandatory に切り替え、BSR 未登録トピックへの Publish を拒否する構成に移行する。

## Ack / Nack と delivery_token

at-least-once 配信で正確な ack 管理をするため、tier1 側で delivery_token を採番してクライアントに渡し、ack 時に該当 token を突き合わせる。

**設計項目 DS-SW-EIF-267 delivery_token の採番と突合**

`facade-pubsub` が Subscribe で event を配信する際、`<partition>:<offset>:<uuid>` 形式の delivery_token を生成しクライアントに同送する。uuid 部分は tier1 が Valkey に `<delivery_token> → <partition, offset, consumer_group, subscribed_at>` を TTL 60 秒で記録する。クライアントが Ack / Nack を送ってきた際、token から partition/offset を復元して Kafka に commit する。

token に partition/offset を含める理由は、Valkey が一時的にダウンしても token 単独から復元可能にするフォールバックを確保するため。uuid を含める理由は同一 offset で token が重複発行されるエッジケース（再配信時）を区別するため。TTL 60 秒は Subscribe の処理猶予（30 秒）+ network RTT + 余裕の経験値で、60 秒超過した Ack は token 失効として `PUBSUB_TOKEN_EXPIRED` を返す。

## 固有エラーコード

共通規約 DS-SW-EIF-203 に基づき、PubSub 固有エラーを 10300〜10399 に採番する。

**設計項目 DS-SW-EIF-268 PubSub 固有エラーコード**

| enum 値 | 番号 | gRPC status | HTTP | 発生条件 |
|--------|------|------------|------|---------|
| `PUBSUB_TOPIC_NOT_FOUND` | 10300 | `NOT_FOUND` | 404 | Subscribe 対象トピック未存在 |
| `PUBSUB_TOPIC_INVALID_NAME` | 10301 | `INVALID_ARGUMENT` | 400 | 命名規約違反 |
| `PUBSUB_SCHEMA_MISMATCH` | 10302 | `INVALID_ARGUMENT` | 400 | BSR 登録 schema 不一致 |
| `PUBSUB_PAYLOAD_TOO_LARGE` | 10303 | `INVALID_ARGUMENT` | 413 | 1MB 超過 |
| `PUBSUB_QUOTA_EXCEEDED` | 10304 | `RESOURCE_EXHAUSTED` | 429 | テナント別 publish rate quota 超過 |
| `PUBSUB_CONSUMER_LAG_EXCEEDED` | 10305 | `FAILED_PRECONDITION` | 424 | Consumer Group のラグが閾値（10000 messages）超過、SRE 通知済 |
| `PUBSUB_TOKEN_EXPIRED` | 10306 | `FAILED_PRECONDITION` | 424 | Ack / Nack の token 失効（TTL 60 秒超過） |
| `PUBSUB_DLQ_FULL` | 10307 | `RESOURCE_EXHAUSTED` | 503 | DLQ トピック容量逼迫、SRE 対応中 |

エラー発生時は `K1s0Error.details[]` に `google.rpc.ErrorInfo` で `tenant_id` / `topic` / `consumer_group`（該当する場合）を付与し、運用側の調査を即時化する。

## サイズ上限

Kafka の `max.message.bytes` 初期値は 1MB で、これを超える message は消費側のメモリ・レイテンシを圧迫する。tier1 も 1MB を上限とする。

**設計項目 DS-SW-EIF-269 Message サイズ上限**

単一 event の payload（`data` フィールド）は最大 1MB。根拠は以下の 3 点。

1. Kafka `max.message.bytes` 初期値 1MB と整合。Strimzi デフォルトを使用するため拡張変更は不要。
2. 1MB を超える payload は S3 / MinIO に保存し、event には参照 URL のみ載せる「Claim Check パターン」で対応すべきで、これは Binding API の領分（`05_Binding_API方式.md` で定義）。
3. Ordering を保つ partition 内で連続する巨大 message は他 event の消費を遅延させ、SLO に直接影響するため上限を保守的に設定する。

1MB 超過は `PUBSUB_PAYLOAD_TOO_LARGE` を返す。クライアント SDK は 512KB を超えた時点で warning log を出し、Claim Check パターンへの移行を案内する。

## SLO 内訳

親ファイル DS-SW-EIF-013 で Publish p99 50ms と宣言した。本ファイルで Subscribe end-to-end レイテンシ p99 200ms も追加定義し、両者の区間別内訳を明示する。

**設計項目 DS-SW-EIF-270 SLO 区間別内訳**

| 操作 | p99 | 内訳 |
|------|-----|------|
| Publish | 50 ms | SDK 1 + NW 2 + Envoy 2 + facade 2 + BSR schema check 5（cache hit） + Dapr UDS 1 + Kafka acks=all + ISR sync 30 + 応答 7 |
| Subscribe end-to-end（Publish → 消費側配信） | 200 ms | Publish 50 + Kafka broker fetch wait 100 + facade 配信 20 + NW 20 + クライアント到達 10 |
| Ack | 20 ms | SDK 1 + NW 2 + Envoy 2 + facade 2 + Valkey token 照合 1 + Kafka commit 10 + 応答 2 |
| Nack | 20 ms | Ack と同等の処理経路 |

Subscribe の 100ms 配信ウェイトは Kafka `fetch.max.wait.ms` の既定値（500ms）を 100ms に短縮してレイテンシ重視にする設定値を前提とする。fetch.max.wait.ms を短くすると broker への fetch 回数が増えるが、Phase 1b のスループット 5000 msg/sec では broker 負荷に余裕がある。

## Phase 別展開

Phase 1a は単一 Kafka broker、Phase 1b で Strimzi 3 broker に拡張する。拡張時の互換性リスクを明示する。

**設計項目 DS-SW-EIF-271 Phase 別 Kafka 構成**

| Phase | Kafka 構成 | 制約 |
|-------|-----------|------|
| Phase 1a（MVP-0） | 単一 broker、KRaft 単一 controller、partition 3 / topic、replication 1 | 可用性低、データロスリスクあり、開発検証用途 |
| Phase 1b（MVP-1a） | Strimzi 3 broker、KRaft quorum 3 controller、partition 12 / topic、replication 3、ISR 2 min | 本番 SLO 達成構成、broker 1 台障害で継続可 |
| Phase 1c 以降 | Strimzi 5 broker、partition 24 / topic、Rack-aware 配置、Tiered Storage（S3 長期保管） | 高スループット + 長期 retention、SaaS 外販向け |

Phase 1a → 1b の移行時、既存 partition の replication factor を 1 → 3 に変更する運用作業が発生する（`kafka-reassign-partitions`）。移行計画は `../../../40_運用ライフサイクル/03_環境構成管理.md` で別途定義する。クライアント SDK は Phase 差を意識せず同一 API で呼び出せる抽象を維持する。

## 対応要件一覧

本ファイルは PubSub API の詳細方式設計であり、要件 ID → 設計 ID の 1:1 対応を以下の表で固定する。表形式併記は DR-COV-001 への緩和策として、CI スクリプトでの機械検証の一次入力となる。

| 要件 ID | 要件タイトル | 対応設計 ID | カバー状況 |
|---|---|---|---|
| FR-T1-PUBSUB-001 | Publish と CloudEvents v1.0 準拠 | DS-SW-EIF-260, DS-SW-EIF-261 | 完全 |
| FR-T1-PUBSUB-002 | Subscribe と Consumer Group 分離 | DS-SW-EIF-262 | 完全 |
| FR-T1-PUBSUB-003 | at-least-once と DLQ | DS-SW-EIF-263, DS-CTRL-MSG-003 | 完全 |
| FR-T1-PUBSUB-004 | Ordering とトピック命名強制 | DS-SW-EIF-264 | 完全 |
| FR-T1-PUBSUB-005 | Schema Registry 連携 | DS-SW-EIF-265 | 完全 |
| NFR-B-PERF-005 | PubSub Publish p99 50ms / Subscribe e2e p99 200ms | DS-SW-EIF-266, DS-NFR-PERF-004 | 完全 |
| NFR-C-OPS-020 | DLQ 監視と SRE アラート | DS-SW-EIF-267, DS-NFR-OPS-020 | 完全 |
| NFR-E-AC-007 | テナント分離（topic 命名強制） | DS-SW-EIF-264, DS-CF-AUTHZ-004 | 完全 |

表に載せた要件数は FR-T1-PUBSUB-* 5 件 + NFR 3 件 = 計 8 件。

補助参照は以下のとおり。

- ADR 参照: ADR-TIER1-001（Go+Rust）/ ADR-DATA-002（Kafka/Strimzi 採用、KRaft）/ ADR-PUBSUB-NNN（CloudEvents v1.0 準拠、未起票、Phase 1a 前起票予定）
- 共通規約参照: [00_API共通規約方式.md](00_API共通規約方式.md) DS-SW-EIF-200〜211
- 親参照: [01_tier1_11API方式概要.md](../01_tier1_11API方式概要.md) DS-SW-EIF-001 / 013 / 016
- 本ファイルで採番: DS-SW-EIF-260 〜 DS-SW-EIF-271
