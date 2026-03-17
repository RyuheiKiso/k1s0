# セキュリティスキャンツール整備 (F-025)

本ドキュメントは、k1s0 プラットフォームで使用するセキュリティスキャンツールの構成・運用方針を定義する。

## 目次

- [概要](#概要)
- [SAST（静的アプリケーションセキュリティテスト）](#sast静的アプリケーションセキュリティテスト)
- [DAST（動的アプリケーションセキュリティテスト）](#dast動的アプリケーションセキュリティテスト)
- [コンテナスキャン](#コンテナスキャン)
- [依存関係スキャン](#依存関係スキャン)
- [CI 統合ポイント](#ci-統合ポイント)
- [インストール要件](#インストール要件)

---

## 概要

セキュリティスキャンは以下の4層で構成し、CI パイプラインの各段階に組み込む。

| 層 | 目的 | 主要ツール |
|---|---|---|
| SAST | ソースコードの脆弱性検出 | cargo-audit, gosec, npm audit, dart pub outdated |
| DAST | 実行中アプリケーションの脆弱性検出 | OWASP ZAP |
| コンテナスキャン | コンテナイメージの脆弱性検出 | Trivy |
| 依存関係スキャン | サードパーティライブラリの脆弱性検出 | 各言語ツール + Trivy |

---

## SAST（静的アプリケーションセキュリティテスト）

ソースコードおよびビルド成果物を対象に、既知の脆弱性パターンを静的に検出する。

### Rust: cargo-audit

Rust クレートの既知の脆弱性を RustSec Advisory Database に基づいて検出する。

```bash
# インストール
cargo install cargo-audit

# 全ワークスペースの監査を実行
cargo audit

# JSON 形式で結果を出力（CI パイプライン連携用）
cargo audit --json

# 特定の Advisory を無視する場合（正当な理由がある場合のみ）
cargo audit --ignore RUSTSEC-XXXX-XXXX
```

**検出対象:**
- 既知の CVE（Common Vulnerabilities and Exposures）
- yanked クレートの使用
- 非メンテナンスクレートの警告

**設定ファイル:** プロジェクトルートに `audit.toml` を配置して無視ルールを管理する。

```toml
# audit.toml — 正当な理由のある除外のみ記載
[advisories]
ignore = []
```

### Go: gosec

Go ソースコードのセキュリティ問題を検出する静的解析ツール。

```bash
# インストール
go install github.com/securego/gosec/v2/cmd/gosec@latest

# 全パッケージをスキャン
gosec ./...

# SARIF 形式で出力（GitHub Code Scanning 連携用）
gosec -fmt sarif -out results.sarif ./...

# 特定ルールの除外（正当な理由がある場合のみ）
gosec -exclude=G104 ./...
```

**主要な検出ルール:**

| ルール ID | 内容 |
|---|---|
| G101 | ハードコードされた認証情報 |
| G104 | エラーの未チェック |
| G201 | SQL インジェクション |
| G301 | ディレクトリ作成時の不適切なパーミッション |
| G401 | 脆弱な暗号アルゴリズムの使用 |
| G501 | ブラックリストされたインポート |

### TypeScript/Node.js: npm audit

npm パッケージの既知の脆弱性を npm Advisory Database に基づいて検出する。

```bash
# 脆弱性の確認
npm audit

# JSON 形式で結果を出力
npm audit --json

# 自動修正可能な脆弱性を修正
npm audit fix

# 本番依存のみを対象にする
npm audit --omit=dev

# 重大度でフィルタ（CI では high 以上を失敗条件とする）
npm audit --audit-level=high
```

**重大度レベルと対応方針:**

| 重大度 | CI での扱い | 対応期限 |
|---|---|---|
| critical | ビルド失敗 | 即時対応 |
| high | ビルド失敗 | 1 営業日以内 |
| moderate | 警告 | 次回スプリント |
| low | 情報 | バックログ |

### Dart: dart pub outdated

Dart パッケージの更新状況と非推奨パッケージを検出する。

```bash
# 依存パッケージの更新状況を確認
dart pub outdated

# JSON 形式で出力
dart pub outdated --json

# 解決可能な更新を表示
dart pub outdated --mode=null-safety

# 脆弱性チェック（Flutter 環境）
flutter pub outdated
```

**補足:** Dart/Flutter エコシステムでは `dart pub outdated` が主要な脆弱性検出手段となる。重大な脆弱性を含むパッケージは pub.dev 上で retracted（取り下げ）されるため、定期的な更新確認が重要。

---

## DAST（動的アプリケーションセキュリティテスト）

実行中のアプリケーションに対してリクエストを送信し、脆弱性を動的に検出する。

### OWASP ZAP (Zed Attack Proxy)

OWASP が提供するオープンソースの Web アプリケーションセキュリティスキャナー。

#### ベースラインスキャン

ステージング環境にデプロイされた API に対して、パッシブスキャンを実行する。

```bash
# Docker を使用したベースラインスキャン
docker run --rm -t ghcr.io/zaproxy/zaproxy:stable zap-baseline.py \
  -t https://staging-api.example.com \
  -r report.html \
  -J report.json \
  -c zap-baseline.conf
```

#### API スキャン

OpenAPI 仕様書を入力として、API エンドポイントに対するスキャンを実行する。

```bash
# OpenAPI 仕様を使用した API スキャン
docker run --rm -t ghcr.io/zaproxy/zaproxy:stable zap-api-scan.py \
  -t https://staging-api.example.com/openapi.json \
  -f openapi \
  -r api-report.html \
  -J api-report.json
```

#### フルスキャン

リリース前の包括的なセキュリティスキャン。アクティブスキャンを含むため実行時間が長い。

```bash
# フルスキャン（リリース前のみ実行）
docker run --rm -t ghcr.io/zaproxy/zaproxy:stable zap-full-scan.py \
  -t https://staging-api.example.com \
  -r full-report.html \
  -J full-report.json
```

**検出対象:**
- SQL インジェクション
- XSS（クロスサイトスクリプティング）
- CSRF（クロスサイトリクエストフォージェリ）
- セキュリティヘッダーの欠如
- 情報漏洩（エラーメッセージ、スタックトレース）

**実行タイミング:**

| スキャン種別 | 実行タイミング | 対象環境 |
|---|---|---|
| ベースライン | PR マージ後（staging デプロイ時） | staging |
| API スキャン | 日次（深夜バッチ） | staging |
| フルスキャン | リリース前 | staging |

---

## コンテナスキャン

### Trivy

Aqua Security が提供するコンテナイメージおよびファイルシステムの脆弱性スキャナー。

#### コンテナイメージスキャン

```bash
# コンテナイメージの脆弱性スキャン
trivy image --severity HIGH,CRITICAL k1s0-platform/auth:latest

# JSON 形式で結果を出力
trivy image --format json --output result.json k1s0-platform/auth:latest

# SARIF 形式で出力（GitHub Code Scanning 連携用）
trivy image --format sarif --output result.sarif k1s0-platform/auth:latest

# 修正済みバージョンが存在する脆弱性のみ表示
trivy image --ignore-unfixed k1s0-platform/auth:latest
```

#### ファイルシステムスキャン

```bash
# リポジトリ全体のスキャン（IaC、シークレット、ライセンスを含む）
trivy fs --scanners vuln,secret,misconfig .

# Terraform 設定のスキャン
trivy config --policy-bundle-repository ghcr.io/aquasecurity/trivy-policies ./infrastructure/terraform/

# Kubernetes マニフェストのスキャン
trivy config ./infrastructure/kubernetes/
```

#### シークレット検出

```bash
# リポジトリ内のシークレット（API キー、パスワード等）を検出
trivy fs --scanners secret .
```

**スキャン対象と重大度:**

| 対象 | スキャナ | CI での扱い |
|---|---|---|
| コンテナイメージ | vuln | HIGH/CRITICAL でビルド失敗 |
| IaC 設定 | misconfig | HIGH/CRITICAL で警告 |
| シークレット | secret | 検出時にビルド失敗 |
| ライセンス | license | 禁止ライセンス検出時に警告 |

---

## 依存関係スキャン

各言語の依存関係を横断的に管理し、脆弱なライブラリの使用を防止する。

### 言語別スキャンツール

| 言語 | ツール | 対象ファイル | スクリプト |
|---|---|---|---|
| Rust | cargo-audit | Cargo.toml / Cargo.lock | `scripts/security/cargo-audit.sh` |
| Go | govulncheck | go.mod / go.sum | `scripts/security/go-vulncheck.sh` |
| TypeScript | npm audit | package.json / package-lock.json | `scripts/security/npm-audit.sh` |
| Dart | dart pub outdated | pubspec.yaml / pubspec.lock | `scripts/security/dart-outdated.sh` |

### govulncheck（Go 専用補助ツール）

Go の脆弱性データベース（vuln.go.dev）に基づいて、実際に呼び出されている脆弱なコードパスを検出する。gosec が静的パターンマッチであるのに対し、govulncheck はコールグラフ解析を行う。

```bash
# インストール
go install golang.org/x/vuln/cmd/govulncheck@latest

# スキャン実行
govulncheck ./...

# JSON 形式で出力
govulncheck -format json ./...
```

### Trivy による横断スキャン

Trivy はコンテナスキャンに加え、ファイルシステムモードで全言語の依存関係を一括スキャンできる。

```bash
# 全言語の依存関係を一括スキャン
trivy fs --scanners vuln --severity HIGH,CRITICAL ./regions/
```

---

## CI 統合ポイント

### パイプライン構成

セキュリティスキャンは CI/CD パイプラインの以下のステージに組み込む。

```
PR 作成時       → SAST + 依存関係スキャン（高速、ブロッキング）
マージ後        → コンテナスキャン + ベースライン DAST
日次バッチ      → API スキャン + フルスキャン（非ブロッキング）
リリース前      → 全スキャン実行（ブロッキング）
```

### GitHub Actions ワークフロー例

#### PR 時の SAST チェック

```yaml
# .github/workflows/security-sast.yml
name: Security - SAST
on:
  pull_request:
    branches: [main]

jobs:
  rust-audit:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Rust セキュリティ監査
        run: |
          cargo install cargo-audit
          cargo audit --json > rust-audit.json
      - name: 結果をアップロード
        uses: actions/upload-artifact@v4
        with:
          name: rust-audit
          path: rust-audit.json

  go-security:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-go@v5
        with:
          go-version: '1.24'
      - name: Go セキュリティスキャン
        run: |
          go install github.com/securego/gosec/v2/cmd/gosec@latest
          gosec -fmt sarif -out gosec.sarif ./regions/system/library/go/...
      - name: SARIF アップロード
        uses: github/codeql-action/upload-sarif@v3
        with:
          sarif_file: gosec.sarif

  npm-audit:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: npm セキュリティ監査
        run: bash scripts/security/npm-audit.sh

  dart-outdated:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dart-lang/setup-dart@v1
      - name: Dart パッケージ監査
        run: bash scripts/security/dart-outdated.sh
```

#### コンテナイメージスキャン

```yaml
# .github/workflows/security-container.yml
name: Security - Container Scan
on:
  push:
    branches: [main]

jobs:
  trivy-scan:
    runs-on: ubuntu-latest
    strategy:
      matrix:
        image:
          - k1s0-platform/auth
          - k1s0-platform/tenant
          - k1s0-platform/payment
    steps:
      - uses: actions/checkout@v4
      - name: Trivy コンテナスキャン
        uses: aquasecurity/trivy-action@master
        with:
          image-ref: ${{ matrix.image }}:${{ github.sha }}
          format: sarif
          output: trivy-${{ matrix.image }}.sarif
          severity: HIGH,CRITICAL
      - name: SARIF アップロード
        uses: github/codeql-action/upload-sarif@v3
        with:
          sarif_file: trivy-${{ matrix.image }}.sarif
```

### スキャン結果の通知

スキャン結果は以下のチャネルで通知する。

| 重大度 | 通知先 | タイミング |
|---|---|---|
| CRITICAL | Slack #security-alerts + PagerDuty | 即時 |
| HIGH | Slack #security-alerts | スキャン完了時 |
| MODERATE | GitHub Issue 自動作成 | 日次集計 |
| LOW | ダッシュボード表示のみ | — |

---

## インストール要件

### 開発環境（ローカル）

開発者のローカル環境に導入するツールの一覧。

| ツール | バージョン | インストールコマンド |
|---|---|---|
| cargo-audit | >= 0.21 | `cargo install cargo-audit` |
| gosec | >= 2.21 | `go install github.com/securego/gosec/v2/cmd/gosec@latest` |
| govulncheck | latest | `go install golang.org/x/vuln/cmd/govulncheck@latest` |
| trivy | >= 0.58 | `brew install trivy` / `choco install trivy` |
| npm | >= 10.0 | Node.js に同梱 |
| dart | >= 3.6 | Dart SDK に同梱 |

### CI 環境

CI 環境では以下のツールを Docker イメージまたは GitHub Actions として利用する。

| ツール | CI での利用方法 |
|---|---|
| cargo-audit | `cargo install` でジョブ内インストール |
| gosec | `go install` でジョブ内インストール |
| govulncheck | `go install` でジョブ内インストール |
| trivy | `aquasecurity/trivy-action` Action |
| OWASP ZAP | `ghcr.io/zaproxy/zaproxy:stable` Docker イメージ |
| npm audit | Node.js セットアップ後に利用可能 |
| dart pub | Dart SDK セットアップ後に利用可能 |

### Docker イメージ

CI で使用する Docker イメージの一覧。

```dockerfile
# セキュリティスキャン用ベースイメージ
FROM aquasec/trivy:latest AS trivy
FROM ghcr.io/zaproxy/zaproxy:stable AS zap
```

### ネットワーク要件

スキャンツールが参照する外部データベース。CI 環境のファイアウォールで以下の通信を許可する。

| ツール | 接続先 | 用途 |
|---|---|---|
| cargo-audit | https://rustsec.org | RustSec Advisory Database |
| gosec / govulncheck | https://vuln.go.dev | Go Vulnerability Database |
| npm audit | https://registry.npmjs.org | npm Advisory Database |
| trivy | https://ghcr.io | 脆弱性データベース（OCI） |
| OWASP ZAP | 対象アプリケーション URL | 動的スキャン対象 |
