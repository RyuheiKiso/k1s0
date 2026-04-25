# Decision API

本書は、tier1 が公開する Decision API の機能要件を定義する。ビジネスルール・意思決定ロジックを JSON Decision Model（JDM）で宣言的に記述し、ZEN Engine（Rust in-process）で評価する。

## API 概要

業務アプリの分岐条件（承認者決定、権限判定、割引計算、アラート閾値等）を、if/else のハードコードではなく決定表として外部化する。tier2 のアプリコード変更なしに、業務担当が決定表を更新することでルールを変更できる将来像を目指す（採用側の全社展開期 の JDM エディタ Backstage プラグイン化）。

内部実装は tier1 Rust 自作領域に ZEN Engine を in-process 組込し、評価レイテンシをマイクロ秒オーダで維持する。

## 機能要件

### FR-T1-DECISION-001: JDM 決定表評価

**業務根拠**: BR-PLATUSE-005（業務ルール変更のリードタイム短縮と外部化による保守性向上）。

**現状**: tier2 のアプリで業務ルール（例: 部門 × 金額による承認者決定）を if/else でハードコードすると、ルール変更の都度アプリ改修・テスト・リリースが発生する。情シスが稟議ルールを変更したくても、tier2 開発チームの工数が空かないと反映できない。社内既存システムでは、業務ルール変更 1 件あたり「仕様確定 2 週 → 開発 1 週 → テスト 1 週 → リリース待ち 1 週」で平均リードタイム 5 週間を要している。年間 40 件のルール変更で 200 週分の業務部門待機時間が発生し、決算処理・規制対応の遅延が頻発している。

**要件達成後**: `k1s0.Decision.Evaluate("decision-table-name", input)` で JDM 決定表を評価する。決定表は Git リポジトリで管理され、Argo CD で自動デプロイ。tier2 アプリコードは評価 API 呼び出しのみ、ルール変更は JDM ファイル更新で完結。リードタイムは 5 週間 → 1 日に短縮（JDM PR レビュー 0.5 日 + Argo CD 同期 1 時間）。採用側の全社展開期 の Backstage JDM エディタ実装後は、業務担当者自身が編集可能となり、40 件/年 × 5 週 = 200 週分の業務部門待機時間が解消される。

**崩れた時**: 業務ロジック（業務システムの稟議等）の変更のリードタイムが長期化し、業務部門から情シスへの不満が蓄積する。tier2 アプリに業務ロジックが埋め込まれ、横展開できなくなる。規制対応の遅延は法令違反リスクにも直結し、金融庁報告義務・税制改正対応で「システムが間に合わない」事態が発生する。

**動作要件**:
- JDM v1 準拠の決定表を評価可能
- 決定表未存在時は `K1s0Error.DecisionNotFound` を返す
- 評価結果の JSON スキーマは tier2 側で事前定義可能

**品質基準**:
- 評価レイテンシは NFR-B-PERF-004（p99 < 1ms、in-process 実行、マイクロ秒オーダ目標）に従う
- JDM バリデーションは CI の Schema チェックで強制（不正な決定表の Merge 不可）

### FR-T1-DECISION-002: 決定表バージョン管理

**現状**: 決定表の変更履歴が追えないと、「過去の申請はどのルールで承認されたのか」の監査対応が困難。

**要件達成後**: 決定表は Git 管理され、全バージョンが参照可能。`k1s0.Decision.Evaluate(name, input, version?)` で特定バージョンを指定して評価可能（デフォルトは最新）。評価時に使用したバージョンは Audit API に記録される。

**崩れた時**: 監査で「去年の承認はどのルールか」を聞かれた時に再現不能になり、コンプライアンス違反を疑われる。

**受け入れ基準**:
- Git commit hash がバージョン識別子
- バージョン指定評価をサポート
- 決定表一覧とバージョン履歴を Backstage で参照可能

### FR-T1-DECISION-003: 評価履歴 Audit 連携

**現状**: 業務ルール評価の結果（誰がどの入力で何を返したか）は、tier2 アプリのログに出る程度で、統合的な監査ビューが無い。

**要件達成後**: 全評価操作が tier1 Audit API に自動記録される。Audit イベントには `decision_table_name`、`version`、`input`、`output`、`user_id`、`tenant_id` が含まれる。個人情報を含む入力は PII マスキング連携で保護される。

**崩れた時**: 業務ルール評価の後追い監査が不可能となり、採用検討の不正な書き換えを検知できない。

**受け入れ基準**:
- 評価操作ごとに 1 Audit イベント
- PII マスキングが自動適用（FR-T1-PII-001 連携）
- 7 年保存（Audit 長期保存ポリシー）

### FR-T1-DECISION-004: 決定表ホットリロード

**現状**: 決定表変更には ZEN Engine の再起動（tier1 Pod 再起動）が必要な場合、業務断が発生する。

**要件達成後**: 決定表の Git 更新を Argo CD が検出、ConfigMap として配信、ZEN Engine が watch で自動再読込。tier1 Pod 再起動なしに新版が適用される。再読込中は旧版と新版の両方が短時間共存するが、評価単位では一貫性を保つ。

**崩れた時**: 決定表変更のたびに業務時間外メンテナンスウィンドウを待つ必要があり、緊急ルール変更が遅延する。

**受け入れ基準**:
- Git push から新版評価開始まで 5 分以内
- 再読込中の評価は旧版 or 新版のどちらかで一貫
- 優先度 COULD（リリース時点 で実装判定）

## 入出力仕様

本 API の機械可読な契約骨格（Protobuf IDL）は [40_tier1_API契約IDL/09_Decision_API.md](../40_tier1_API契約IDL/09_Decision_API.md) に定義されている。SDK 生成・契約テストは IDL 側を正とする。以下は SDK 利用者向けの疑似インタフェースであり、IDL の `DecisionService` RPC と意味論的に対応する。

```
k1s0.Decision.Evaluate(
    table_name: string,
    input: JSON,
    options?: {
        version?: string,     // Git commit hash、省略時は最新
        trace?: bool          // 評価トレース（どのルールがマッチしたか）を返す
    }
) -> (output: JSON, trace?: DecisionTrace, error: K1s0Error?)

k1s0.Decision.ListTables() -> (tables: TableInfo[], error?)
k1s0.Decision.GetTable(name: string, version?: string) -> (table: JDM, error?)
```

`DecisionTrace` は「どの条件がマッチしてどの結果になったか」の評価経路を含む構造。障害調査用。

## 受け入れ基準（全要件共通）

- 決定表ファイルの JDM バリデーションを CI で実施、不正な決定表はマージ不可
- 評価入力と出力のサイズ上限 1MB
- ZEN Engine のバージョン差による後方互換破壊を tier1 側で吸収

## 段階対応

- **リリース時点**: FR-T1-DECISION-001（Go SDK 経由の Rust in-process 呼び出し）
- **リリース時点**: FR-T1-DECISION-002、003（バージョン管理、Audit 連携、C# SDK）
- **リリース時点**: FR-T1-DECISION-004（ホットリロード）
- **採用後の運用拡大時**: Python / Rust SDK 直接呼び出し、Backstage JDM エディタ

## 関連非機能要件

- **NFR-B-PERF-004**: Decision 評価 p99 < 1ms
- **NFR-E-MON-003**: Decision 評価の Audit 記録
- **NFR-E-ENC-002**: PII マスキング連携
- **NFR-C-MGMT-001**: 決定表の Git 管理
