# ADR-0001: k1s0 実装スコープと前提の固定

## ステータス

承認済み（Accepted）

## 日付

2026-01-25

## コンテキスト

k1s0 は framework / templates / CLI を含むモノレポとして、開発基盤チームが提供する共通部品と、個別機能チームが開発するサービスを統合的に管理するプロジェクトである。

本 ADR は、k1s0 の実装計画（フェーズ0〜34）における「対象範囲」と「対象外」を明確に固定し、以降の実装フェーズで混乱が生じないようにする。

## 決定

### 対象（本プランで作成するもの）

以下を k1s0 実装計画の対象とする：

#### 1. CLI（Rust）

- `k1s0 init`: リポジトリ初期化（`.k1s0/` 作成等）
- `k1s0 new-feature`: 新規サービスの雛形生成
- `k1s0 lint`: 規約違反の検査
- `k1s0 upgrade --check`: 差分提示と衝突検知
- `k1s0 upgrade`: テンプレート更新の適用

#### 2. templates

- backend/rust: Rust バックエンドサービスの雛形
- backend/go: Go バックエンドサービスの雛形（置き場の固定）
- frontend/flutter: Flutter フロントエンドの雛形（置き場の固定）
- frontend/react: React フロントエンドの雛形（置き場の固定）
- framework 共通サービス: auth-service / config-service / endpoint-service の雛形

#### 3. framework

共通 crate（Rust）：
- `k1s0-config`: 設定読み込み（`--env` / `--config` / `--secrets-dir`）
- `k1s0-observability`: ログ/トレース/メトリクスの初期化
- `k1s0-error`: エラー表現の統一
- `k1s0-grpc-client`: gRPC クライアント共通（deadline 必須、retry 原則禁止）

共通サービス：
- `auth-service`: 認証・認可
- `config-service`: 動的設定（`fw_m_setting`）
- `endpoint-service`: エンドポイント情報管理

#### 4. 規約 lint

- manifest（`.k1s0/manifest.json`）の存在・整合性
- 必須ファイル/ディレクトリの存在検査
- 禁止事項の検出
  - 環境変数参照（Rust: `std::env`、Go: `os.Getenv` 等）
  - ConfigMap への機密値直書き（`*_file` 参照のみ許可）
- 依存方向の逸脱検知（domain → infrastructure 禁止等）

#### 5. 契約管理の枠

- gRPC: `proto/` + `buf.yaml` / `buf.lock` + `buf lint` / `buf breaking`
- REST: `openapi/openapi.yaml` + 差分検知ツールの導線
- 生成一致チェック: 正本（proto/openapi）から生成物の再現性を CI で検証

#### 6. CI 必須チェック

- `k1s0 lint` を必須チェックとして実行
- 契約 lint/breaking（buf / OpenAPI diff）を必須チェックへ追加

### 対象外（本プランでは作成しないもの）

以下は本実装計画のスコープ外とする：

#### 1. 開発環境構築

- Dev Container / Docker Compose の整備
- ローカル依存起動スクリプト（`dev-up.ps1` 等）の実装
- 依存サービス（PostgreSQL / Redis / OTel Collector 等）の起動・シード

#### 2. Kubernetes 運用

- クラスタ/namespace の払い出し
- Service Mesh（Istio / Linkerd）の導入・設定
- Secret 配布基盤（External Secrets Operator 等）の構築

#### 3. Observability スタック

- OTel Collector の実環境構築
- Trace UI（Jaeger / Zipkin）の構築
- Grafana / Prometheus の構築

### 完了条件（全体）

本プラン全体の完了条件を以下とする：

1. **雛形生成と lint の成立**
   - `k1s0 new-feature` でサービス雛形を生成でき、生成直後に `k1s0 lint` が通る

2. **規約逸脱の PR 検知**
   - 必須ファイル削除、禁止事項（環境変数参照等）、依存方向違反が PR 時点で検知される

3. **契約管理の枠の成立**
   - gRPC: `buf lint` / `buf breaking` の枠があり、CI で必須化できる
   - REST: OpenAPI の破壊的変更検知の導線があり、CI で必須化できる

4. **upgrade の安全性**
   - `.k1s0/manifest.json` により、テンプレート更新が安全に差分提示・限定適用できる

## 帰結

### 正の帰結

- 実装スコープが明確になり、各フェーズで「何を作り、何を作らないか」が判断しやすくなる
- 対象外を明示することで、運用基盤の整備と混同せず、コード/テンプレ/CLI に集中できる
- 完了条件を先に固定することで、MVP の到達点が明確になる

### 負の帰結

- 開発環境（Dev Container / Docker Compose）が未整備のまま進むため、実際の動作確認は別途対応が必要
- Observability の実環境がないため、テンプレで出力設定だけ固定し、動作検証は後回しになる

### リスクと軽減策

| リスク | 軽減策 |
|--------|--------|
| 開発環境なしでテンプレ動作確認が困難 | 最小限のローカル実行手順を README に記載 |
| Observability 出力の検証遅延 | 出力フォーマットを先に固定し、単体テストで検証 |

## 関連ドキュメント

- [work/構想.md](../../work/構想.md): k1s0 の全体方針
- [work/プラン.md](../../work/プラン.md): 実装計画（フェーズ0〜34）
