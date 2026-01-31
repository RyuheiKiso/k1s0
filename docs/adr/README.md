# Architecture Decision Records (ADR)

本ディレクトリには、k1s0 プロジェクトのアーキテクチャ決定記録（ADR）を格納する。

## ADR とは

ADR は、アーキテクチャ上の重要な決定を記録するための軽量なドキュメント形式である。

- **なぜ**その決定をしたのか（コンテキスト）
- **何を**決定したのか（決定内容）
- **どのような影響**があるか（帰結）

を記録し、将来の開発者が過去の決定を理解できるようにする。

## ADR の作成基準

以下のいずれかに該当する場合、ADR を作成する：

1. **破壊的変更（MAJOR）** を行う場合
2. **規約の追加・変更** を行う場合
3. **技術選定** で複数の選択肢から選定した場合
4. **例外的な運用** を認める場合

## ADR のファイル命名規則

```
ADR-{4桁連番}-{kebab-case-title}.md
```

例：
- `ADR-0001-scope-and-prerequisites.md`
- `ADR-0002-service-mesh-selection.md`

## ADR のステータス

| ステータス | 説明 |
|-----------|------|
| 提案（Proposed） | レビュー中 |
| 承認済み（Accepted） | 採用決定 |
| 非推奨（Deprecated） | 新規では使わないが既存は維持 |
| 却下（Rejected） | 採用しないことを決定 |
| 置換（Superseded） | 別の ADR で置き換えられた |

## ADR 一覧

| ADR | タイトル | ステータス |
|-----|---------|-----------|
| [ADR-0001](ADR-0001-scope-and-prerequisites.md) | k1s0 実装スコープと前提の固定 | 承認済み |
| [ADR-0002](ADR-0002-versioning-and-manifest.md) | バージョニングと manifest の型の固定 | 承認済み |
| [ADR-0003](ADR-0003-template-fingerprint-strategy.md) | テンプレート fingerprint 戦略 | 承認済み |
| [ADR-0004](ADR-0004-docker-integration.md) | Docker 統合コマンドの導入 | 承認済み |
| [ADR-0005](0005-grpc-contract-management.md) | gRPC 契約管理（buf lint/breaking） | 承認済み |
| [ADR-0006](ADR-0006-three-layer-architecture.md) | 3層アーキテクチャ（framework -> domain -> feature） | 承認済み |
| [ADR-0007](ADR-0007-csharp-backend-support.md) | C# バックエンドサポートの追加 | 承認済み |
| [ADR-0008](ADR-0008-python-backend-support.md) | Python バックエンドサポートの追加 | 承認済み |
| [ADR-0009](ADR-0009-kotlin-backend-support.md) | Kotlin バックエンドサポートの追加 | 承認済み |
| [ADR-0010](ADR-0010-android-frontend-support.md) | Android フロントエンドサポートの追加 | 承認済み |
| [ADR-0011](ADR-0011-playground-command.md) | Playground コマンドの導入 | 承認済み |
| [ADR-0012](ADR-0012-migrate-command.md) | Migrate コマンドの導入 | 承認済み |
| [ADR-0013](ADR-0013-observability-infrastructure.md) | 可観測性基盤の技術選定（OTEL Collector + Jaeger + Loki + Prometheus + Grafana） | 承認済み |
| [ADR-0014](ADR-0014-consensus-protocol.md) | コンセンサスプロトコル機構（k1s0-consensus） | 承認済み |
| [ADR-0015](ADR-0015-rate-limiting.md) | レート制限フレームワーク（k1s0-rate-limit） | 承認済み |
| [ADR-0016](ADR-0016-backpressure-control.md) | バックプレッシャー制御機構 | 承認済み |
| [ADR-0017](ADR-0017-ast-based-lint-engine.md) | AST ベース Lint エンジンへの移行 | 承認済み |

## テンプレート

新規 ADR は [TEMPLATE.md](TEMPLATE.md) を使用して作成する。
