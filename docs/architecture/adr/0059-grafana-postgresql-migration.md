# ADR-0059: Grafana SQLite → PostgreSQL 移行

## ステータス

承認済み

## コンテキスト

本番環境で Grafana が SQLite をデータベースバックエンドとして使用しており、以下の問題が生じていた（M-09 監査対応）。

- SQLite はファイルベースの単一プロセス DB であるため、複数の Grafana インスタンスを同時に起動する水平スケールアウトが不可能
- HA（高可用性）構成で Grafana を複製した場合、セッション情報・ダッシュボード設定・アラートルールが各インスタンス間で共有されない
- SQLite ファイルは PersistentVolume 上に存在するため、Pod 再スケジュール時にデータが失われるリスクがある
- 負荷集中時に SQLite のロック競合によりダッシュボードアクセスが遅延する

k1s0-system インフラにはすでに PostgreSQL クラスターが稼働しており、Grafana 専用データベースを追加することで既存インフラを再利用できる。

## 決定

Grafana のデータベースバックエンドを SQLite から PostgreSQL（k1s0-system インフラの既存クラスター）に移行する。

Helm values に以下の設定を追加する:

```yaml
# Grafana の DB バックエンドを PostgreSQL に切り替え、HA 構成を可能にする
grafana:
  grafana.ini:
    database:
      type: postgres
      host: postgres-grafana.monitoring.svc.cluster.local:5432
      name: grafana
      user: grafana
      # パスワードは Kubernetes Secret から環境変数経由で注入する
      password: ${GF_DATABASE_PASSWORD}
      ssl_mode: require
```

Kubernetes Secret（`grafana-db-secret`）を作成し、`GF_DATABASE_PASSWORD` を注入する。

## 理由

- **HA 構成の実現**: PostgreSQL バックエンドに移行することで、Grafana の複数レプリカが同一 DB を共有でき、セッション・設定の一貫性が保証される
- **既存インフラの活用**: k1s0-system の PostgreSQL クラスターは監視・バックアップ・RLS 等の運用が整備されており、Grafana 専用 DB を追加するだけで運用コストが低い
- **セッション共有**: 複数インスタンス間でログインセッションが共有されるため、負荷分散環境でのユーザー体験が向上する
- **ダッシュボード永続化**: SQLite ファイルとは異なり、Pod 再スケジュール時もデータが失われない
- **スケールアウト**: 水平スケールが可能になり、負荷増加時に Grafana レプリカを追加できる

## 影響

**ポジティブな影響**:

- Grafana の水平スケールアウト（HPA）が可能になる
- Pod 再スケジュール時のデータロストリスクがなくなる
- 複数インスタンス間でセッション・ダッシュボードが共有される
- PostgreSQL の定期バックアップにより Grafana データも保護される

**ネガティブな影響・トレードオフ**:

- Grafana 専用の PostgreSQL ユーザー・データベースを作成する初期セットアップが必要
- PostgreSQL クラスターへの依存が増加し、PostgreSQL 障害時に Grafana も影響を受ける
- 既存の SQLite データを PostgreSQL に移行する場合は grafana-migrate ツールの実行が必要
- Helm values への DB 接続設定追加（`grafana.ini.database` セクション）が必要

## 代替案

| 案 | 概要 | 採用しなかった理由 |
|----|------|-----------------|
| SQLite のまま単一レプリカ（現状維持） | Grafana を 1 レプリカに固定し、SQLite を継続使用 | HA 構成が不可能。Pod 障害時のダウンタイムが発生する。M-09 監査対応にならない |
| 外部 Grafana Cloud 利用 | クラウドマネージドの Grafana を使用 | k1s0 の監視データを外部サービスに送信するセキュリティリスクがある。コスト増加。オフライン環境対応不可 |
| MySQL バックエンド | MySQL を新規構築して使用 | k1s0-system に既存の PostgreSQL クラスターがあり、新規 MySQL の構築・運用は不要なコストを増加させる |

## 参考

- [インフラ設計](../../infrastructure)
- [Grafana 公式ドキュメント: Configure database](https://grafana.com/docs/grafana/latest/setup-grafana/configure-grafana/#database)
- M-09 外部技術監査指摘: Grafana の SQLite 使用によるスケールアウト不可問題

## 更新履歴

| 日付 | 変更内容 | 変更者 |
|------|---------|--------|
| 2026-03-30 | 初版作成 | @team |
