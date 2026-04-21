# 09. Decision API 方式

本ファイルは IPA 共通フレーム 2013 の 7.1.2.2「外部及びコンポーネント間のインタフェース方式設計（外部側）」に対応し、tier1 が tier2 / tier3 へ公開する `k1s0.public.decision.v1.Decision` サービスの外部インタフェース詳細を固定化する。共通契約は [../01_tier1_11API方式概要.md](../01_tier1_11API方式概要.md) DS-SW-EIF-001〜016 を参照とし、本ファイルは Decision API 固有の責務・メソッド・JDM モデル管理・署名検証・低レイテンシ保証を扱う。

## 本ファイルの位置付け

Decision API は業務ロジックが「ルール / ポリシーによる判定」を tier1 に委譲する価値抽象である。与信限度・割引適用・特権承認などの判定を各アプリでハードコードすると、ルール変更時のリリース多発・重複実装・齟齬という典型的な JTC の痛点が発生する。tier1 は [ZEN Engine](https://gorules.io) を Rust 実装 custom-decision pod にライブラリリンクで内包し、JDM（JSON Decision Model）準拠の宣言的ルールをプロセス内 JIT コンパイルで評価する構成で、p99 1ms の低レイテンシを実現する。

この設計は [ADR-RULE-001] で確定した。OSS の決定エンジン候補（OPA Rego / DMN Camunda / Drools）の比較検討の結果、ZEN Engine を採用した理由は (1) MPL-2.0 ライセンスで商用利用に制約なし、(2) Rust ネイティブで tier1 の Rust 自作領域との親和性、(3) JDM のビジュアルエディタ提供、(4) ベンチマークで p99 100µs を記録（tier1 の 1ms SLO に対して 10 倍マージン）の 4 点である。本ファイルは ZEN Engine を tier1 公開 API として安全に露出するための契約を固定化する。

## サービス定義と公開メソッド

Decision API は Protobuf サービス `k1s0.public.decision.v1.Decision` として定義し、以下 5 メソッドを公開する。評価系（Evaluate / EvaluateBatch）と管理系（DeployPolicy / ListPolicies / GetPolicy）を分離し、評価系は low-latency 経路、管理系は GitOps 経路の 2 系統で運用する。

**設計項目 DS-SW-EIF-380 Decision サービスのメソッド粒度**

5 メソッドの内訳は `Evaluate`（単一入力評価）/ `EvaluateBatch`（最大 100 入力の並列評価）/ `DeployPolicy`（ポリシー登録・更新）/ `ListPolicies`（ポリシー一覧、管理 UI 用）/ `GetPolicy`（特定ポリシーの JDM 取得、監査用）である。評価系 2 つは tier2 / tier3 のビジネスクリティカル経路から呼ばれ、p99 1ms / 5ms の SLO を守る。管理系 3 つは CI/CD パイプライン（Argo CD）から呼ばれ、人間のポリシー編集フローに属する。評価系と管理系を別サービスに分けず単一サービスに統合する選択は、IAM 権限（`decision.evaluate` vs `decision.deploy`）をメソッド単位で分離することで達成し、tier2 / tier3 アプリに `decision.deploy` を誤付与する事故を抑止する。

## JDM（JSON Decision Model）準拠

JDM は GoRules 社が提唱する宣言的ルール記述形式で、Decision Table / If-Else / Expression / Function の 4 ノードを有向グラフで接続する。OSS 標準として 2026 年時点で JDM Spec 1.0 が確定し、ZEN Engine が参照実装である。

**設計項目 DS-SW-EIF-381 JDM モデルフォーマットの固定化**

ポリシー定義は JDM 1.0 準拠の JSON を正とする。JDM は `nodes[]`（ノード配列、各ノードは id / name / type / content フィールドを持つ）と `edges[]`（ノード間の有向接続）の 2 配列を root に持ち、`inputNode` から始まり `outputNode` で終わる有向非巡回グラフ（DAG）として記述される。tier1 はノード type として `decisionTable`（決定表）/ `expression`（式）/ `function`（JS 関数）/ `switch`（分岐）の 4 種を受け付ける。`function` ノードは ZEN Engine 内蔵の QuickJS で実行されるが、ネットワーク I/O / ファイル I/O / 無限ループを検出するサンドボックス制約を [DS-SW-EIF-387] で付与する。JDM スキーマは `.proto` の `bytes jdm_content` として受け入れ、tier1 側で JSON Schema バリデーションを行う。

## リクエスト / レスポンス契約

評価系の最主要メソッド `Evaluate` は業務処理のクリティカルパスに入るため、リクエスト / レスポンスの契約を厳密に固定する。

**設計項目 DS-SW-EIF-382 Evaluate メソッドの入出力契約**

`EvaluateRequest` は `policy_id`（string、必須、`<name>@<version>` 形式）/ `input`（`google.protobuf.Struct`、JDM 入力データ）/ `context`（map<string, string>、トレース / テナント情報を評価ログに添付）/ `audit_flag`（bool、true で Audit-Pii API に自動連携）の 4 フィールドで構成する。`EvaluateResponse` は `output`（`google.protobuf.Struct`、JDM 出力データ）/ `decision_id`（UUIDv7、監査トレイル参照キー）/ `trace`（ZEN Engine の評価トレース、デバッグ時のみ充填）/ `evaluation_ms`（評価所要時間）の 4 フィールドで返す。`output` の構造は JDM の `outputNode` 定義に依存するため Protobuf としては Struct で緩く型付けし、SDK 側で型付きラッパを生成する運用は tier2 / tier3 各言語で提供する。

**設計項目 DS-SW-EIF-383 EvaluateBatch の並列評価と部分失敗**

`EvaluateBatch` は最大 100 入力の並列評価を単一 RPC で提供する。tier1 内部では Rust の tokio runtime で並列タスクとして実行し、各入力の評価結果を独立して返す。部分失敗（100 入力中 3 件が `EVALUATION_TIMEOUT`）は 200 OK で返し、`BatchEvaluateResponse.results[]` の各要素に `result` または `error` を設定する all-or-nothing ではない設計である。100 入力の上限は ZEN Engine の並列度上限（16 並列）× 平均評価時間 100µs × 余裕 6 倍 = 9.6ms の逆算から p99 10ms を確保する値として決定した。上限超過時は `INVALID_ARGUMENT` を返す。

## ポリシー管理とバージョニング

ポリシーは業務ルールそのものであり、金額・対象顧客・判定ロジックの誤りは直接的な財務損失に繋がる。変更・ロールバック・段階展開を安全に行うためにバージョニングと署名検証を義務化する。

**設計項目 DS-SW-EIF-384 policy_id のバージョン suffix 必須化**

`policy_id` は `<name>@<version>` 形式を必須とする。version は SemVer（`1.0.0`）または Calendar Versioning（`2026.04`）のどちらかを選択可能で、ポリシー単位で固定する。新旧バージョンの並行提供を前提とし、tier2 / tier3 アプリは明示的に `policy_id = "credit-check@1.2.0"` を指定して評価する。バージョン省略時の latest 解決は提供しない。理由は、アプリ側が意図しない新ルールで判定されることによる本番事故（過去 JTC 事例で頻発）を構造的に防ぐためである。段階展開は Feature API と連携して `if feature.enabled("credit-check-v2") then "credit-check@2.0.0" else "credit-check@1.2.0"` のパターンで実現する。

**設計項目 DS-SW-EIF-385 Sigstore cosign によるポリシー署名検証**

`DeployPolicy` は `policy_id` / `jdm_content`（bytes）/ `signature`（bytes、Sigstore cosign 署名）/ `version` / `deployer_identity` の 5 フィールドを受ける。tier1 は受領した JDM バイト列の cosign 署名を Sigstore Fulcio 発行の x509 証明書チェーンで検証し、検証失敗時は `SIGNATURE_VERIFICATION_FAILED` を返し登録を拒否する。署名は CI/CD パイプライン（GitHub Actions OIDC → Sigstore）で自動付与され、人間が手動で JDM を編集して直接 API 呼び出しする経路を遮断する。この署名フローは [../../../02_構想設計/04_CICDと配信/](../../../02_構想設計/04_CICDと配信/) の Keyless Signing 方針と連動する。

**設計項目 DS-SW-EIF-386 ListPolicies / GetPolicy の監査用途**

`ListPolicies` は tenant_id スコープでポリシー一覧（policy_id / version / deployed_at / deployer）を返す。`GetPolicy` は特定 policy_id の JDM バイト列と署名・デプロイ履歴を返す。両者は `decision.read` IAM 権限を持つ監査担当者 / CI/CD ボット / 管理 UI からのみ呼び出し可能で、評価系（`decision.evaluate`）とは権限分離する。Postgres ポリシーストア（tier1 Node Pool 内 CloudNativePG Cluster）を直接参照せず必ず API 経由とすることで、監査ログを Audit-Pii API に自動記録する経路を確保する。

## サンドボックスと実行時制約

JDM の `function` ノードは JavaScript を実行可能なため、悪意あるまたは不注意な JS コードが tier1 の可用性を損なう可能性がある。実行時制約を API レベルで固定する。

**設計項目 DS-SW-EIF-387 QuickJS サンドボックス制約**

`function` ノード内 JavaScript は ZEN Engine 内蔵の QuickJS エンジンで実行され、以下の制約を強制する: (1) 実行時間 50ms 上限（超過で `EVALUATION_TIMEOUT`）、(2) メモリ 16MB 上限、(3) 再帰深さ 128 上限、(4) ネットワーク I/O / ファイル I/O / `eval` / `Function` コンストラクタ禁止、(5) `setTimeout` / `setInterval` 禁止。これらは QuickJS の Realm 分離とカスタムホストフックで実装する。制約違反は JDM の static analyzer（DeployPolicy 時）と runtime enforcer（Evaluate 時）の 2 段で検出し、前者は登録拒否、後者はエラー返却 + Audit-Pii への記録を行う。

## 低レイテンシ保証（p99 1ms）

Decision API の p99 1ms（DS-SW-EIF-013）は tier1 最速の SLO であり、プロセス内ライブラリリンクでのみ達成可能である。

**設計項目 DS-SW-EIF-388 Rust プロセス内 JIT 評価の実装戦略**

tier1 custom-decision pod は Rust で実装され、ZEN Engine を `zen-engine` crate としてライブラリリンクする。pod 起動時に Postgres から全有効ポリシーをロードし、JDM を内部中間表現にコンパイル（JIT）してメモリ常駐させる。Evaluate 時はリクエスト受信 → 中間表現ルックアップ → tokio thread-per-core での評価 → レスポンス返却が全て同一プロセス内で完結し、ネットワーク I/O を一切含まない。ZEN Engine 公式ベンチマーク（[gorules/zen](https://github.com/gorules/zen)）では 1 evaluation あたり 100µs（複雑度中程度、決定表 100 行）を記録しており、tier1 1ms 目標に対して 10 倍のマージンを持つ。ポリシー更新時は差分ロード（新 policy_id を追加 / 旧 version は残置）で無停止反映する。

**設計項目 DS-SW-EIF-389 NUMA 最適化と CPU affinity**

custom-decision pod は 4 vCPU / 8GB RAM の構成で、K8s `topologyManager` policy を `single-numa-node` に設定して NUMA ノードローカルメモリのみ使用させる。Rust の tokio runtime は `worker_threads=4` + `thread_per_core` モードで起動し、CPU affinity を pinning する。この最適化により L3 キャッシュミス率が 30% 減少し、p99 100µs → 70µs（ベンチマーク内部計測）の改善を確認済み。Phase 1b 時点では single-NUMA 構成で足り、マルチ NUMA 構成（Phase 2 の高負荷期）では per-NUMA pod でシャーディングする方針を [ADR-ZEN-002]（未起票、Phase 2 時点で起票予定）として保留する。

**設計項目 DS-SW-EIF-390 MPL-2.0 ライセンス制約とコンプライアンス**

ZEN Engine は MPL-2.0（Mozilla Public License 2.0）で配布されており、ライブラリリンクによる tier1 Rust バイナリへの組み込みはライセンス条項上問題ない。MPL-2.0 は「MPL 下のファイル単位」のコピーレフト条項であり、ZEN Engine のソースファイルを改変しない限り、tier1 の Rust コードは MPL に汚染されない（ファイルレベル copyleft）。tier1 の custom-decision Pod バイナリは「ZEN Engine オリジナルファイル + tier1 独自ファイル」の組み合わせであり、ZEN Engine オリジナルファイルのソース公開義務のみ負う。この義務はベンダ組込ではなく OSS そのものを流用するため実質的な追加負担なしで充足する。詳細は [../../../02_構想設計/05_法務とコンプライアンス/](../../../02_構想設計/05_法務とコンプライアンス/) の OSS ライセンス審査結果に従う。

## 監査トレイル連携

業務判定結果はその後の業務処理（決済実行・承認通知・契約生成）の根拠となるため、事後検証のための監査トレイルが必須である。tier1 は Decision の結果を Audit-Pii API に自動連携する仕組みを提供する。

**設計項目 DS-SW-EIF-391 audit_flag による Audit-Pii API 自動連携**

`EvaluateRequest.audit_flag = true` の場合、tier1 は評価完了後に Audit-Pii API の `RecordEvent` を非同期呼び出しし、以下フィールドを記録する: `actor`（JWT 主体）/ `action = "decision.evaluate"` / `resource = policy_id` / `outcome`（評価結果のサマリ）/ `context`（input ハッシュ / output ハッシュ / decision_id / evaluation_ms）/ `pii_classification`（input に PII 含有の有無）。連携失敗時は Decision 結果を成功として返しつつ、別途 DLQ（Kafka トピック `decision-audit-dlq`）に記録して後追い可能とする。audit_flag = false の場合は連携しないが、エラー発生時（`EVALUATION_TIMEOUT` / `POLICY_NOT_FOUND`）は audit_flag に関わらず必ず記録する。

## エラーコード

Decision API 固有のエラーは K1s0Error の `code` フィールドに以下 6 種を登録する。

**設計項目 DS-SW-EIF-392 Decision 固有エラーコード**

| コード | gRPC status | 発生条件 | 根拠 |
|--------|-------------|----------|------|
| `POLICY_NOT_FOUND` | NOT_FOUND | 指定 policy_id / version が未登録 | バージョン指定必須（DS-SW-EIF-384） |
| `INVALID_INPUT` | INVALID_ARGUMENT | input が JDM 入力スキーマに非適合 | JDM 静的型チェック |
| `COMPILATION_FAILED` | FAILED_PRECONDITION | DeployPolicy 時の JDM パース失敗 | デプロイ前の妥当性確認 |
| `SIGNATURE_VERIFICATION_FAILED` | PERMISSION_DENIED | cosign 署名検証失敗 | DS-SW-EIF-385 署名必須 |
| `EVALUATION_TIMEOUT` | DEADLINE_EXCEEDED | function ノード 50ms 超過 | DS-SW-EIF-387 サンドボックス制約 |
| `SANDBOX_VIOLATION` | PERMISSION_DENIED | function ノードが禁止 API 呼び出し | 同上 |

## フェーズ別提供範囲

**設計項目 DS-SW-EIF-393 フェーズ別提供範囲**

Phase 1a（MVP-0）: 提供なし。Phase 1b（MVP-1a）: `Evaluate` / `DeployPolicy` / `ListPolicies` / `GetPolicy` の 4 メソッド、単一評価のみ、JDM ノード type は `decisionTable` / `expression` / `switch` の 3 種（`function` ノードは未対応）。Phase 1c（MVP-1b）: `EvaluateBatch` 追加、`function` ノード（QuickJS サンドボックス付き）追加、段階展開 Feature API 連携。Phase 2: マルチテナント向けポリシー共有機能（テナント間で共通ポリシーを参照する仕組み、ただしテナント分離は維持）、NUMA シャーディング検討。

## 対応要件一覧

本ファイルは Decision API 公開インタフェースの詳細方式設計であり、以下の要件 ID に対応する。

- FR-T1-DECISION-001〜FR-T1-DECISION-004（Decision API 機能要件、JDM 評価 / バージョン管理 / Audit 連携 / ホットリロード）
- FR-T1-DECISION-001（JDM 決定表評価）/ FR-T1-DECISION-002（決定表バージョン管理）/ FR-T1-DECISION-003（評価履歴 Audit 連携）/ FR-T1-DECISION-004（決定表ホットリロード）
- NFR-B-PERF-005（Decision p99 1ms 低レイテンシ）
- NFR-H-INT-004（ポリシー改ざん防止、cosign 署名）
- NFR-H-COMP-002（判定結果の監査証跡、Audit-Pii 連携）
- NFR-D-MIG-003（ポリシー変更の段階展開）
- ADR 参照: ADR-TIER1-001（Go+Rust 分担、Decision は Rust）/ ADR-TIER1-002（Protobuf gRPC 必須）/ ADR-RULE-001（ZEN Engine 採用）
- 共通契約: DS-SW-EIF-001〜016（[../01_tier1_11API方式概要.md](../01_tier1_11API方式概要.md)）
- 本ファイルで採番: DS-SW-EIF-380 〜 DS-SW-EIF-393
