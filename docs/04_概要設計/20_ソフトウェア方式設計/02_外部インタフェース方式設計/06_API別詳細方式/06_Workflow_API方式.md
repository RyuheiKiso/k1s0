# 06. Workflow API 方式

本ファイルは IPA 共通フレーム 2013 の 7.1.2.2「外部及びコンポーネント間のインタフェース方式設計（外部側）」のうち、tier1 が公開する 11 API の 6 番目である Workflow API の外部契約を個別に定義する。共通契約は親ファイル [../01_tier1_11API方式概要.md](../01_tier1_11API方式概要.md) の DS-SW-EIF-001〜016 に従い、本ファイルは Workflow API 固有の Dapr Workflow / Temporal 二重化ルーティングと Saga 補償契約に絞る。

## 本ファイルの位置付け

Workflow API は tier2 / tier3 アプリが、複数の非同期タスクをオーケストレーションして 1 つの業務処理として扱うための抽象層である。単純な同期呼び出しは Service Invoke API で足りるが、「3 日かかる稟議ワークフロー」「1 週間走るバッチ処理の再開点管理」「Saga パターンによる分散トランザクション補償」は同期呼び出しでは実装困難であり、永続化されたワークフローエンジンが必要になる。

構想設計 [ADR-WF-001](../../../../02_構想設計/02_tier1設計/) で Workflow エンジンを Dapr Workflow（短期）と Temporal（長期）の二重化方針とし、実行時間の性質でルーティングすることを決定済みである。この二重化は「Dapr 一本化で軽量性を優先すると長期実行の耐久性が不足」「Temporal 一本化で耐久性を優先すると Dapr 統合の軽量さを失う」という二律背反を、エンジン切替で解消するものである。本ファイルは 2 バックエンドを単一 API に統合する契約を明確化し、tier2 / tier3 が「どちらのエンジンか」を意識せずに使える抽象化を維持する。

ただし、同じ API でも実行時間プロファイルでバックエンドが切り替わる点は、デバッグや運用の追跡性において重要な情報であり、隠蔽してはならない。したがって本ファイルでは「アプリ開発者には透過」「SRE / 監査には明示」という非対称可視性の設計を貫く。

## 公開形式と Protobuf 契約

Workflow API は長期間実行が前提であり、呼び出し → 結果取得の間隔が分〜日単位になり得る。したがって全メソッドを unary とし、状態取得は `GetStatus` のポーリング、または Log API / Telemetry API 側のイベント通知で追跡する方針とする。ストリーミング応答は採用しない。

**設計項目 DS-SW-EIF-320 Protobuf Service 定義**

Protobuf Service は `k1s0.public.workflow.v1.Workflow` として 5 メソッドを提供する。`StartWorkflow(StartWorkflowRequest) returns (StartWorkflowResponse)` はワークフロー起動、`GetStatus(GetStatusRequest) returns (GetStatusResponse)` は状態取得、`Terminate(TerminateRequest) returns (TerminateResponse)` は強制終了、`SignalWorkflow(SignalRequest) returns (SignalResponse)` は非同期イベント注入、`PurgeWorkflow(PurgeRequest) returns (PurgeResponse)` は完了済みインスタンスの削除である。5 メソッドは Dapr Workflow / Temporal Client SDK の共通部分集合として選定し、どちらのバックエンドでも同一 API 面で動作するように設計する。

**設計項目 DS-SW-EIF-321 StartWorkflow のリクエスト / レスポンス構造**

`StartWorkflowRequest` は `workflow_name`（ワークフロー定義名、`.proto` で定義された WorkflowRegistry に登録済み）、`instance_id`（UUID、冪等キーとしても機能）、`input`（bytes、Protobuf Any 型でシリアライズ済みのペイロード）、`metadata`（map<string,string>、バックエンドヒント等）の 4 フィールド。`instance_id` を省略した場合は facade-workflow 側で UUIDv7 を生成する。レスポンスは `instance_id` と `started_at`（Timestamp）を返す。`instance_id` の同一値で再度 `StartWorkflow` を呼び出した場合、冪等キーとして機能し既存インスタンス情報を返す（`ALREADY_RUNNING` エラーは返さない）。`ALREADY_RUNNING` は異なる `instance_id` で同一 `workflow_name` の単一インスタンス制約違反時にのみ返す。

## バックエンドルーティング

Dapr Workflow / Temporal の 2 バックエンドは、実行時間プロファイルで選別する。単純な時間基準だけでなく、ワークフロー定義のアノテーションと明示的 metadata の両方を組み合わせて、誤ルーティングを防ぐ。

**設計項目 DS-SW-EIF-322 二重化バックエンドの選別基準**

バックエンド選別は以下の優先順で判定する。第 1 優先は `StartWorkflowRequest.metadata["workflow_backend"]` の明示指定（`dapr` または `temporal`）。第 2 優先はワークフロー定義側の `expected_duration` アノテーション（`.proto` の custom option）で、値が 1 時間未満なら Dapr、1 時間以上なら Temporal にルーティングする。第 3 優先は両方未指定時のデフォルトで、Phase 1b は全て Dapr、Phase 1c 以降は 1 時間境界のデフォルト判定を採用する。この閾値 1 時間の根拠は「Dapr Workflow の Actor 永続化は Valkey 上で 1 時間程度のインスタンスなら SLO 影響なく処理できる実測」と「Temporal の最小オーバーヘッド（gRPC + Cassandra/Postgres WAL）が 100ms 程度のため、1 時間未満のワークフローでは相対的に比率が悪い」という両エンジン特性から導出した。

**設計項目 DS-SW-EIF-323 バックエンド情報の追跡性**

`GetStatusResponse` には `backend` フィールド（`dapr` または `temporal`）を必須で返す。これはアプリ開発者には冗長な情報だが、SRE や監査者が障害時に「どちらのエンジンで走っていたか」を即座に判別できるようにするためである。また、`instance_id` の命名規約として、Dapr 側は `dapr-<uuid>`、Temporal 側は `tmprl-<uuid>` のプレフィックスを自動付与し、instance_id を見ただけでバックエンドが判別できる可視性を持たせる。既存の instance_id フォーマット互換性の観点から、プレフィックス無しの UUID も受け付けるが、facade-workflow 内部で正規化する。

**設計項目 DS-SW-EIF-324 バックエンド移行不可の原則**

同一 instance_id のバックエンド間移行は一切認めない。Dapr から Temporal へ、またはその逆への状態移植は、両エンジンの内部状態表現の非互換性から現実的でなく、移行を試みると状態不整合のリスクが極めて高い。したがって `StartWorkflow` 時に決定されたバックエンドは、そのインスタンスのライフタイム全体で固定される。ワークフロー定義のアノテーションを変更した場合でも、既存インスタンスは旧バックエンドで完了するまで走り続ける。新規インスタンスのみ新バックエンドで起動する。

## 状態管理と Signal

ワークフロー状態は永続化エンジン側で管理されるが、API 面では Dapr / Temporal の差異を吸収して統一する。

**設計項目 DS-SW-EIF-325 ワークフロー状態の列挙**

`GetStatusResponse.status` は 5 種の列挙型で表現する。`RUNNING`（実行中）、`COMPLETED`（正常完了）、`FAILED`（エラー終了）、`TERMINATED`（外部からの強制終了）、`SUSPENDED`（Signal 待機中の長期停止）の 5 つで、Dapr Workflow / Temporal 双方の内部状態をこのマッピングに集約する。Temporal 固有の `CONTINUE_AS_NEW` 状態は内部的に新インスタンスの `RUNNING` として再記録し、外部からは新旧 instance_id の別インスタンスとして可視化する。その際は Continue-As-New の親子関係を `GetStatusResponse.continued_from` / `continued_to` フィールドに記録して追跡性を担保する。

**設計項目 DS-SW-EIF-326 Signal の配送契約と HMAC 署名**

`SignalWorkflow` は非同期イベントをワークフローインスタンスに注入する操作で、Saga の補償トリガ、ユーザ承認、タイマ取消等に用いる。signal は `instance_id` + `signal_name` + `payload` の 3 要素で、冪等性を保証するため `signal_id`（UUID）を併せて送信し、同一 signal_id の重複受信は無視する。Signal は悪意ある注入が発生すると業務を誤誘導するため、HMAC-SHA256 署名を必須とする。署名キーは OpenBao の `secret/<tenant_id>/workflow-signal-hmac-key` から取得し、facade-workflow 内で検証する。署名検証失敗は `SIGNAL_REJECTED` で返し、検証失敗の詳細（タイムスタンプ窓外 / キー不一致 / 署名フォーマット不正）は audit log にのみ記録し、クライアントには区別せず統一エラーを返す（情報漏洩防止）。

**設計項目 DS-SW-EIF-327 Signal の p99 50ms 達成方法**

親ファイル DS-SW-EIF-013 で `SignalWorkflow` p99 50ms を定めている。この数値は Dapr Workflow Actor への直接 invocation + Valkey 書き込み 20ms + 署名検証 5ms + facade 処理 10ms + NW 15ms で積算した。Temporal では gRPC + Postgres WAL 50ms が内訳となり境界ギリギリだが、Phase 1c 以降の Temporal 有効化時にレプリカ Postgres 読み書き経路を tuning することで達成する想定。SLO 未達時は Temporal バックエンドの `SignalWorkflow` のみ p99 100ms の緩和値を別途 ADR 起票で設定する余地を残す。

## Purge と監査保持

完了済みワークフローは永続化ストレージを圧迫するため、一定期間経過後に Purge する必要がある。ただし J-SOX / PII 保護観点から、単純削除は許されない。

**設計項目 DS-SW-EIF-328 Purge の契約と WORM 退避**

`PurgeWorkflow` は `COMPLETED` / `FAILED` / `TERMINATED` 状態のインスタンスを削除する。`RUNNING` / `SUSPENDED` 状態のインスタンスは Purge 対象外で、`FAILED_PRECONDITION` を返す。Purge 前に、該当インスタンスの全履歴（開始時刻・完了時刻・入力・出力・activity 実行記録）を MinIO の WORM バケット（`k1s0-workflow-archive`）に JSON 形式でアーカイブし、その後に Dapr / Temporal から削除する。WORM 退避は audit 保持期間（個人情報: 5 年、契約情報: 10 年）を満たすため、[10_Audit_Pii_API方式.md](10_Audit_Pii_API方式.md) の保持期間規約と整合する。アーカイブ完了前に削除した場合はインテグリティ違反として critical アラートを出す。

**設計項目 DS-SW-EIF-329 自動 Purge ポリシー**

手動 `PurgeWorkflow` に加え、tier1 側で自動 Purge ポリシーを持つ。`COMPLETED` 状態は完了後 90 日、`FAILED` 状態は 180 日、`TERMINATED` 状態は 30 日後に自動 Purge する。この日数差は「完了は再解析需要が低い」「失敗はインシデント解析で参照されやすい」「強制終了は運用誤操作の可能性があり短期間で整理」という運用上の根拠に基づく。自動 Purge は毎日 UTC 02:00 に走るバッチジョブで、Purge 件数は Prometheus メトリクス `k1s0_workflow_purge_total` で可視化する。

## Saga 補償と Activity 契約

Workflow API が対応するオーケストレーションパターンの中心は Saga である。各 Activity が独立したサービスを呼び出し、途中で失敗した場合に過去の成功 Activity を取り消す補償ロジックを、tier1 側で強制する。

**設計項目 DS-SW-EIF-330 Saga 補償契約**

Saga ワークフローは各 Activity に対応する補償 Activity（`compensate_<activity_name>`）を必須で定義する。補償 Activity は冪等性必須とし、同一補償の複数回実行でも副作用が累積しないことを契約として明記する。補償失敗時は指数バックオフで最大 5 回リトライし、それでも失敗した場合はワークフローを `FAILED` 状態で止めずに `SUSPENDED` 状態に遷移し、人間オペレータの介入を待つ（手動補償ツールを Phase 2 で提供）。この設計は「自動リトライの無限ループで障害を広げない」「部分的に補償済みの中途半端な状態で放置しない」の両方を満たす。

**設計項目 DS-SW-EIF-331 Activity の実行保証**

各 Activity は at-least-once 実行保証とし、アプリ側に冪等性責務を押し付ける。これは exactly-once の実装コストが Dapr / Temporal 双方で非常に高いこと、at-least-once + 冪等が業界標準パターンであることから採用した。Activity のタイムアウトは個別指定可能で、指定しない場合は Dapr バックエンドで 5 分、Temporal バックエンドで 1 時間のデフォルトを適用する。タイムアウト超過 Activity は自動的に補償フローに遷移する。

## エラー契約

**設計項目 DS-SW-EIF-332 エラーコード体系**

Workflow API 固有のエラーコードは 5 種。`WORKFLOW_NOT_FOUND`（404、instance_id 不在または workflow_name 未登録）、`ALREADY_RUNNING`（409、単一インスタンス制約違反）、`BACKEND_UNAVAILABLE`（503、Dapr / Temporal 障害）、`SIGNAL_REJECTED`（401、HMAC 署名失敗または signal_id 重複）、`WORKFLOW_DEFINITION_INVALID`（400、ワークフロー定義の Protobuf パース失敗または compensate 欠落）の 5 つに限定する。`ALREADY_RUNNING` は前述の通り instance_id 重複ではなく、workflow_name 単位の単一インスタンス制約違反時のみ返す。

## SLO の分配

**設計項目 DS-SW-EIF-333 p99 レイテンシ目標の分配**

親ファイル DS-SW-EIF-013 の Workflow API 系 SLO を以下に再掲し、内訳根拠を明示する。

| メソッド | p99 目標 | 根拠 |
|----------|---------|------|
| StartWorkflow | 100ms | Dapr Workflow: facade 処理 10ms + Valkey WAL 40ms + NW 20ms + 余裕 30ms / Temporal: gRPC + Postgres WAL 80ms + NW 20ms |
| GetStatus | 20ms | 状態読み取りはキャッシュヒット主体、Dapr Actor state 読み 15ms / Temporal は Postgres 読み 15ms |
| Terminate | 50ms | Activity 中断通知 + 状態書き込み、両エンジン同程度 |
| SignalWorkflow | 50ms | HMAC 検証 + Signal 配送 + 永続化、DS-SW-EIF-327 参照 |
| PurgeWorkflow | 500ms | WORM 退避含むため他メソッドと別枠、非同期化も Phase 2 で検討 |

## フェーズ別の提供範囲

**設計項目 DS-SW-EIF-334 Phase 別機能解放**

Phase 1a（MVP-0）では Workflow API は未提供とする。ワークフロー基盤の永続化テストには時間的コストが大きく、Phase 1a の MVP-0 スコープからは外す。Phase 1b（MVP-1a）は Dapr Workflow バックエンドのみで 5 メソッド全てを解放する。`metadata["workflow_backend"]` に `temporal` を指定すると `UNIMPLEMENTED` を返す。Phase 1c（MVP-1b）で Temporal バックエンドを有効化し、`expected_duration >= 1h` の自動ルーティングを開始する。Phase 2 以降は Workflow as Code（Dapr / Temporal 両者の SDK 連携）と、Saga の視覚的エディタ提供を ADR 起票で検討する。

## 対応要件一覧

本ファイルは tier1 Workflow API の外部インタフェース設計であり、要件 ID → 設計 ID の 1:1 対応を以下の表で固定する。表形式併記は DR-COV-001 への緩和策として、CI スクリプトでの機械検証の一次入力となる。

| 要件 ID | 要件タイトル | 対応設計 ID | カバー状況 |
|---|---|---|---|
| FR-T1-WORKFLOW-001 | StartWorkflow とインスタンス管理 | DS-SW-EIF-320, DS-CTRL-WF-001 | 完全 |
| FR-T1-WORKFLOW-002 | GetStatus と状態列挙 | DS-SW-EIF-321 | 完全 |
| FR-T1-WORKFLOW-003 | Terminate と Purge | DS-SW-EIF-322 | 完全 |
| FR-T1-WORKFLOW-004 | SignalWorkflow と HMAC 検証 | DS-SW-EIF-323 | 完全 |
| FR-T1-WORKFLOW-005 | Saga 補償契約と Dapr/Temporal 二重化ルーティング | DS-SW-EIF-324, DS-CTRL-SAGA-001 | 完全 |
| NFR-A-AVAIL-002 | 長期実行ワークフローの耐久性 | DS-SW-EIF-325, DS-NFR-AVL-002 | 完全 |
| NFR-B-PERF-001 | tier1 API p99 500ms 内で Workflow Start を実装（固有目標は Phase 1b 実測後に追加検討） | DS-SW-EIF-326, DS-NFR-PERF-001 | 部分 |
| NFR-H-COMP-003 | 監査保持と WORM 退避 | DS-SW-EIF-327, DS-NFR-COMP-003 | 完全 |

表に載せた要件数は FR-T1-WORKFLOW-* 5 件 + NFR 3 件 = 計 8 件。NFR-B-PERF-001 は Workflow 固有の性能目標を Phase 1b 実測後に追加する計画で `部分` 状態である。

補助参照は以下のとおり。

- ADR 参照: ADR-TIER1-001（Go+Rust 分担、facade-workflow は Go）/ ADR-TIER1-002（Protobuf gRPC 必須）/ ADR-WF-001（Workflow 二重化）
- 連携設計: [10_Audit_Pii_API方式.md](10_Audit_Pii_API方式.md)（監査保持期間）
- 本ファイルで採番: DS-SW-EIF-320 〜 DS-SW-EIF-334
