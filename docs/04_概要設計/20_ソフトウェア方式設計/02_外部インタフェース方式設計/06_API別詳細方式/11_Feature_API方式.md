# 11. Feature API 方式

本ファイルは IPA 共通フレーム 2013 の 7.1.2.2「外部及びコンポーネント間のインタフェース方式設計（外部側）」に対応し、tier1 が tier2 / tier3 へ公開する `k1s0.public.feature.v1.Feature` サービスの外部インタフェース詳細を固定化する。共通契約は [../01_tier1_11API方式概要.md](../01_tier1_11API方式概要.md) DS-SW-EIF-001〜016 を参照とし、本ファイルは Feature API 固有の責務・メソッド・flagd バックエンド構成・対象化ルール・GitOps 変更フローを扱う。

## 本ファイルの位置付け

Feature API は業務ロジックに「機能の段階展開 / A/B テスト / テナント別機能差」を提供する価値抽象である。従来の JTC では新機能の本番投入が「全機能一斉リリース → 障害で全面ロールバック → 再テスト」のサイクルを繰り返し、リリース頻度が四半期 1 回まで低下する問題があった。Feature Flag は「コードをデプロイ済みでも機能を無効化できる」仕組みにより、デプロイとリリースを分離し、段階展開 / カナリア / 即時ロールバックを API 1 本で提供する。

tier1 は OpenFeature 標準（CNCF Graduated プロジェクト、2024 GA）を API の契約として採用し、バックエンド実装として [flagd](https://flagd.dev)（OpenFeature Reference Implementation）を Go 実装 facade-feature pod 内にライブラリリンクする。OpenFeature 標準を採用することで、将来的なバックエンド差し替え（LaunchDarkly / Unleash / Split.io 等の商用 SaaS への移行）をアプリ側コード変更なしで実施可能にする。本ファイルは OpenFeature + flagd の組み合わせを tier1 公開 API として露出する契約を固定する。

## サービス定義と公開メソッド

Feature API は Protobuf サービス `k1s0.public.feature.v1.Feature` として定義し、以下 4 メソッドを公開する。評価系（Evaluate / EvaluateAll）と参照系（ListFlags / GetFlag）で構成し、フラグ設定の更新は Git リポジトリ経由の GitOps に一本化する。

**設計項目 DS-SW-EIF-420 Feature サービスのメソッド粒度**

4 メソッドは `Evaluate`（単一フラグ評価）/ `EvaluateAll`（複数フラグ一括評価）/ `ListFlags`（フラグ一覧、管理 UI 用）/ `GetFlag`（特定フラグの定義取得）で構成する。フラグの作成 / 更新 / 削除メソッドは API に含めず、[../../40_制御方式設計/](../../40_制御方式設計/) で規定する GitOps フロー（Git に YAML PR → Argo CD → Postgres → flagd watcher 反映）に一本化する。これは、Feature Flag の変更が本番動作を直接変える性質上、レビュー / 承認 / 監査証跡を Git に集約することが運用安全性を最大化するためであり、API 経由のその場変更を禁じる設計判断である。商用 Feature Flag SaaS が多く提供する「管理 UI から直接フラグ変更」方式は JTC の変更管理プロセスと相性が悪く、明示的に採用しない。

## バックエンド構成（flagd）

flagd は OpenFeature の参照実装で、Cloud Native Computing Foundation 傘下の OSS である。tier1 は flagd を Go ライブラリとしてリンクし、facade-feature pod 内で評価を完結させる。

**設計項目 DS-SW-EIF-421 flagd インメモリ評価と Postgres 設定ストア**

facade-feature pod は起動時に Postgres `feature_flags` テーブルから全有効フラグ定義（JSON）をロードし、flagd の inmemory evaluator に投入する。Evaluate リクエストは flagd の `Resolve<Type>` 関数呼び出しでインメモリ評価を完結させ、Postgres への問い合わせは発生しない。この構成により p99 5ms（DS-SW-EIF-013）を達成する。Postgres 設定ストアは `(flag_key, version, flag_type, default_value, targeting_rules_json, enabled, updated_at, updated_by)` のスキーマで、1 flag あたり 1 行を保持する。flagd 自体を別 Pod として独立デプロイする構成（sidecar 方式）は採用せず、ライブラリリンクで Pod 数を削減する。これは flagd の評価ロジックが純粋関数でありサイドカー分離の必要性が低く、ネットワークホップを 1 段減らすレイテンシ優位性を重視した選択である。

**設計項目 DS-SW-EIF-422 Postgres LISTEN/NOTIFY による即時反映**

Postgres `feature_flags` テーブルの UPDATE は、`AFTER UPDATE` トリガで `pg_notify('feature_flag_changed', flag_key)` を発行する。facade-feature pod は Postgres への LISTEN 接続を常時保持し、通知受信後に該当 flag_key のみ Postgres から再ロードして flagd inmemory evaluator を部分更新する。反映 SLO は変更 commit 完了から 10 秒以内（p99）とし、GitOps の Argo CD sync 間隔（3 分）と合わせて「Git merge 後、3 分 10 秒で本番反映」の時系列を確立する。pod 間の反映タイミング差（3 レプリカ間の不整合）は最大 10 秒発生する可能性があるが、Feature Flag の性質上（厳密整合が不要な設定値）許容する。厳密な同時切り替えが必要な機能は Feature API の scope 外とし、代わりに blue-green デプロイ等の別手法に誘導する。

## フラグ型と評価契約

OpenFeature は 5 つのフラグ型を定義する。tier1 は全型を受け入れつつ、Phase 別に提供範囲を段階拡張する。

**設計項目 DS-SW-EIF-423 フラグ型 5 種の定義と用途**

サポート型は `Boolean`（ON/OFF 切り替え、最多用途）/ `String`（設定値、例 `"blue"` / `"green"`）/ `Integer`（閾値、例 `timeout_ms = 500`）/ `Double`（比率、例 `discount_rate = 0.15`）/ `Object`（JSON、複雑な設定オブジェクト）の 5 種である。`Object` 型は JSON Struct として受け入れ、アプリ側で Protobuf 型にデシリアライズする運用を前提とする。型不一致（Boolean フラグを String として評価）は `INVALID_CONTEXT` エラーで拒否する。型の設計方針は「アプリ側でコード分岐する値は Boolean、設定値は String / Integer / Double、構造化設定は Object」と用途別に推奨し、[../../50_開発者体験/](../../../03_要件定義/50_開発者体験/) のゴールデンパス文書で具体例を提示する。

**設計項目 DS-SW-EIF-424 Evaluate メソッドの入出力契約**

`EvaluateRequest` は `flag_key`（string、必須）/ `flag_type`（enum: BOOLEAN / STRING / INTEGER / DOUBLE / OBJECT）/ `default_value`（`google.protobuf.Value`、フラグ未定義時の安全値）/ `context`（`EvaluationContext` メッセージ）の 4 フィールド。`EvaluationContext` は OpenFeature 仕様準拠で `targeting_key`（string、通常は user_id）/ `tenant_id`（string、JWT から自動注入）/ `attributes`（map<string, Value>、ユーザ属性 / セッション属性）の 3 フィールドを含む。`EvaluateResponse` は `value`（`google.protobuf.Value`、評価結果）/ `variant`（string、当該 variant 名、例 `"control"` / `"treatment"`）/ `reason`（enum: STATIC / TARGETING_MATCH / SPLIT / DEFAULT / DISABLED / ERROR）/ `flag_metadata`（map、追加メタデータ）の 4 フィールドで返す。`default_value` の必須化は「フラグが存在しない / 評価失敗時に安全側の値を返す」ゼロダウンタイム設計の中核である。

**設計項目 DS-SW-EIF-425 EvaluateAll による複数フラグ一括評価**

`EvaluateAll` は最大 50 フラグを単一 RPC で評価する。リクエストは `flag_specs[]`（各要素は flag_key / flag_type / default_value）と `context`（1 回のみ指定、全フラグで共通）、レスポンスは `results[]`（各要素は flag_key / value / variant / reason）を返す。部分失敗（50 フラグ中 2 件エラー）は 200 OK で返し、エラーフラグは default_value にフォールバックする。50 フラグ上限は画面 1 つで参照する Feature Flag 数の典型値（10〜30）に対して 1.5 倍の余裕を持たせ、それ以上の一括評価は設計見直しを促す境界として決定した。

## 対象化ルール（Targeting）

Feature Flag の価値は「全ユーザ一律 ON/OFF」ではなく「特定条件のユーザにのみ ON」の対象化（targeting）にある。tier1 は flagd の JsonLogic 拡張を採用する。

**設計項目 DS-SW-EIF-426 Targeting Rule 構文と評価順**

Targeting Rule は flagd 準拠の JsonLogic 構文で記述する。代表的なルールは (1) テナントマッチ `{"==": [{"var": "tenant_id"}, "tenant-abc"]}`、(2) パーセント rollout `{"fractional": [{"var": "targeting_key"}, ["treatment", 10], ["control", 90]]}`（10% に treatment）、(3) 属性マッチ `{"in": [{"var": "attributes.region"}, ["JP", "KR"]]}`、(4) 複合条件 `{"and": [rule1, rule2]}` の 4 パターンである。評価順は (a) flag が `enabled = false` なら即 default、(b) targeting rules を上から順に評価し最初のマッチで確定、(c) 全てマッチしなければ `default_variant` を返す。パーセント rollout の `targeting_key` による一貫性（同一ユーザは常に同一 variant）は flagd が SHA-1 ハッシュで保証し、rollout 比率を変更しても既存 variant 割当は極力維持される（stable hashing）。

**設計項目 DS-SW-EIF-427 テナント別 override とマルチテナント分離**

マルチテナント環境では「テナント A は Phase 1、テナント B は Phase 2 機能」のような個別制御が必要である。tier1 は flagd の `targeting` 配列を `tenant_id` で最優先マッチさせる規約を採用し、テナント固有設定をグローバル設定より優先する評価順を強制する。実装上は、全フラグの targeting rules の先頭に `tenant_id` マッチ条件を自動挿入する Postgres VIEW を定義し、フラグ作成者がテナント分離を忘れても構造的に漏れない仕組みとする。テナント別 override の監査は Audit-Pii API へ自動連携し、「いつ誰がどのテナントに何を有効化したか」を 7 年保管する。

## GitOps による設定変更

Feature Flag の変更はそれ自体が「本番動作変更」であり、コードデプロイと同等の変更管理を適用する。

**設計項目 DS-SW-EIF-428 GitOps フロー定義**

フラグ定義は Git リポジトリ `k1s0-feature-flags`（protected branch: main、PR レビュー 2 名必須）の `flags/<flag_key>.yaml` として管理する。YAML は flagd 準拠スキーマ（flag_key / flag_type / default_variant / variants / targeting_rules / metadata）。Git merge 後の反映は (1) Argo CD が 3 分間隔で Git ポーリング、(2) 変更検出で Kubernetes `FeatureFlag` CR（CRD は tier1 定義）を apply、(3) Operator（tier1 facade-feature-operator pod）が CR を Postgres `feature_flags` テーブルに反映、(4) Postgres LISTEN/NOTIFY で facade-feature pod が inmemory evaluator を更新、という 4 段階で進行する。全段階のタイムスタンプとアクターは Audit-Pii API に自動記録する。緊急の無効化（インシデント時の kill switch）は「PR レビュー 1 名で即 merge 可能な emergency label 付き変更」を例外運用として定義し、merge から 10 秒以内の反映を保証する。

**設計項目 DS-SW-EIF-429 変更の Audit-Pii 自動連携**

フラグ変更は facade-feature-operator から Audit-Pii API `RecordEvent` へ自動連携する。記録内容は `actor`（Git commit author、CI/CD OIDC token で認証）/ `action = "feature.flag.update"` / `resource = flag_key` / `outcome`（SUCCESS/FAILURE）/ `context`（git_commit_sha / old_value / new_value / pr_url）。過去の JTC 事例で「Feature Flag 変更による本番障害の原因特定に 3 日かかる」事象があり、Audit-Pii への自動記録は障害時の root cause analysis を 30 分に短縮する設計意図を持つ。

## OpenFeature SDK 互換

tier2 / tier3 は OpenFeature 公式 SDK を使う経路と、tier1 生成 SDK を使う経路の 2 方式で統合可能にする。

**設計項目 DS-SW-EIF-430 OpenFeature SDK 互換 Provider**

tier1 は OpenFeature の `Provider` インタフェースを実装した `k1s0-openfeature-provider-<lang>` を Go / Rust / TypeScript / C# の 4 言語で提供する。tier2 / tier3 アプリは OpenFeature 公式 SDK（`@openfeature/server-sdk` 等）を使い、Provider 初期化時に tier1 Provider を指定する構成で、フラグ評価コードは標準 OpenFeature API（`client.getBooleanValue('my-flag', false, context)`）のまま tier1 バックエンドに接続する。この構成により、将来的に商用 SaaS へバックエンド変更する場合も Provider 差し替えのみでアプリ側コード変更を不要とする。tier1 独自生成 SDK（`protoc-gen-<lang>`）も並行提供するが、OpenFeature Provider の採用を第一推奨とし、将来の移植性を重視する。

## A/B テストと段階展開

新機能の検証は「全社一斉投入」ではなく「段階的な比率拡大」で進めることが本番事故の発生確率を桁違いに下げる。tier1 は標準的な 4 段階展開パターンを推奨として定義する。

**設計項目 DS-SW-EIF-431 段階展開 0% → 1% → 10% → 100% の運用標準**

新機能の本番展開は以下 4 段階を標準とする: (1) 0%（デプロイのみ、機能 OFF、コードは本番に存在）、(2) 1%（社内 / パイロットテナント限定、targeting_key で抽出）、(3) 10%（パーセント rollout、問題発生時の影響範囲を限定）、(4) 100%（全面展開、一定期間の観測後に Feature Flag 削除）。各段階の滞留時間は機能重要度に応じて 1 日〜2 週間とし、SRE ダッシュボード（Grafana）で該当機能の SLO 指標（エラー率 / レイテンシ / ビジネス KPI）を可視化する。段階拡大の判断基準（例: エラー率 < 0.1% 維持 24 時間）は [../../../03_要件定義/40_運用ライフサイクル/](../../../03_要件定義/40_運用ライフサイクル/) の「段階展開ガイドライン」で規定する。全段階の Feature Flag 変更は Audit-Pii に記録され、事後の原因追跡を可能にする。

**設計項目 DS-SW-EIF-432 A/B テスト機能（Phase 1c 以降）**

A/B テストは `Evaluate` の `variant` + `reason = SPLIT` 返却を使用する。tier1 は fractional rollout（`targeting_key` 基準の stable hashing）で variant 割当を安定させる。A/B テスト結果の集計は tier1 の責務ではなく、tier2 / tier3 アプリが Telemetry API 経由で variant 属性付きメトリクス（`feature_<key>_variant = "treatment"` 等）を送信し、Grafana / BigQuery 側で variant 別 KPI 集計する。これは「Feature Flag は評価のみを担当、実験効果測定はアプリ + 観測性基盤」という責務分離で tier1 の設計複雑度を抑える判断である。Phase 1a / 1b では variant の固定割当のみ、Phase 1c から fractional rollout と A/B テスト機能を有効化する。

## エラーコード

Feature API 固有のエラーは K1s0Error の `code` フィールドに以下 4 種を登録する。

**設計項目 DS-SW-EIF-433 Feature 固有エラーコード**

| コード | gRPC status | 発生条件 | 根拠 |
|--------|-------------|----------|------|
| `FLAG_NOT_FOUND` | NOT_FOUND | 指定 flag_key が未登録 | default_value にフォールバックする契約 |
| `INVALID_CONTEXT` | INVALID_ARGUMENT | context の型不一致 / targeting_key 欠落 | OpenFeature 仕様準拠 |
| `EVALUATION_FAILED` | INTERNAL | targeting rule 評価中の例外 | JsonLogic パース失敗等の予期せぬ事象 |
| `FLAG_DISABLED` | (200 OK で reason=DISABLED) | enabled = false | エラーではなく運用状態、default_value を返す |

`FLAG_NOT_FOUND` はエラーとしつつクライアント SDK では default_value を返すラップ処理を推奨し、アプリ側でフラグ未定義時も動作継続可能にする設計を徹底する。

## フェーズ別提供範囲

**設計項目 DS-SW-EIF-434 フェーズ別提供範囲**

Phase 1a（MVP-0）: `Evaluate` のみ、Boolean 型のみ、targeting は tenant_id 一致判定のみ（パーセント rollout 未対応）、default_value 必須。Phase 1b（MVP-1a）: 全 5 型対応、全 targeting rule（tenant / percent / attribute match）対応、`EvaluateAll` / `ListFlags` / `GetFlag` 追加、GitOps フロー完備。Phase 1c（MVP-1b）: A/B テスト（fractional rollout）、段階展開 4 段運用標準の完全実装、emergency kill switch フロー。Phase 2: 機械学習ベース targeting（ユーザ属性からの自動セグメンテーション）、Feature Flag 依存グラフ可視化、廃止予定フラグ（technical debt flag）の自動検出。

## 対応要件一覧

本ファイルは Feature API 公開インタフェースの詳細方式設計であり、以下の要件 ID に対応する。

- FR-T1-FEATURE-001〜FR-T1-FEATURE-004（Feature API 機能要件、評価 / 段階ロールアウト / circuit breaker / A/B テスト）
- FR-T1-FEATURE-001（フラグ評価と 5 型対応）/ FR-T1-FEATURE-002（段階ロールアウト %）/ FR-T1-FEATURE-003（circuit breaker ルール）/ FR-T1-FEATURE-004（A/B テスト / fractional rollout）
- NFR-B-PERF-001（Feature p99 5ms インメモリ評価）
- NFR-C-OPS-002（設定変更のトレーサビリティ、Audit-Pii 自動連携）
- NFR-C-OPS-003（緊急時 kill switch 10 秒以内反映）
- NFR-D-MIG-003（段階展開による移行リスク最小化）
- NFR-H-COMP-002（設定変更の監査証跡、7 年保管）
- ADR 参照: ADR-TIER1-001（Go+Rust 分担、Feature は Go）/ ADR-TIER1-002（Protobuf gRPC 必須）/ ADR-FEAT-001（flagd / OpenFeature 採用、未起票だが Phase 1a 前に起票予定）
- 共通契約: DS-SW-EIF-001〜016（[../01_tier1_11API方式概要.md](../01_tier1_11API方式概要.md)）
- 本ファイルで採番: DS-SW-EIF-420 〜 DS-SW-EIF-434
