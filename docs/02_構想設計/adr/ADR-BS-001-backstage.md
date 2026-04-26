# ADR-BS-001: 開発者ポータルに Backstage を採用

- ステータス: Accepted
- 起票日: 2026-04-19
- 決定日: 2026-04-19
- 起票者: kiso ryuhei
- 関係者: tier2/tier3 開発チーム / DevEx チーム / Product Council

## コンテキスト

k1s0 は多数のマイクロサービス（tier1 公開 11 API、tier2 ドメインサービス、tier3 UI/BFF）が集合する PaaS であり、開発者が横断的に必要な情報は多岐にわたる。具体的には以下の 4 つの課題が慢性的に発生する。

1. **サービスカタログの欠如**: 「このサービスは誰が所有しているか」「どの API に依存しているか」「SLO はどうか」が散在し、属人化する
2. **雛形生成の非標準化**: 新規サービス立ち上げで Protobuf IDL・CI/CD・Dockerfile・観測設定をコピペする文化が発生し、Golden Path から逸脱する
3. **ドキュメント分散**: README / Confluence / Wiki / Git issue にドキュメントが散らばり、検索不能
4. **Feature Flag / インシデント / SLO ダッシュボードが別 UI**: 開発者がタブを大量に開く必要がある

Spotify の Backstage は CNCF Incubating で、これらを統合する開発者ポータルとして業界標準化している。

## 決定

**開発者ポータルは Backstage（CNCF Incubating、Apache 2.0）を採用する。**

- Backstage 1.26+
- Software Catalog（catalog-info.yaml）を各サービス Git Repo に配置、自動検出
- Software Templates（Golden Path）で Protobuf IDL・CI/CD・Dockerfile・観測設定を自動生成
- TechDocs（MkDocs ベース）でドキュメント統合、Git Repo 内の md を自動ビルド
- Backstage Plugin: Argo CD / Grafana / Temporal / Keycloak / Feature Flag（flagd）を埋込み
- Keycloak と OIDC 統合（ADR-SEC-001）で SSO 化
- tier2/tier3 開発者は Backstage 経由で k1s0 のすべての機能にアクセス

## 検討した選択肢

### 選択肢 A: Backstage（採用）

- 概要: Spotify 発、CNCF Incubating、業界標準の IDP（Internal Developer Portal）
- メリット:
  - Software Catalog / Template / TechDocs / Plugin が統合
  - Plugin エコシステムが豊富（300+ plugin）
  - Argo CD / Grafana / Kubernetes / Temporal 等のメジャーツールは公式 Plugin 存在
  - OIDC 統合標準、RBAC を内蔵
- デメリット:
  - TypeScript / Node.js 前提、運用知識が必要
  - Plugin カスタマイズで UI 開発スキルが必要
  - Backstage Server の運用（DB、HA）が必要

### 選択肢 B: Port

- 概要: SaaS の IDP
- メリット: UX 洗練、Plugin 豊富
- デメリット:
  - SaaS、オンプレ制約で対象外
  - 商用ライセンス費用

### 選択肢 C: 自作ポータル

- 概要: React (Vite) + Dapr で独自実装
- メリット: 完全カスタマイズ
- デメリット:
  - 実装工数が膨大（Catalog / Template / Plugin エコシステムを自作）
  - 車輪の再発明

### 選択肢 D: Confluence + GitLab Wiki 組合せ

- 概要: 従来型のドキュメント + Wiki
- メリット: 既存インフラ活用
- デメリット:
  - Software Catalog / Template の概念なし
  - 自動化できない、属人化する
  - Plugin 統合できない

### 選択肢 E: GitLab Developer Portal（GitLab Premium）

- 概要: GitLab 内蔵
- メリット: GitLab 利用済み組織で即採用可
- デメリット:
  - Backstage の Catalog / Template / Plugin エコシステムに比べて機能が薄い
  - 組織が GitHub 採用時に使えない

## 帰結

### ポジティブな帰結

- 開発者は Backstage 単一 UI で全機能にアクセス、DX-GP Golden Path の起点
- Software Template で新規サービス立ち上げが 1 日以下に短縮（DX-VEL-001）
- TechDocs でドキュメント検索が統一、ナレッジロスを最小化
- Software Catalog でサービス所有者・依存関係・SLO が可視化、インシデント対応の起点

### ネガティブな帰結

- Backstage Server の運用コスト（Node.js、PostgreSQL、HA）
- Plugin カスタマイズで UI 開発スキル必要、DevEx チームに TypeScript 人材確保
- Plugin バージョン互換性の監視、アップグレード工数
- 初期 Plugin 構築に 3〜6 ヶ月の工期

## 実装タスク

- Backstage Helm Chart バージョン固定、Argo CD 管理
- PostgreSQL（CloudNativePG）をバックエンドに統合
- Keycloak OIDC 統合、RBAC 設定
- Software Catalog: catalog-info.yaml のスキーマ策定、自動検出設定
- Software Templates: tier2 / tier3 / ライブラリ の 3 種類の Golden Path Template
- TechDocs: MkDocs 統合、docs/ ディレクトリからビルド
- Plugin 統合: Argo CD / Grafana / Temporal Web / Feature Flag / Kubernetes
- SSO 統合と RBAC（Realm / Role）
- 開発者向け初期トレーニング（BC-TRN）、ナレッジ基盤整備

## 参考文献

- Backstage 公式: backstage.io
- CNCF Backstage Project
- Spotify Engineering Blog: Backstage の背景
- Internal Developer Platform (internaldeveloperplatform.org)
