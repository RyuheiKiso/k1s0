# ADR-0109: vault-db / event-monitor-db / master-maintenance-db のテナント分離除外設計

## ステータス

承認済み

## コンテキスト

外部技術監査（HIGH-003/004/005, MED-004, LOW-003）により、以下の 3 データベースに RLS（Row Level Security）が実装されていないことが指摘された。

| データベース | 指摘 |
|------------|------|
| vault-db | key_path による論理分離のみで RLS なし（HIGH-003）|
| event-monitor-db | システムレベルのイベント監視 DB に RLS なし（HIGH-004）|
| master-maintenance-db | マスターデータ定義 DB に RLS なし（HIGH-005）|

本 ADR は、これら 3 DB を RLS 適用範囲から除外する設計根拠と、vault-db における key_path バリデーションによる補完的テナント分離策を明文化する。

## 決定

### vault-db

vault-db はテナント固有シークレットを `key_path` プレフィックスで論理分離する。RLS は適用しないが、ユースケース層でテナント境界を強制する。

**key_path バリデーション規則**:
- 全 Vault 操作（Create / Get / Delete / List）は `key_path` が `{tenant_id}/` で始まることをユースケース層で検証する
- バリデーション失敗時は PERMISSION_DENIED エラーを返し操作を拒否する
- RLS と異なり、バリデーションは DB レベルではなくアプリケーション層で適用される

### event-monitor-db

event-monitor-db は全テナントにまたがるシステムレベルのイベント監視基盤であり、設計上テナントスコープを持たない。

**除外理由**:
- `flow_definitions` および `event_records` テーブルはインフラ横断の監視定義・記録であり、特定テナントに属さない
- 本 DB は k1s0_system スキーマのシステムサービスのみがアクセスし、テナント API からは直接参照されない
- 監視データは全テナントの集約ビューを必要とするため、テナントフィルタリングは機能要件と矛盾する

### master-maintenance-db

master-maintenance-db はテナント横断のマスターデータ定義を保持するメタデータスキーマである。

**除外理由**:
- `table_definitions`, `column_definitions` 等のテーブルはシステム全体のスキーマメタデータであり、テナント概念を持たない
- 本 DB はシステム管理者のみが書き込み権限を持ち、テナントアプリケーションは読み取り専用でアクセスする
- マスターデータはテナント間で共有されることが設計意図であり、RLS によるテナント分離は機能要件と矛盾する

## 理由

### 代替案との比較

| 案 | 内容 | 採用しなかった理由 |
|----|------|-----------------|
| vault-db に RLS を追加 | tenant_id カラムを追加して RLS を適用 | key_path プレフィックスによる論理分離が既に実装されており、アプリケーション層バリデーションとの二重管理が生じる。また既存の key_path 体系との後方互換性が失われる |
| event-monitor-db にテナントカラム追加 | event_records に tenant_id を追加して RLS | 全テナント集約が必要なシステム監視の機能要件と矛盾する。監視基盤のデータモデルを変更するリスクが高い |
| master-maintenance-db にテナントカラム追加 | table_definitions に tenant_id を追加 | マスターデータはシステム横断の共有定義であり、テナント固有にする意図がない |

### vault-db のリスク軽減策

RLS 非採用の補完策として以下を実装する：

1. **ユースケース層バリデーション**: 全 Vault 操作で `key_path` が `{tenant_id}/` プレフィックスを持つことを検証
2. **ロールベースアクセス制御**: vault-server のみが vault-db への接続権限を持つ（他サービスは直接アクセス不可）
3. **audit log**: 全 Vault 操作は監査ログに記録する

## 影響

**ポジティブな影響**:
- vault-db: key_path バリデーションにより、RLS と同等のテナント境界強制を実現できる
- event-monitor-db / master-maintenance-db: 現行の設計意図を維持し、機能の整合性を保てる
- テナント分離の除外理由がドキュメント化され、将来の監査に対して説明責任を果たせる

**ネガティブな影響・トレードオフ**:
- vault-db: RLS と異なり、アプリケーション層バリデーションはバグや実装漏れのリスクがある。全 Vault 操作に対してバリデーションを徹底する必要がある
- event-monitor-db / master-maintenance-db: アクセス制御がロールベースに限定されるため、将来テナント固有の監視データが必要になった場合は設計変更が必要

## 代替案

| 案 | 概要 | 採用しなかった理由 |
|----|------|-----------------|
| 全 3 DB に RLS を追加 | セキュリティ強化 | 機能要件との矛盾・マイグレーションコスト・後方互換性の問題 |
| vault-db を RLS + key_path の二重管理 | 多層防御 | 実装複雑性の増大と、既存の key_path モデルとの整合性破綻 |

## 参考

- [ADR-0042: RLS 適用方針（アプリケーション DB 全体）](0042-rls-per-database.md)
- [ADR-0080: outbox IS NULL 設計根拠](0080-outbox-is-null-design.md)
- [multi-tenancy.md](../multi-tenancy.md) — RLS 除外 DB 一覧
- 外部技術監査報告書 HIGH-003/004/005, MED-004, LOW-003

## 更新履歴

| 日付 | 変更内容 | 変更者 |
|------|---------|--------|
| 2026-04-05 | 初版作成（外部監査 HIGH-003/004/005, MED-004, LOW-003 対応） | @kiso |
