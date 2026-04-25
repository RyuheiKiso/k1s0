# 開発者ポータル (Backstage)

## 目的

k1s0 上で稼働する各種サービス (tier1 / tier2 / tier3) を **開発者・運用者が一覧 / 検索 / 把握 / 起票できる場所** を Backstage で構築する。エンドユーザー向けの [`02_アプリ配信ポータル.md`](./02_アプリ配信ポータル.md) とは対象ユーザーも目的も異なる別物である。

---

## 1. Backstage の位置付け

Backstage は「k1s0 の代替」ではなく「k1s0 の構成要素 (開発者ポータル領域)」である。アプリ配信ポータルとは **対象ユーザーで完全に分離** する。混同を避けるため社内呼称は次のとおり統一する。

| 呼称 | 実体 | 対象ユーザー |
|---|---|---|
| 開発者ポータル | Backstage | 開発者 / 運用者 / SRE |
| 業務アプリストア | アプリ配信ポータル (k1s0 自製 tier3) | 業務担当 / エンドユーザー |

---

## 2. 採用理由

### メリット

| 項目 | 内容 |
|---|---|
| OSS / Apache 2.0 | 採用検討ハードルなし |
| CNCF Incubating | エコシステムが豊富、長期存続性が見込める |
| プラグイン文化 | Grafana Tempo / Grafana / GitHub Actions / Kubernetes など主要ツールの統合済みプラグインが揃う |
| Software Templates | 雛形生成 CLI と統合可能 (Backstage UI から CLI を起動) |
| TechDocs | Markdown で書いたサービスドキュメントを自動公開 |
| 採用事例の多さ | Spotify / アメリカン航空 / Expedia 他、ガバナンス資料が豊富 |

### k1s0 が自製しない理由

- 開発者ポータルは「機能の幅」が必要で自製は車輪の再発明
- Backstage は「機能の幅」を OSS で既に実現しており、k1s0 の限られたリソースを浪費する必要がない
- アプリ配信ポータルとは対象ユーザーが違うため、UI / UX を別物として設計する方が分かりやすい

---

## 3. 採用する Backstage 機能

| 機能 | 用途 | 採用段階 |
|---|---|---|
| **Software Catalog** | tier1 / tier2 / tier3 の全サービスを一覧化。オーナーチーム / 依存関係 / API / Lifecycle を可視化 | リリース時点 |
| **TechDocs** | tier1 公開 API のリファレンス、各サービスの README / 設計書を Markdown で集中管理 | リリース時点 |
| **Software Templates** | 新規サービス作成時に Backstage UI からテンプレート選択 → k1s0 雛形生成 CLI を裏で起動 | 採用後の運用拡大時 |
| **Kubernetes Plugin** | 各サービスの Pod / Deployment / リソース状況を Backstage 上で確認 | 採用後の運用拡大時 |
| **Grafana Tempo Plugin** | 分散トレースのリンクをサービスページから直接開ける (Grafana 経由) | 採用後の運用拡大時 |
| **Grafana Plugin** | サービスごとのダッシュボードを埋め込み | 採用後の運用拡大時 |
| **GitHub Actions Plugin** | CI/CD ステータスをサービスページで表示 | 採用後の運用拡大時 |
| **Argo CD Plugin** | デプロイ状態を可視化 | 採用後の運用拡大時 |
| **API Docs** | OpenAPI / gRPC proto を公開してサービス間連携の発見性を上げる | 採用側のマルチクラスタ移行時 |
| **Cost Insights** (任意) | リソース使用量とコストの可視化 | 採用側の全社展開期 以降 |

---

## 4. Software Templates と雛形生成 CLI の統合

k1s0 には独自の **雛形生成 CLI** がある ([`../02_tier1設計/02_API契約/03_API設計原則.md`](../02_tier1設計/02_API契約/03_API設計原則.md) 参照)。Backstage の Software Templates 機能はこの CLI と連動させる。

| ステップ | 担当 |
|---|---|
| テンプレート選択 UI (C# / Go / TS 等) | Backstage |
| パラメータ入力 (サービス名 / オーナー) | Backstage |
| 雛形ファイル生成ロジック | k1s0 雛形生成 CLI |
| GitHub リポジトリへの初期コミット | 雛形生成 CLI |
| Backstage Catalog への登録 | 雛形生成 CLI が `catalog-info.yaml` を生成 |

### 設計原則

- 雛形生成のロジックは k1s0 CLI 側に持ち、Backstage 側に複雑なロジックを置かない
- Backstage は **UI とパラメータ入力を担当**
- これにより CLI 単体でも (CI / 自動化スクリプトから) 利用可能なまま

---

## 5. アプリ配信ポータルとの棲み分け

| 軸 | アプリ配信ポータル (k1s0 自製) | Backstage (開発者ポータル) |
|---|---|---|
| 対象ユーザー | 業務担当 / エンドユーザー | 開発者 / 運用者 / SRE |
| 主な目的 | 業務アプリの利用開始 | サービスの発見・把握・運用 |
| 表示内容 | 業務アプリ一覧 / 説明 / レビュー | サービスカタログ / 依存関係 / API / ログリンク |
| インストール対象 | 業務アプリ (PWA / MSIX) | なし (リンク先を開くだけ) |
| 認証 | 全社員 SSO | 開発者アカウント (限定) |
| 端末設定コピー | あり | なし |
| 監査 | tier1 監査ログ | Backstage 標準ログ + tier1 監査ログ |
| ホスティング | tier3 として k1s0 上 | `operation` namespace |
| 実装責任 | tier3 (個別アプリ開発チーム) | 運用チーム + システム基盤チーム |

---

## 6. 配置と運用

| 項目 | 方針 |
|---|---|
| 配置 namespace | `operation` (運用ツールのため) |
| 認証 | Keycloak OIDC 連携 (tier1 認証ライブラリと同じ ID 基盤) |
| データベース | PostgreSQL (Backstage 標準)。`infra` 層の CloudNativePG 共有クラスタ上に `backstage` DB を作成 |
| バックアップ / アップグレード | 運用チーム責任 |
| 参照先 | k8s API / Grafana Tempo / Grafana / Loki / Prometheus / GitHub / tier1 監査ログ API |

---

## 7. 採用側組織の情シス特有の配慮

| 配慮 | 対応 |
|---|---|
| 閉域ネットワーク | Backstage は 100% オンプレで動作。プラグインのインストールは社内 npm レジストリ経由 |
| 日本語化 | Backstage の i18n 機能で日本語ラベル提供。サービス説明文は元から日本語で書く |
| 学習コスト | 「サービスカタログを見る」だけなら学習不要。テンプレート利用は開発者向けなので情シス全体の負担にならない |
| バージョン更新の追従 | Backstage はアップデート頻度が高い。LTS バージョンを採用し、年 1 回の計画的更新に留める |

---

## 8. 段階計画

| 段階 | 含める機能 |
|---|---|
| **リリース時点** | Backstage 基本セットアップ / Software Catalog / TechDocs / SSO 連携 |
| **採用後の運用拡大時** | Software Templates (雛形生成 CLI 統合) / Kubernetes / Grafana Tempo / Grafana / GitHub Actions / Argo CD プラグイン |
| **採用側のマルチクラスタ移行時** | API Docs (OpenAPI / proto) / 依存関係グラフ / オーナーシップ管理の徹底 |
| **採用側の全社展開期** | プラグイン拡張 / 内製プラグイン (k1s0 固有メトリクス等) |

---

## 関連ドキュメント

- [`00_CICDパイプライン.md`](./00_CICDパイプライン.md) — Backstage への統合アノテーション
- [`02_アプリ配信ポータル.md`](./02_アプリ配信ポータル.md) — エンドユーザー向けポータルとの棲み分け
- [`../01_アーキテクチャ/01_基礎/03_配置形態.md`](../01_アーキテクチャ/01_基礎/03_配置形態.md) — `operation` namespace の配置方針
- [`../02_tier1設計/02_API契約/03_API設計原則.md`](../02_tier1設計/02_API契約/03_API設計原則.md) — 雛形生成 CLI の位置付け
- [`../03_技術選定/03_周辺OSS/02_周辺OSS.md`](../03_技術選定/03_周辺OSS/02_周辺OSS.md) — Keycloak / Argo CD の選定根拠
