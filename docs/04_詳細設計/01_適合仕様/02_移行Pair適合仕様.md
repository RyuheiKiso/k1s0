---
id: detail.tier1.migration_pair_conformance
axis: tier1
phase: detail
kind: conformance_spec
status: published
version: 1.0.0
depends_on:
  - arch.tier1.tier1_index
  - detail.meta.axis_registry_conformance
covered_by:
  defense_in_depth_layers: [A, B, C, D, E]
  proof_classes:
    - v1_program_correctness_proof
    - v1_refinement_proof
lock_artifacts:
  - dry_run.lock.yaml
trace:
  fr_ids:
  - FR-tier1-002

---

# tier1 L1+ 移行 Pair 適合仕様（v1）

## 一文定義
- 移行 Pair 適合仕様は、L1+ カテゴリ 4 つ × 1 primary pair × 5 phase（schema_diff / state_replicate / dual_write_ramp / cutover / rollback）の cross-product として宣言される 20 cell に対し、Testcontainers で 1.0.0 ship 前 + 年次 dry-run green を物理 enforce し、L1+ 単一深耕の哲学を migration 軸に転写する meta-axis である。

## 位置づけ
- [Library](../../03_概要設計/02_tier1設計方針/02_Library.md) L1+ 移行コミットメント節が宣言する「移行先候補（カテゴリごとに 1 つ以上）を文書化し、Testcontainers での dry-run 移行テストを年次で実施する。dry-run の green を Library リリース条件に含める」を、機械可読な単一の真として一箇所に固定
- 04_Library が候補を複数列挙する一方で、1.0.0 ship blocker となる「dry-run green 必須 pair」を本仕様で 4 pair に固定し、各 pair × phase × assertion を全数機械検証可能な形で組む

## 設計原則
- **1 カテゴリ = 1 primary pair**: L1+ カテゴリ各々に対し、1.0.0 ship 前に dry-run green を要件とする primary pair を 1 つだけ確定。複数候補の並走は採らない
- **移行 phase は 5 段階で固定**: phase の追加 / 順序変更は破壊的変更（major version up）
- **dry-run は 1.0.0 ship 前と年次**: 1.0.0 ship 前に少なくとも 1 度の全 pair × 全 phase green を必須（ship blocker）。1.0.0 以降は年次で同 dry-run、green 維持を Library リリース条件
- **pure OSS 内移行を 1.0.0 で証明**: cloud lock-in 想定（Aurora / Neon / AWS MSK / Confluent Cloud / Temporal Cloud 等）への移行は dry-run primary pair から除外
- **defense-in-depth は 5 層**: pair 宣言 / phase assertion / Testcontainers / runtime / 物理（年次 cadence + release blocker）
- **dead spec を CI で殺す**: pair / phase / assertion / scenario のいずれも参照消失で CI fail

## v1 primary pair セット（4 pair、1.0.0 ship 前 dry-run green 必須）

| pair_id | from（本命） | to（移行先 primary） | wire 互換 |
|---|---|---|---|
| `relational_pg_pair` | PostgreSQL（CloudNativePG） | PostgreSQL（StackGres） | 完全（同 engine） |
| `messaging_kafka_pair` | Apache Kafka（Strimzi） | RedPanda | wire 完全（Kafka API） |
| `workflow_pair` | Temporal | Cadence | 部分（state model） |
| `rule_engine_pair` | ZEN Engine | 自製 DSL backend（Rust 実装） | DSL 真として同一表現 |

### 各 pair 選定理由
- `relational_pg_pair`: 同 PostgreSQL engine、Operator 差のみ。pure OSS 内 + 同 engine + 同 wire の最小移行 pair。1.0.0 で証明する難度が現実的かつ意味的に十分
- `messaging_kafka_pair`: Kafka wire 完全互換、別実装（C++ vs JVM）で wire 互換の真価が現れる pair
- `workflow_pair`: 元々同一プロジェクト由来、state model 概念が近い。業務コード影響の最も大きい pair（露出概念一覧で WorkflowID 体系 / 補償処理の書き方 / 決定論実行制約 で差）。これを 1.0.0 で覆うことが移行コミットメントの中核
- `rule_engine_pair`: ZEN Engine の決定表 DSL を真として、評価エンジンを Rust で再実装。tier1 Library が ZEN の DSL semantics を内部表現として完全把握していることを証明

4 pair で v1 covering。新 L1+ カテゴリ（Distributed SQL 予約カテゴリ等）が確定した時点で v2 候補として pair 追加検討。

## v1 移行 phase セット（5 phase、決定論的順序）

| phase_id | 入力 | 出力 | 業務影響 |
|---|---|---|---|
| `schema_diff` | 旧 backend の schema / state 定義 | 差分レポート + tier2 露出概念影響箇所一覧 | なし（読取のみ） |
| `state_replicate` | 旧 backend の状態スナップショット | 新 backend の同期状態 | なし（旧側継続） |
| `dual_write_ramp` | 業務トラフィック | 旧 / 新両方への二重書込、読取 ramp 切替 | 読取は旧 100% → 新 100% へ ramp |
| `cutover` | dual_write 完了状態 | 新 backend が単独動作 | 短時間の write freeze |
| `rollback` | cutover 後 24h 以内の異常検知 | 旧 backend への切戻し | 短時間の write freeze |

### 各 phase の不変条件
- `schema_diff`: 旧 / 新の schema / state 定義を機械的に diff し、業務コード影響箇所を露出概念一覧に射影してチェックリスト化。tier2 / tier3 への影響度を「無修正 / 設定変更 / コード修正」の 3 段階で分類。business outcome に影響する差分（Workflow ID 体系変更、ロック挙動差、SQL 方言差等）は赤線扱い
- `state_replicate`: 旧 backend の現在状態を新 backend に複製。完了後に状態整合（行数 / hash / sequence / consumer group offset 等）を assertion で検証。業務 traffic は旧 backend に流れ続ける
- `dual_write_ramp`: tier1 Library 抽象が dual-write 経路を持ち、書込は両 backend、読取は旧 → 新へ ramp で切替（0% / 5% / 25% / 50% / 75% / 100%）。各 ramp 段階で 32 適合仕様の integrity property (P1)〜(P4) が両 backend で満たされることを assertion で検証。39 の Domain Event subscription が中断しないことを `resume_token` で透過化
- `cutover`: 短時間の write freeze（数秒〜数十秒）で書込を新 backend のみに切替。tier1 Library の dual-write を新側 only に切替。cutover 直前 / 直後の business outcome の連続性を observability connection（trace_id / span 名）で検証
- `rollback`: cutover 後 24h 以内の異常検知で旧 backend に切戻す経路。cutover 中に新 backend が受けた書込は、rollback 時に旧 backend に再生（reverse replication）。24h 以降の rollback は新規移行扱い（再 dual_write_ramp）

## 単一の真

### `pairs.yaml`（軸 catalog）
- pair_id → from / to / wire 互換 / 露出概念影響カテゴリ の純粋関数テーブル

### `phases.yaml`（軸 catalog）
- phase カタログ + pair × phase × assertion id マトリクス（4 pair × 5 phase = 20 cell すべて非空、unreachable cell 検出）

### `test_matrix.yaml`（軸 catalog）
- dry-run scenarios。4 pair × 5 phase の全組合せを Testcontainers で実行する scenario カタログ
- 障害注入 dry-run（replication lag / network partition / disk full / clock skew）も同 matrix に含める

### `dry_run.lock.yaml`（build artifact）
- 各 pair の `last_green_at` / `version_pinning` / `toolchain_revision` を記録。手書き禁止、年次 dry-run の green を CI が自動更新
- 1.0.0 release の必要条件は本 lock に 4 pair すべての `last_green_at` が「ship 日から 365 日以内」かつ「`toolchain_revision` が現行 release tag と整合」であること

## 5 層 defense-in-depth
- 層 A: pair 宣言（`pairs.yaml`）。pair_id を tier1 内部の Rust crate と enum で claim。`exposed_concepts_affected` は 04_Library 露出概念一覧と CI 照合（drift 検出）
- 層 B: phase / assertion 宣言（`phases.yaml`）。全 assertion id を Rust test runner が attribute claim（`#[migration_assert("...")]`）
- 層 C: Testcontainers での dry-run 自動実行（`test_matrix.yaml`）。4 pair × 5 phase の全 scenario を並列実行
- 層 D: runtime（露出概念整合検査）。tier2 業務コード移行ガイドは `exposed_concepts_affected` を起点に機械的に生成、手書き diff があれば fail
- 層 E: 物理層（年次 cadence + release blocker）。`dry_run.lock.yaml` の `last_green_at` が 365 日を超えた pair があれば、Library のリリース pipeline を物理的に block

## atomic_triple_write と tier1 schema 配下の tier2bridge 配置

`src/tier1/schema/tier2/buf.yaml` および `src/tier1/schema/tier2/tier1/tier2bridge/v1/triple_write.proto` が tier1 schema 配下に置かれる理由を以下に明示する。

**設計決定**: tier1 transport layer が tier2 の atomic_triple_write の wire 形式を定義する必要があるため、tier2bridge の proto schema は tier1 schema 配下（`src/tier1/schema/tier2/`）に置かれる。

- tier1 transport は Bidi RPC の物理 wire 形式（gRPC / Connect-RPC）を所有する
- tier2 の atomic 三表書込（State change + Outbox + Audit を同一トランザクション内で書く）は、tier1 transport を介した RPC として実装される
- したがって、その wire 形式（`triple_write.proto`）は tier1 が SoT として定義しなければならない
- これは axis 越境配置ではなく、「tier1 transport layer が tier2 の atomic_triple_write の proto schema を所有する」という設計決定である

この配置は以下の不変条件を満たす:
- tier2 実装は `triple_write.proto` の生成コードを buf generate で取得する（手書き禁止）
- tier2 が tier1 transport proto を直接 modify することは禁止（CR 経由で tier1 maintainer が修正）
- tier1 / tier2 の依存方向（tier2 は tier1 proto 生成コードに依存、逆方向禁止）を維持する

## 三軸との接合
本仕様は三軸（07 transport / 32 data / 39 client）の上位レイヤとして、移行軸の不変条件を形式化する。dual_write_ramp / cutover / rollback の各 phase 中でも三軸の不変条件は保たれる必要がある:
- `relational_pg_pair` の dual_write_ramp / cutover: 32 atomic 三表書込 (P1)〜(P4) が両 backend で同時に成立、32 RLS が両 backend で同一 policy 適用
- `messaging_kafka_pair` の dual_write_ramp / cutover: 07 conformance_class（`v1_event_feed` の SESSION_ORDERED + REQUIRED + 5000ms）が両 backend で満たされる、07 `messaging_bridge` adapter の constraints（`partition_key = session_id`）が両 backend で同等に物理的成立
- `workflow_pair` の cutover: 39 client conflict tree の `resume_token` / Idempotency-Key が cutover 前後で連続性を保つ、39 pending_queue resume が cutover 中の write_freeze 区間で正しく hold され、解除後に新 backend に流れる
- `rule_engine_pair` の dual_write_ramp: 32 atomic 三表書込内で rule evaluation が両 backend で同一結果を返すことを assert

これら接合 assertion は `test_matrix.yaml` の各 scenario で `cross_axis_assertions` として claim する。

## CI 不変条件（merge 不可）
- 整合 1: `pairs.yaml` の全 pair が `phases.yaml` の全 phase に assertion を持つ（4 pair × 5 phase で 20 cell すべて非空）
- 整合 2: `phases.yaml` の全 assertion id が Rust test runner で実装され、`test_matrix.yaml` の少なくとも 1 scenario でカバーされる
- 整合 3: `test_matrix.yaml` の各 pair について full_dry_run + 障害注入 scenario が両方存在
- 整合 4: `pairs.yaml` の `exposed_concepts_affected` が 04_Library 露出概念一覧と完全整合
- 整合 5: `dry_run.lock.yaml` は手書き禁止、build artifact と git の差分検出
- 整合 6: 三軸接合 `cross_axis_assertions` が 07 / 32 / 39 の不変条件 id と参照整合
- 整合 7: 1.0.0 ship 前 release は 4 pair の `last_green_at` すべて存在することを ship blocker
- 整合 8: 1.0.0 後の release は 4 pair の `last_green_at` が 365 日以内
- 整合 9: `pairs.yaml` に from / to の version_pin が記録されており、本命 OSS の version up と toolchain 整合が年次で再検証される
- 整合 10: dead spec 検出（参照されない pair / phase / assertion / scenario は CI fail）

## 1.0.0 ship blocker
- `pairs.yaml` に 4 pair が登録され、`phases.yaml` に 5 phase × 4 pair = 20 cell すべて非空
- `test_matrix.yaml` の 10 scenarios（4 full + 6 障害注入）が Testcontainers で全 green
- `dry_run.lock.yaml` に 4 pair の `last_green_at` が記録されている
- 上記が満たされない限り 1.0.0 release tag は作成しない

## 製造業 pack stress test（v1 の正当性証跡）

| # | dry-run scenario | pair | 主要 phase |
|---|---|---|---|
| 1 | 発注 / 検査 aggregate (32) を CNPG → StackGres | `relational_pg_pair` | 全 5 phase |
| 2 | Domain Event topic (39 event_feed) を Kafka → RedPanda | `messaging_kafka_pair` | 全 5 phase |
| 3 | 多段承認 Workflow を Temporal → Cadence | `workflow_pair` | 全 5 phase |
| 4 | 検査ルール決定表を ZEN → 自製 DSL backend | `rule_engine_pair` | 全 5 phase |
| 5 | replication_lag 注入 (CNPG → StackGres) | `relational_pg_pair` | dual_write_ramp |
| 6 | partition rebalance 中の cutover (Kafka) | `messaging_kafka_pair` | cutover |
| 7 | open workflow 1000 件超での cutover | `workflow_pair` | cutover |
| 8 | rule evaluation 不一致発覚 → rollback | `rule_engine_pair` | rollback |
| 9 | cutover 後 23h 経過時点の rollback | `relational_pg_pair` | rollback |
| 10 | dwr 中の atomic 三表書込 P1〜P4 違反検知 | `relational_pg_pair` | dual_write_ramp |

全 scenario が Testcontainers で 2h 以内に完走することを実装目標。1.0.0 ship 前に 10 scenario すべての green を要件。

## v2 拡張ルール
- 新 pair の追加は purely additive。Distributed SQL カテゴリ確定時に `distributed_sql_pair` を追加。Vector Search カテゴリ（L1+、本命 pgvector）の primary pair も v2 候補として予約
- 新 phase の追加は破壊的変更（major version up）。既存 4 pair × N phase の全 cell に対する assertion 追加が必須
- 既存 pair の primary 移行先変更（StackGres → Crunchy PGO 等）は破壊的変更扱い
- cloud lock-in 移行先（Aurora / Neon / AWS MSK / Confluent Cloud / Temporal Cloud）への対応は v2 以降。本仕様の primary pair には採用しない

## 採用しない設計
- L1+ カテゴリでの day-1 multi-backend 並走
- cloud lock-in 移行先を 1.0.0 primary pair に採用
- dry-run green の不在で 1.0.0 release tag 作成
- pair / phase / assertion の手動更新（全て build artifact 経由）

## parity_vectors.yaml SoT（4 言語等価強度検証ベクタ）

`src/tier1/library/parity_vectors.yaml` は、Rust / Go / C# / TypeScript の 4 言語 Library 実装が同一入力から同一出力を返すことを保証する検証ベクタの SoT である。

### 各 factory / method の等価 API シグネチャ一覧

| ベクタ id | module | operation | 入力 | 期待出力スキーマ（型制約） |
|---|---|---|---|---|
| `key_handle_generate_ed25519` | KeyHandle | generate | `{key_type: ed25519, purpose: signing}` | `{key_id: string, algorithm: ed25519, has_private: false}` |
| `auth_context_validate_jwt_format` | AuthContext | validate_format | `{token: "header.payload.signature"}` | `{valid_format: boolean, algorithm: string}` |
| `repository_tenant_scope_query` | Repository | tenant_scope_check | `{tenant_id: string, resource_id: string}` | `{in_scope: boolean}` |
| `quota_rate_limit_check` | Quota | check_limit | `{class: v1_standard, current_qps: integer}` | `{allowed: boolean, remaining: integer}` |
| `bidi_handshake_capabilities` | Bidi | negotiate_capabilities | `{conformance_class: c1_bidirectional_full}` | `{accepted: boolean, negotiated_class: string}` |
| `auth_context_v1_human_session` | AuthContext | create_session | `{auth_class: v1_human_session, session_id: string, tenant_id: UUID}` | `{access_token_exposed: false, auth_class: v1_human_session, tenant_id: UUID}` |
| `key_handle_v1_signing` | KeyHandle | generate_signing_key | `{key_class: v1_signing}` | `{raw_bytes_exposed: false, key_id: string}` |
| `idempotency_key_chaining` | IdempotencyKey | chain_key | `{original_key: string}` | `{starts_with_original_key: true, contains_wall_clock_timestamp: false, result_type: string}` |

### 等価強度の判定基準（equivalence_criteria）

- `function_name_semantic`: 関数名の命名規則差（camelCase / PascalCase / snake_case）は許容し、意味同一性で判定
- `argument_type_equivalent`: 言語の型システム差を考慮した等価型（例: Go `string` ↔ Rust `&str` ↔ TypeScript `string`）
- `return_type_equivalent`: 言語の async パターン差を許容（Go error tuple / Rust Result / TypeScript Promise / C# Task）
- `error_semantic_equivalent`: 4 言語で同一エラーセマンティクスを持つこと（同一エラー種別が同一状況で発生）

4 言語間で API surface に drift が生じた場合は `ship_blocker_on_drift: true`（`language_parity` 宣言）により CI fail となる。

## migration_commitments.yaml SoT（言語間 API 差分コミットメント）

`src/tier1/library/migration_commitments.yaml` は、L1+ カテゴリ各 OSS に対するライフサイクルイベント時の移行先候補と toolchain 整備状況を機械可読形式で宣言するコミットメント一覧の SoT である。

### L1+ 移行コミットメント一覧（カテゴリ × 移行先 primary pair）

| category | current_oss | current_operator | primary 移行先 | exposed_concepts（API 表面に露出する OSS 概念） |
|---|---|---|---|---|
| `relational_store` | postgresql | cnpg | postgresql（stackgres） | pg_advisory_lock / pg_skip_locked / pg_guc / pg_logical_replication / pg_vector_index_type |
| `messaging` | apache_kafka | strimzi | redpanda | kafka_consumer_group / kafka_transaction_producer / kafka_isolation_level / schema_registry_id |
| `workflow` | temporal | temporal_operator | cadence | temporal_workflow_definition / temporal_activity_definition / temporal_schedule |
| `rule_engine` | zen_engine | null | custom_dsl（Rust 実装） | zen_decision_table_format / zen_evaluation_context |
| `vector_search` | pgvector | null | pgvector（hnsw index） | pgvector_index_type / pgvector_distance_metric / pgvector_search_params |

### 引数順序差等の注意点

- `workflow_pair`（Temporal → Cadence）: Workflow ID 体系差・補償処理の書き方・決定論実行制約が差異として生じる。これは `exposed_concepts_affected` の `workflow_pair` エントリに赤線扱いで記録される
- `rule_engine_pair`（ZEN → 自製 DSL）: DSL 評価コンテキスト（`custom_node` / `code_node`）の API が変わる可能性がある。移行先は ZEN DSL の semantics を SoT として Rust で再実装するため、決定表フォーマット互換性は保証される

### 年次 dry-run 管理

- `dry_run_schedule.month: 1`（毎年 1 月に全 pair の dry-run を実施）
- `ship_blocker_days: 365`（dry-run から 365 日超過で CI fail）
- 各 migration_target の `last_dry_run` フィールドに実施日を記録。null = 未実施（1.0.0 ship 前必須）

## 関連参照
- [Library](../../03_概要設計/02_tier1設計方針/02_Library.md)
- [提供機能カテゴリ](../../03_概要設計/02_tier1設計方針/04_提供機能カテゴリ.md)
- [Bidi 適合仕様](01_Bidi適合仕様.md)
- [OSS ライフサイクル適合仕様](08_OSSライフサイクル適合仕様.md)
- [tier1 強制機構](../02_強制機構/01_tier1強制機構.md)
