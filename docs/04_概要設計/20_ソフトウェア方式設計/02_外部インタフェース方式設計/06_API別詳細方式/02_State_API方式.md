# 02. State API 方式

本ファイルは IPA 共通フレーム 2013 の 7.1.2.2「外部及びコンポーネント間のインタフェース方式設計（外部側）」に対応し、tier1 11 API のうちキー・バリュー状態管理を担う State API の詳細方式を定める。共通契約は [00_API共通規約方式.md](00_API共通規約方式.md)（DS-SW-EIF-200〜211）を参照し、本ファイルは State 固有の Protobuf 定義・キースコープ強制・ETag 楽観ロック・TTL・Transaction・Bulk・Consistency モード・SLO 内訳の 8 点を深掘りする。

## 本ファイルの位置付け

State API は tier2 / tier3 が共有キャッシュ、セッションストア、短命な業務状態を保持するための唯一の正規経路である。親ファイル [01_tier1_11API方式概要.md](../01_tier1_11API方式概要.md) の DS-SW-EIF-001 で Valkey Cluster をバックエンドに採用し、Go ファサード（`facade-state` Pod）が Dapr State コンポーネント経由でアクセスする構成を確定させた。業務アプリから見ると「キー名と値を指定するだけで、テナント分離・楽観ロック・TTL・トランザクションがすべて付随する」抽象として機能する。

この抽象が崩れる代表シナリオは 2 つある。1 つ目はキー名の衝突でテナント間の情報混入が発生するシナリオ、2 つ目は ETag を実装しないクライアントが楽観ロック競合を検出できずに Lost Update を起こすシナリオである。本ファイルは両者を tier1 側で強制する仕組みを明示する。

State API は Service Invoke と並んで p99 ミリ秒台の厳しい SLO を課されており、Valkey を採用した理由（Redis 7.2 BSL 回避、in-memory op、単一ホップ）が SLO 達成の直接的な根拠となる。TTL と Transaction の仕様は Valkey（Redis 互換）のネイティブ機能を忠実にラップする方針とし、Dapr 独自の拡張機能（例: Dapr State Store の secondary index）は採用しない。Dapr 固有拡張を使うと将来の Valkey 直接実装への移行路が閉ざされる。

## Protobuf Service 定義

State API は 6 つの RPC（Get / Set / Delete / BulkGet / BulkSet / Transaction）を公開する。RPC を 6 つに分離する理由は、Bulk 版と単一版で内部実装が大きく異なる（Valkey MGET / MSET vs GET / SET）ためと、Transaction の複数操作セマンティクスを単一メソッドに混ぜると誤用を招くためである。

**設計項目 DS-SW-EIF-240 Protobuf Service 定義**

`protos/k1s0/public/state/v1/state.proto` に以下を定義する。

```protobuf
service State {
  rpc Get(GetRequest) returns (GetResponse);
  rpc Set(SetRequest) returns (SetResponse);
  rpc Delete(DeleteRequest) returns (DeleteResponse);
  rpc BulkGet(BulkGetRequest) returns (BulkGetResponse);
  rpc BulkSet(BulkSetRequest) returns (BulkSetResponse);
  rpc Transaction(TransactionRequest) returns (TransactionResponse);
}

message GetRequest {
  // Dapr State Store 名（Phase 1a は "valkey-primary" 固定）
  string store_name = 1;
  // キー（DS-SW-EIF-242 の prefix は tier1 側で自動付与）
  string key = 2;
  // Consistency モード（DS-SW-EIF-247 参照）
  Consistency consistency = 3;
}

message GetResponse {
  // 値。未存在時は空 bytes かつ error=KEY_NOT_FOUND
  bytes data = 1;
  // ETag（楽観ロック用、DS-SW-EIF-243）
  string etag = 2;
  // 残 TTL 秒（未設定時は -1）
  int64 ttl_remaining_seconds = 3;
}

message SetRequest {
  string store_name = 1;
  string key = 2;
  bytes value = 3;
  // ETag 指定時は if-match、空の場合は無条件 set
  string etag = 4;
  // TTL 秒（DS-SW-EIF-244、0 は無期限、最大 2592000=30 日）
  int64 ttl_seconds = 5;
  Consistency consistency = 6;
  // concurrency=FirstWrite で楽観ロック、LastWrite で強制上書き
  Concurrency concurrency = 7;
}

enum Consistency {
  CONSISTENCY_UNSPECIFIED = 0;
  CONSISTENCY_EVENTUAL = 1;
  CONSISTENCY_STRONG = 2;
}

enum Concurrency {
  CONCURRENCY_UNSPECIFIED = 0;
  CONCURRENCY_FIRST_WRITE = 1;
  CONCURRENCY_LAST_WRITE = 2;
}
```

Delete / BulkGet / BulkSet / Transaction のメッセージも対称的に定義する。Transaction は `repeated TransactionOperation operations` を保持し、各操作が `oneof { SetRequest set; DeleteRequest delete; }` で表現される。

## キースコープ強制

State API で最もクリティカルなリスクは、テナント間でキー名が衝突して情報混入が発生することである。親ファイル DS-SW-EIF-004 でテナント ID 伝搬を必須化したが、State のキー命名で tier1 側強制がないとヘッダ付与漏れが即座に情報混入に直結する。

**設計項目 DS-SW-EIF-241 キープレフィックス強制**

クライアントが指定する `key` は tier1 側で必ず `tenant_id:<tid>:app:<aid>:<key>` のプレフィックスを自動付与して Valkey に保存する。tid は JWT claim から取得し、aid は JWT claim の `app_id`（Keycloak client_id）から取得する。クライアントが自らプレフィックスを含めた key を送ってきた場合は tier1 側が検出して二重付与しないよう正規化するが、`tenant_id:` / `app:` といった予約プレフィックスのリテラル使用は VALIDATION エラーで拒否する。

この設計により、仮にクライアント側が tenant-id ヘッダを偽装しようとしても JWT 由来の正規 tenant_id が key に強制されるため、情報混入の物理的な経路が閉じる。Valkey の DBSIZE や KEYS コマンドで一覧を見ても `tenant_id:t0001:app:order:*` のようにテナント別に分離されており、運用時のデータ漏洩リスクも最小化される。

**設計項目 DS-SW-EIF-242 キー名のサイズ上限と文字種**

クライアント指定の key（prefix 除く）は最大 256 byte、文字種は `[a-zA-Z0-9_\-:.]` に制限する。根拠: 256 byte は Valkey の理論上限（512MB）より遥かに保守的で、監視ツール・ログの表示上の可読性を重視した値。文字種制限は `*` `?` などのワイルドカード文字を排除し、誤って `KEYS` コマンド互換のパターンを送信するミスを防ぐ。プレフィックス付与後の最終キー長は最大 512 byte（tenant 16 + app 32 + 区切り + user 256 + マージン）を超えないことを `facade-state` 側で validation する。

## ETag 楽観ロック

複数クライアントが同じキーを同時更新すると Lost Update が発生する。State API は ETag 楽観ロックを標準機能として提供し、クライアントは If-Match セマンティクスで競合を検出できる。

**設計項目 DS-SW-EIF-243 ETag 楽観ロックの仕様**

Get レスポンスの `etag` は Valkey の `WATCH` コマンドと同等のバージョン識別子として機能する。実装上は Valkey の `HINCRBY <key>:_meta version 1` で生成される単調増加整数を base16 エンコードした文字列とする。Set リクエストで `etag` を指定した場合、`facade-state` は Valkey `WATCH` + `MULTI` + `GET etag` + `EXEC` の Lua script で原子的に比較し、不一致なら `STATE_ETAG_MISMATCH` を返す。

`Concurrency` enum が `FIRST_WRITE` の場合は etag 指定必須、空 etag は key 未存在を意味する（Redis `SET NX` 相当）。`LAST_WRITE` の場合は etag を無視して強制上書きする。クライアント SDK は FIRST_WRITE をデフォルトに設定し、明示的にオプトアウトしない限り楽観ロックが有効になる構成とする。

HTTP/JSON 経路では If-Match ヘッダで ETag を送信し、不一致時は HTTP 412 Precondition Failed を返す。gRPC では `ABORTED` (10) / HTTP 409 Conflict にマッピングする（HTTP 412 との差異は共通規約 DS-SW-EIF-204 で HTTP 側のみ 412 を使用する例外規定とする）。

## TTL

短命な業務状態（セッション、一時トークン、キャッシュ）を自動削除する機能が必要。TTL は Valkey の EXPIRE コマンドで実装するが、上限値を設定しないと誤って数年の TTL を設定して Valkey のメモリを食い尽くす事故が起きる。

**設計項目 DS-SW-EIF-244 TTL の仕様と上限**

`ttl_seconds` は 0（無期限）または 1 〜 2592000（30 日）を許容する。30 日を上限とする根拠は以下の 3 点。

1. 30 日を超える長期データは State ではなく Audit-Pii（Postgres）または Binding（Output）で S3 / MinIO に永続化すべきで、Valkey の in-memory 特性と合致しない。
2. Phase 1a の Valkey は単一ノード構成（メモリ 16 GB）で、長期データが蓄積すると OOM リスクが高まる。30 日でサイクリングすることで実効メモリ使用量を予測可能に保つ。
3. 運用側のコスト予測（$200/月/クラスタ の Phase 1a 想定）の前提値として 30 日上限を採用している。

上限超過の TTL 指定は `VALIDATION_SIZE_EXCEEDED` を返す。TTL 0（無期限）は tier1 側では禁止しない（Feature Flag で切替可）が、デフォルトは 24 時間を推奨しクライアント SDK のドキュメントで明示する。無期限 data が大量に蓄積した場合は SRE が `k1s0_state_keys_without_ttl` メトリクスで検知してテナントに是正依頼を出す運用とする。

## Transaction

複数キーの原子更新は業務ロジックの正しさに必須だが、分散トランザクションを tier1 で提供すると複雑度が跳ね上がる。State API は「単一 Valkey ノード内のローカルトランザクション」に限定し、クロスノード・クロスバックエンドのトランザクションは Workflow API（Saga パターン）に委譲する責務分離とする。

**設計項目 DS-SW-EIF-245 Transaction の仕様**

Transaction は 1 リクエスト内に最大 10 操作を含められる。内部実装は Valkey の `MULTI` / `EXEC` で原子的に実行する。1 トランザクション内のキーがすべて同一 Valkey ノード上にあることを前提とし、Phase 1b の Cluster 構成では hash tag `{tenant_id}:{aid}` で slot 固定することで同一ノード配置を保証する。

10 操作の上限根拠は、Valkey の `MULTI`/`EXEC` の実行時間が操作数に線形比例し、10 操作で p99 50ms を安定達成できる計測値（社内 bench、8KB value × 10 ops で 32ms p99）に由来する。11 操作以上は `VALIDATION_SIZE_EXCEEDED` を返し、クライアントには Workflow API への切替または操作分割を案内する。

Transaction 内の楽観ロック衝突（1 つでも etag 不一致）は全操作を abort し、`STATE_ETAG_MISMATCH` を返す。部分コミットは存在しない（Valkey の `EXEC` 仕様通り）。

## Bulk 操作

複数キーの取得・設定を 1 リクエストで行う効率化。Valkey の `MGET` / `MSET` を使うが、無制限な一括処理は Valkey のレイテンシを押し上げるため上限設定が必要。

**設計項目 DS-SW-EIF-246 Bulk の上限**

BulkGet / BulkSet は 1 リクエストあたり最大 100 キーまで。根拠は以下の 2 点。

1. Valkey の `MGET` / `MSET` は O(N) で、100 キーでも p99 30ms 以内に収まる実測値がある（社内 bench、1KB value × 100 keys で 18ms p99）。
2. 100 を超える場合はクライアント側でページングするほうが、Valkey の単一コマンド待機時間による全体ブロッキング（他クライアントからの Get 遅延）を避けられる。

101 キー以上は `VALIDATION_SIZE_EXCEEDED` を返す。BulkSet の各要素は個別 TTL と etag を指定可能。BulkGet は存在しないキーを `data=empty, etag=""` として返し、部分的に存在するケースでも成功応答を返す（全キー未存在時も成功、業務判定は呼出側）。

## Consistency モード

Valkey は Master-Replica 構成で、Replica 経由の Get は Eventual（数 ms の遅延）となる。厳密な Read-Your-Write セマンティクスが必要な業務には Strong モードを提供する。

**設計項目 DS-SW-EIF-247 Consistency モードの実装**

`Consistency.EVENTUAL` は Replica から Get し、レプリカラグ（典型 1〜5ms）を許容する。`Consistency.STRONG` は Master から Get し、Set の場合は Valkey の `WAIT <numreplicas> <timeout_ms>` コマンドで指定レプリカ数（Phase 1b は 2）への同期完了を待つ。STRONG モードは p99 が EVENTUAL 比 2〜3 倍（Set で 20ms → 50ms）になるため、クライアントは必要時のみ指定する。

デフォルト（UNSPECIFIED）は Set / Delete / Transaction は STRONG、Get / BulkGet は EVENTUAL とする。この非対称なデフォルト設定は「書き込みは確実に、読み込みは速度優先」という業務実利に合わせた設計である。EVENTUAL Get でレプリカラグによる古いデータ読み出しが発生した場合、Set 側の etag と Get 側の etag が乖離するため ETag 楽観ロックで事実上 Strong 相当の整合性が得られる。

## サイズ上限

Value のサイズ上限が未設定だと、巨大な blob を State に保存してしまい Valkey のメモリ圧迫・レイテンシ劣化を招く。適切な上限を設定する。

**設計項目 DS-SW-EIF-248 Value サイズ上限**

単一 value は最大 1 MB。根拠は以下の 3 点。

1. 1MB を超える blob は S3 / MinIO への保存が適切で、Valkey の in-memory 特性と逆行する。
2. Redis 公式推奨（1 value は KB 〜 数 MB、理論上限 512 MB は非推奨）と整合。
3. 1MB の value を 100 keys Bulk で受信すると 100 MB のペイロードになるが、これは DS-SW-EIF-207 の gRPC 4MB 制約に抵触するため、Bulk 時は実質的に各 value 40KB 以下に制限される（100 keys × 40KB = 4MB）。

1MB 超過は `VALIDATION_SIZE_EXCEEDED` を返す。クライアント SDK は 512KB を超えた時点で warning log を出し、S3 / MinIO への移行を案内する。

## 固有エラーコード

共通規約 DS-SW-EIF-203 に基づき、State 固有エラーを 10200〜10299 に採番する。

**設計項目 DS-SW-EIF-249 State 固有エラーコード**

| enum 値 | 番号 | gRPC status | HTTP | 発生条件 |
|--------|------|------------|------|---------|
| `STATE_KEY_NOT_FOUND` | 10200 | `NOT_FOUND` | 404 | Get / Delete で対象キー未存在 |
| `STATE_ETAG_MISMATCH` | 10201 | `ABORTED` | 409（HTTP では 412） | 楽観ロック競合 |
| `STATE_TTL_EXCEEDED` | 10202 | `INVALID_ARGUMENT` | 400 | TTL 上限 30 日超過 |
| `STATE_VALUE_TOO_LARGE` | 10203 | `INVALID_ARGUMENT` | 413 | value 1MB 超過 |
| `STATE_TX_TOO_MANY_OPS` | 10204 | `INVALID_ARGUMENT` | 400 | Transaction 11 操作以上 |
| `STATE_TX_CROSS_SLOT` | 10205 | `FAILED_PRECONDITION` | 424 | Transaction 内キーが同一 slot 外（Phase 1b Cluster） |
| `STATE_QUOTA_EXCEEDED` | 10206 | `RESOURCE_EXHAUSTED` | 429 | テナント別容量 quota（例 1GB/tenant）超過 |
| `STATE_BACKEND_REPLICA_LAG` | 10207 | `FAILED_PRECONDITION` | 424 | STRONG モード時に WAIT タイムアウト |

エラー発生時は `K1s0Error.details[]` に `google.rpc.ErrorInfo` で `tenant_id` / `store_name` / `key`（PII 非該当のため露出可）を付与する。

## SLO 内訳

親ファイル DS-SW-EIF-013 で State Get p99 10ms、Set p99 20ms、Transaction p99 50ms（本ファイルで追加）を宣言した。区間別内訳を明示する。

**設計項目 DS-SW-EIF-250 p99 レイテンシ内訳**

| 操作 | p99 | 内訳 |
|------|-----|------|
| Get（EVENTUAL） | 10 ms | SDK 1 + NW 2 + Envoy 2 + facade 1 + Dapr UDS 1 + Valkey 2 + 応答 1 |
| Get（STRONG） | 15 ms | 上記 + Master 固定（+2ms）+ シリアライズ余裕（+3ms） |
| Set（STRONG） | 20 ms | SDK 1 + NW 2 + Envoy 2 + facade 1 + Dapr UDS 1 + Valkey WAIT 2 レプリカ 10 + 応答 3 |
| Transaction（10 ops） | 50 ms | 上記 Set + MULTI/EXEC 直列 10 op × 2ms + WAIT 15 |
| Delete | 15 ms | Set 相当（WAL 書き込みあり） |

各区間は Prometheus `k1s0_state_latency_seconds{op="...", stage="..."}` で観測可能にし、SLO 逸脱時の原因切り分けを自動化する。

## Phase 別展開

Phase 1a は単一 Valkey ノード、Phase 1b で Cluster 3 shard に拡張する。拡張時の互換性リスクを明示する。

**設計項目 DS-SW-EIF-251 Phase 別 Valkey 構成**

| Phase | Valkey 構成 | 制約 |
|-------|-----------|------|
| Phase 1a（MVP-0） | 単一ノード + 1 レプリカ、メモリ 16GB | Transaction は全キー同一ノード自動成立、Cluster 制約なし |
| Phase 1b（MVP-1a） | Cluster 3 shard、各 shard 1 Master + 1 Replica、メモリ合計 48GB | Transaction 内キーは hash tag `{tenant:aid}` で slot 固定必須、cross-slot は `STATE_TX_CROSS_SLOT` |
| Phase 1c 以降 | Cluster 6 shard、Read Replica 追加 | Consistency.EVENTUAL は Read Replica から serve |

Phase 1a → 1b の移行時、既存キーのハッシュタグ再配置が必要。移行計画は `../../../40_運用ライフサイクル/03_環境構成管理.md` で別途定義する。クライアント SDK は Phase 差を意識せず同一 API で呼び出せる抽象を維持する。

## 対応要件一覧

本ファイルは State API の詳細方式設計であり、以下の要件 ID に対応する。

- FR-T1-STATE-001〜FR-T1-STATE-005（State 機能要件一式）
- FR-T1-STATE-001（Get/Set/Delete 基本操作）/ FR-T1-STATE-002（Bulk と Transaction）/ FR-T1-STATE-003（ETag 楽観ロック）/ FR-T1-STATE-004（TTL とキースコープ強制）/ FR-T1-STATE-005（Consistency モード）
- NFR-E-AC-006（テナント分離、key prefix 強制）
- NFR-B-PERF-002（State p99 10ms / 20ms）
- NFR-G-CLS-003（PII は Audit-Pii 側で、State では扱わない）
- ADR 参照: ADR-TIER1-001（Go+Rust）/ ADR-DATA-004（Valkey 採用、Redis BSL 回避）
- 共通規約参照: [00_API共通規約方式.md](00_API共通規約方式.md) DS-SW-EIF-200〜211
- 親参照: [01_tier1_11API方式概要.md](../01_tier1_11API方式概要.md) DS-SW-EIF-001 / 013 / 016
- 本ファイルで採番: DS-SW-EIF-240 〜 DS-SW-EIF-251
