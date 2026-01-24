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

## テンプレート

新規 ADR は [TEMPLATE.md](TEMPLATE.md) を使用して作成する。
