# ADR-0009: Docker / Docker Compose ファーストクラスサポート

## ステータス

承認（Accepted）

## 日付

2026-01-30

## コンテキスト

k1s0 プラットフォームはサービスのスキャフォールディングとコンベンション強制を提供するが、ローカル開発環境のコンテナ化が標準化されていなかった。各チームが独自に Dockerfile と docker-compose.yaml を作成しており、以下の課題があった：

- Dockerfile のベストプラクティス（multi-stage build、non-root ユーザー、HEALTHCHECK）が統一されていない
- 依存サービス（PostgreSQL、Redis、OTEL Collector）の構成が一貫していない
- monorepo での Docker ビルドコンテキストの扱いが標準化されていない
- Docker Secrets パターンが k1s0 の K021（シークレットハードコード禁止）規約と整合していない

## 決定

Docker と Docker Compose V2 を k1s0 のファーストクラスサポートとして統合する。

### 主要な決定事項

1. **Dockerfile テンプレート**: 全 6 テンプレート（backend-rust/go/csharp/python、frontend-react/flutter）に Dockerfile.tera を提供
2. **2 つのビルドモード**: Mode A（standalone context）と Mode B（monorepo root context）で Dockerfile と Dockerfile.monorepo を分離
3. **compose.yaml テンプレート**: 条件付きサービス定義（`{% if with_db %}`、`{% if with_cache %}`）
4. **`k1s0 docker` サブコマンド**: up / down / logs / build / ps のラッパー
5. **`k1s0 init` 統合**: ルートレベル compose.yaml、.hadolint.yaml、.dockerignore を生成
6. **lint 強制**: K010/K011 で feature 層の Dockerfile を必須ファイル化
7. **プロキシ対応**: 全テンプレートに HTTP_PROXY/HTTPS_PROXY/NO_PROXY ARG を組み込み
8. **observability プロファイル**: otel-collector を `profiles: [observability]` で分離

### 選択肢の比較

| 選択肢 | 長所 | 短所 |
|--------|------|------|
| A: テンプレートに Dockerfile 組み込み | 一貫性、自動生成、lint 統合 | テンプレート複雑化 |
| B: ドキュメントのみ提供 | シンプル | 一貫性なし、手動作業 |
| C: 外部ツール連携（Buildpacks 等） | 柔軟 | 学習コスト、k1s0 との統合困難 |

### 採用理由

選択肢 A を採用。k1s0 のコア哲学である「コンベンション強制」と「自動生成」に最も合致するため。

## 帰結

### 正の帰結

- 全サービスの Dockerfile が統一されたベストプラクティスに従う
- `k1s0 new-feature` で即座にコンテナ化可能なサービスが生成される
- Docker Secrets パターンが K021 規約と自動的に整合する
- monorepo ビルドが標準化される

### 負の帰結

- テンプレートの複雑さが増加（Tera 条件分岐の増加）
- Docker 関連の知識が k1s0 ユーザーに必要
- `--no-docker` オプトアウトが必要

### リスクと軽減策

| リスク | 軽減策 |
|--------|--------|
| Dockerfile テンプレートの陳腐化 | hadolint CI + 定期的なベースイメージ更新 |
| 言語固有の最適化不足 | multi-stage build + 言語別テンプレート |
| Docker 非利用チームへの負担 | `--no-docker` フラグでオプトアウト |

## 関連ドキュメント

- docs/design/template.md - テンプレートシステム設計
- docs/design/cli.md - CLI 設計
- docs/conventions/service-structure.md - サービス構造規約
- docs/conventions/config-and-secrets.md - 設定・シークレット規約
