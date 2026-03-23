# ADR-0026: Service Tier DB統合設計

## ステータス

承認済み

## コンテキスト

service tier の board サービスと activity サービスが個別データベース（`k1s0_board`、`k1s0_activity`）を期待する設定になっていた。一方、`infra/docker/init-db` スクリプトは `k1s0_service` という単一DBに `board_service`、`activity_service` スキーマとして統合作成していた。さらに Rust コード内のスキーマ参照も一致していなかった。この3重不整合により、board・activity サービスが起動できない状態だった。

task サービスは既に `k1s0_service` DB 内の `task_service` スキーマとして正常に動作しており、これが service tier の正しいパターンとして機能していた。

## 決定

service tier の board サービスと activity サービスは `k1s0_service` DB 内でスキーマ分離する。

- board サービス: `k1s0_service` DB の `board_service` スキーマ
- activity サービス: `k1s0_service` DB の `activity_service` スキーマ
- task サービス: `k1s0_service` DB の `task_service` スキーマ（変更なし）

## 理由

task サービスが既に同パターンで安定稼働しており、一貫性を保つために board・activity も同パターンに揃える。マルチスキーマによるDB統合は以下の利点がある。

- DBサーバー数の削減による運用コストの低下
- `init-db` が単一DB接続で全スキーマを管理できる
- service tier 内のパターン統一による設定ミスの減少
- PostgreSQL のスキーマ分離により、テーブル名の衝突を防ぎつつ同一DB内で論理分離が可能

## 影響

**ポジティブな影響**:

- DB数の削減（3DB → 1DB に統合）
- init-db スクリプトの単純化（接続先を `k1s0_service` に統一）
- task サービスと同パターンになるため、新規サービス追加時の設計が明確

**ネガティブな影響・トレードオフ**:

- board・activity の `config.yaml` でデータベース名を `k1s0_service` に変更が必要
- Rust コード内のスキーマプレフィックス参照（`board_service.`、`activity_service.`）を統一が必要
- スキーマ間の物理的な隔離はなく、誤ったクエリが他スキーマのテーブルに影響しうる（アプリ層の責務）

## 代替案

| 案 | 概要 | 採用しなかった理由 |
|----|------|-----------------|
| 案 A: 個別DBを作成 | board/activity それぞれ専用DBを作成し、config.yaml の期待値に合わせる | DBユーザー管理の複雑化、DBサーバー数増加、init-db の複数DB接続が必要 |
| 案 B: スキーマなしで統合 | 全サービスのテーブルを `k1s0_service` DB のデフォルトスキーマに集約 | テーブル名の衝突リスク、論理分離の喪失 |

## 参考

- [ADR-0012: system-tier-rls-scope](./0012-system-tier-rls-scope.md)
- [ADR-0027: DBアプリケーションユーザー権限分離](./0027-db-app-user-role-separation.md)

## 更新履歴

| 日付 | 変更内容 | 変更者 |
|------|---------|--------|
| 2026-03-23 | 初版作成 | @k1s0-team |
