# ADR-0070: event-monitor / master-maintenance の独立 DB 分離

## ステータス

承認済み

## コンテキスト

event-monitor と master-maintenance の両サービスは k1s0_system DB を共有していた。
sqlx は `_sqlx_migrations` テーブルでマイグレーション番号を管理しているが、
2 サービスが同一テーブルを共有した場合、マイグレーション番号が衝突する。
実際に master-maintenance のマイグレーション番号が event-monitor の既存番号と
重複したことにより、master-maintenance が起動不能な状態となった。

また、DB を共有することで以下の問題も発生していた:

- どちらのサービスのマイグレーションが失敗したか特定が困難
- 一方のサービスの DB スキーマ変更が他方のサービスに影響する可能性
- バックアップ・リストア時にサービス境界をまたいだデータが混在する
- DB レベルでの権限分離ができない

## 決定

event-monitor と master-maintenance をそれぞれ独立した専用 DB に分離する。

- event-monitor: `k1s0_event_monitor`
- master-maintenance: `k1s0_master_maintenance`

各サービスの `DATABASE_URL` 環境変数・Kubernetes Secret・docker-compose
設定を新 DB 名に更新し、`_sqlx_migrations` テーブルを各 DB で独立して管理する。

## 理由

DB を共有する設計はマイクロサービスの原則に反する。
各サービスが独自の DB を持つことで、以下が達成できる:

1. **マイグレーション衝突の根本解決**: 各サービスが独立した `_sqlx_migrations`
   テーブルを持つため、番号の衝突が物理的に発生しない
2. **サービス独立デプロイの実現**: DB スキーマの変更が他サービスに影響せず、
   独立したリリースサイクルを維持できる
3. **障害影響範囲の局所化**: 一方の DB 障害が他サービスに波及しない
4. **権限管理の明確化**: サービスごとに DB ユーザー権限を最小化できる
5. **バックアップ対象の明確化**: サービスごとに独立したバックアップ戦略を適用できる

代替案として検討したマイグレーション番号の振り直しは、既存データに対する
再マイグレーションを要求するため運用リスクが高く、また根本原因（DB 共有）を
解決しないため非推奨とした。

## 影響

**ポジティブな影響**:

- `_sqlx_migrations` 衝突によるサービス起動不能の根本解決
- マイグレーション失敗箇所の特定が容易になる
- 各サービスの独立デプロイが可能になる
- DB 単位のバックアップ・リストアが明確になる
- 将来的な DB スケーリング（CPU/メモリ設定）を個別に調整できる

**ネガティブな影響・トレードオフ**:

- 初回環境構築時に DB を追加作成する手順が増える
- docker-compose、Kubernetes Secret、Helm values など設定ファイルの
  更新箇所が増える（一度きりの対応）
- PostgreSQL の接続プール数が増加する可能性がある

## 代替案

| 案 | 概要 | 採用しなかった理由 |
|----|------|-----------------|
| マイグレーション番号の振り直し | event-monitor と master-maintenance のマイグレーション番号帯を分割（例: event-monitor は 001〜100、master-maintenance は 101〜200） | 暫定対処に過ぎず、DB 共有のリスクは残存する。スキーマ論理分離を阻害し、将来の番号管理も煩雑になるため非推奨 |
| スキーマ分離（同一 DB 内の異なる schema） | PostgreSQL の schema 機能で論理分離する | `_sqlx_migrations` テーブルはデフォルトでスキーマを考慮しないため、sqlx 設定変更が必要になる。また DB レベルの権限分離ができない |

## 参考

- [ADR-0060: saga-rust 専用データベース分離](./0060-saga-dedicated-database.md)
- [ADR-0026: Service Tier DB 統合設計](./0026-service-tier-db-integration.md)
- [sqlx マイグレーション公式ドキュメント](https://docs.rs/sqlx/latest/sqlx/macro.migrate.html)

## 更新履歴

| 日付 | 変更内容 | 変更者 |
|------|---------|--------|
| 2026-04-02 | 初版作成 | kiso ryuhei |
