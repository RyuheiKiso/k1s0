# ADR-RULE-002: ワークフロー基盤に Temporal を採用

- ステータス: Accepted
- 起票日: 2026-04-19
- 決定日: 2026-04-19
- 起票者: kiso ryuhei
- 関係者: tier1 開発チーム / tier2 リードエンジニア / 運用チーム

## コンテキスト

tier1 Workflow API（FR-T1-WORKFLOW-001〜005）は、長時間実行されるビジネスワークフロー（承認フロー、バッチ処理、マイクロサービスオーケストレーション、SAGA パターンでの補償処理等）を持続的に実行する基盤を必要とする。

課題は以下の通り。

- **永続性**: プロセス再起動・ホスト障害でもワークフロー状態が失われない
- **非同期 / 長時間**: 承認待ちで数日〜数週間進行しないワークフローも成立
- **Retry と補償**: 失敗時の自動リトライ、SAGA パターンでの補償処理
- **観測性**: ワークフローの実行履歴・現在の状態を運用側から可視化
- **Determinism**: Replay による復元時に同じ結果を保証

従来の実装は、アプリ側でジョブ管理テーブル・状態遷移を手書きし、cron + ジョブキューで進行させる方式だったが、「ワークフローの可視化」「複雑な SAGA パターン」「再起動後の再開」を手実装すると膨大なコード量になり、不具合の温床になる。

候補は Temporal、Cadence、Netflix Conductor、AWS Step Functions（SaaS 除外）、Camunda Zeebe など。

## 決定

**ワークフロー基盤は Temporal（MIT、Temporal Technologies）を採用する。**

- Temporal 1.24+
- Persistence は PostgreSQL（ADR-DATA-001）
- tier1 Rust 自作領域から Temporal Go SDK / Rust SDK（experimental）で Worker を起動
- ワークフロー定義は tier2 側のコード（Go / Rust）で、Temporal の規約に従って記述
- Determinism 違反検出は Replay テストを CI/CD 必須（FMEA RPN 135 対策）
- Temporal Web UI（Backstage 内に埋込み）で運用側から実行履歴を可視化
- Worker の HPA 連動、Task Queue depth を SLI 化

## 検討した選択肢

### 選択肢 A: Temporal（採用）

- 概要: Uber の Cadence から fork、MIT、業界デファクト
- メリット:
  - 大規模運用実績（Uber、Netflix、Stripe、Snap 等）
  - SDK が Go、Java、TypeScript、Python、.NET、Rust (experimental) 揃う
  - Determinism、Replay、Signal、Query、Child Workflow、SAGA すべて標準
  - Web UI で実行履歴可視化
  - PostgreSQL / MySQL / Cassandra のいずれでも永続化可能
- デメリット:
  - Determinism ルールの学習コスト（非決定コードを書きにくい）
  - 運用に Worker、Server、Database の 3 コンポーネント必要

### 選択肢 B: Cadence（Uber 本家）

- 概要: Temporal の元。MIT
- メリット: Uber の内製実績
- デメリット:
  - Temporal が fork で事実上の後継、コミュニティの活発度で劣る

### 選択肢 C: Netflix Conductor

- 概要: Netflix 発、JSON DSL でワークフロー
- メリット: GUI エディタあり、DSL 記述
- デメリット:
  - Determinism・Replay の成熟度が Temporal より低い
  - Java 前提、Rust/Go 利用が不自然

### 選択肢 D: Camunda Zeebe

- 概要: BPMN ネイティブの Workflow Engine
- メリット: BPMN 標準準拠
- デメリット:
  - コードファーストの tier1 設計と相性が悪い
  - 運用コストが大（ブローカー、Zeebe クラスタ）

### 選択肢 E: 自作（DB + cron + ジョブキュー）

- 概要: PostgreSQL + Kafka + アプリ実装
- メリット: ツール追加なし
- デメリット:
  - 膨大な工数、不具合リスク高
  - 可視化・Determinism・Replay の自作は現実的でない

## 帰結

### ポジティブな帰結

- 長時間ワークフローが自作不要、tier2 開発者はワークフローロジックに集中
- Replay で過去のワークフロー実行を再現、デバッグ・検証容易
- Web UI で運用側の可視化、異常検知
- SAGA パターンの補償処理が標準サポート

### ネガティブな帰結

- Determinism 違反（FMEA RPN 135）が最大リスク、CI/CD での Replay テスト必須
- Temporal の運用コンポーネント（Server、Worker、DB）の冗長化必要
- SDK Version upgrade 時の Workflow 互換性検証（バージョニング戦略が設計必要）
- PostgreSQL 依存で、Postgres 障害時のワークフロー進行停止

## 実装タスク

- Temporal Helm Chart バージョン固定、Argo CD 管理
- PostgreSQL CloudNativePG と統合、Schema migration 手順 Runbook
- Temporal Web UI を Backstage に埋込み、SSO 統合
- Replay テストライブラリ（temporalio/replay）を tier2 の CI/CD 必須化
- Task Queue depth 監視、HPA 連動
- Workflow 命名規則・versioning ポリシー策定（DX-GP Golden Path）
- 運用 Runbook: Workflow 強制終了、再実行、migration 手順

## 参考文献

- Temporal 公式: temporal.io
- Temporal Documentation: Determinism / Replay / Versioning
- SAGA パターン: microservices.io/patterns/data/saga.html
- Uber Cadence 論文
