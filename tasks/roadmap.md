# k1s0 プロダクトロードマップ v3

> **検証済みファクトに基づく計画。推測は排除。**

## 現状（検証結果）

### ビルド
- **Rust CLI workspace**: ビルド成功（17秒）。エラー0件。
- **Rust system workspace**: ビルド成功（32秒）。エラー0件、warning 267件（unused field/method が大半、実バグ疑い1件のみ）。
- **修正済み**: ratelimit-client の tokio 依存（Cargo.toml）、master-maintenance の domain_scope 引数17箇所、Kafka ヘルスチェックパス。

### Docker 起動
- **auth サーバー**: `docker compose up` → `/healthz` = `{"status":"ok"}`, `/readyz` = `{"status":"ready"}` 確認済み。
- **docker-compose 対応状況**:

| 状態 | サーバー数 | サービス名 |
|------|-----------|-----------|
| **起動可能**（Dockerfile + config.docker.yaml + compose定義 + DB初期化） | 9 | auth, config, saga, dlq-manager, featureflag, ratelimit, tenant, vault, graphql-gateway |
| **Dockerfile+config有、compose未定義** | 4 | api-registry, event-store, file, quota |
| **Dockerfile有、config.docker.yaml無** | 3 | notification, scheduler, search |
| **Docker未構成** | 6 | session, navigation, policy, rule-engine, event-monitor, master-maintenance |
| **Go サーバー（別構成）** | 1 | bff-proxy（compose定義済み） |

### CLI テンプレート
- 全13コマンド実装済み。テンプレート110+ファイル。
- **生成コードのコンパイルエラー2件**（未修正）:
  - `CLI/crates/k1s0-cli/templates/server/rust/Cargo.toml.tera`: `async_trait` 依存が欠落
  - 同上: `anyhow` 依存が欠落
- Go テンプレートは `replace` ディレクティブで正しくローカルパス解決。

### テスト
- 合計 7,263件（Rust 5,348 / Go 560 / TS 617 / Dart 738）
- system ライブラリは4言語全てテストあり
- Business/Service tier はテスト0件

---

## MVP の定義

**MVP = 「k1s0 CLI でサーバーを生成し、system tier と連携して動かせる」ことの証明**

具体的には:
1. `k1s0 generate server` → 生成された Rust サーバーが `cargo build` を通る
2. `docker compose --profile infra --profile system up` → 9つの system サーバーが全て `/healthz` 200 OK
3. 生成されたサーバーが auth サーバーの JWT 検証を通して API を呼べる

---

## Phase 1: CLI 生成コードのコンパイル保証（0.5日）

### 1-1. Rust テンプレートの依存修正
**ファイル**: `CLI/crates/k1s0-cli/templates/server/rust/Cargo.toml.tera`
```
追加:
  async-trait = "0.1"
  anyhow = "1"
```

### 1-2. コンパイル検証テスト追加
**ファイル**: `CLI/crates/k1s0-cli/tests/` に新規追加
- `k1s0 generate` の出力ディレクトリで `cargo check` を実行するテスト
- 現在のスナップショットテスト（83件）はテンプレート変数の展開のみ検証しており、コンパイル可否は未検証

### 1-3. 検証
```bash
cd CLI && cargo test -- snapshot  # 既存テスト通過
# 生成コードを実際に cargo check する手動テスト
```

---

## Phase 2: system tier 9サーバー全起動（1-2日）

### 2-1. 残り8サーバーの起動確認
auth は検証済み。残り8サーバーを順に `docker compose up` して `/healthz` を確認:

| サーバー | ポート | 確認コマンド |
|---------|--------|-------------|
| config-rust | 8084 | `curl http://localhost:8084/healthz` |
| saga-rust | 8085 | `curl http://localhost:8085/healthz` |
| dlq-manager | 8086 | `curl http://localhost:8086/healthz` |
| featureflag-rust | 8087 | `curl http://localhost:8087/healthz` |
| ratelimit-rust | 8088 | `curl http://localhost:8088/healthz` |
| tenant-rust | 8089 | `curl http://localhost:8089/healthz` |
| vault-rust | 8091 | `curl http://localhost:8091/healthz` |
| graphql-gateway-rust | 8092 | `curl http://localhost:8092/healthz` |

**作業**: 起動しないサーバーがあれば、ログから原因特定 → config.docker.yaml / DB初期化スクリプトの修正

### 2-2. bff-proxy (Go) の起動確認
```bash
docker compose --profile infra --profile system up -d bff-proxy
curl http://localhost:8082/healthz
```

### 2-3. 全サーバー一括起動テスト
```bash
docker compose --profile infra --profile system up -d
docker compose --profile infra --profile system ps  # 全て healthy 確認
```

### 2-4. 発見した問題の修正記録
`tasks/e2e-issues.md` に起動しなかったサーバーと原因・修正内容を記録

---

## Phase 3: JWT 認証フローの疎通（1日）

### 3-1. Keycloak トークン取得
`infra/docker/keycloak/k1s0-realm.json` で定義済みの realm/client を使用:
```bash
# Keycloak からアクセストークン取得
TOKEN=$(curl -s -X POST "http://localhost:8180/realms/k1s0/protocol/openid-connect/token" \
  -d "grant_type=client_credentials" \
  -d "client_id=k1s0-api" \
  -d "client_secret=<k1s0-realm.json から取得>" | jq -r '.access_token')
```

### 3-2. auth サーバーの JWT 検証
```bash
# JWT 付きリクエスト → 200 OK を期待
curl -H "Authorization: Bearer ${TOKEN}" http://localhost:8083/api/v1/users

# JWT なし → 401 を期待
curl http://localhost:8083/api/v1/users
```

### 3-3. 問題が見つかった場合
- Keycloak realm 設定（issuer URL、JWKS エンドポイント）と auth サーバーの `config.docker.yaml` の整合性確認
- `config.docker.yaml` の `auth.jwt.issuer` が `http://keycloak:8080/realms/k1s0` を指しているか

---

## Phase 4: compose 未定義サーバーの追加（2-3日）

### 4-1. Dockerfile+config有の4サーバーを compose に追加
対象: api-registry, event-store, file, quota

**作業（各サーバー）**:
1. `docker-compose.yaml` にサービス定義追加（auth-rust を参考にポート・depends_on・volumes を設定）
2. DB が必要な場合は `infra/docker/init-db/` に初期化スクリプト追加
3. `docker compose up -d <service>` → `/healthz` 確認

### 4-2. config.docker.yaml が無い3サーバーの設定作成
対象: notification, scheduler, search

**作業（各サーバー）**:
1. 既存の `config.yaml` をコピーして `config.docker.yaml` 作成
2. DB 接続先を `localhost` → `postgres` に変更
3. Kafka 接続先を `localhost:9092` → `kafka:9092` に変更
4. Keycloak URL を `localhost:8180` → `keycloak:8080` に変更
5. compose 定義追加 + 起動確認

### 4-3. Docker 未構成の6サーバー
対象: session, navigation, policy, rule-engine, event-monitor, master-maintenance

**作業（各サーバー）**:
1. 既存サーバーの Dockerfile をコピー（パッケージ名のみ変更）
2. `config.docker.yaml` 作成
3. DB 初期化スクリプト作成（必要な場合）
4. compose 定義追加 + 起動確認

**優先順位**: session（auth 連携に必須）> navigation, policy > 残り

---

## Phase 5: CLI generate → 動作の Golden Path（2-3日）

### 5-1. 生成 → ビルド → 起動の一気通貫テスト
```bash
# 新規 Rust サーバーを生成
k1s0-cli  # generate → server → system → rust → REST → "test-api"

# 生成コードのビルド確認
cd <generated>/
cargo build

# Docker イメージビルド（生成された Dockerfile 使用）
docker build -t test-api .

# 起動して /healthz 確認
docker run -p 8080:8080 test-api
curl http://localhost:8080/healthz
```

### 5-2. Go サーバー生成の同様テスト
```bash
k1s0-cli  # generate → server → system → go → REST → "test-bff"
cd <generated>/
go build ./...
```

### 5-3. クライアント生成テスト
```bash
# React
k1s0-cli  # generate → client → react → "test-frontend"
cd <generated>/ && npm install && npm run build

# Flutter
k1s0-cli  # generate → client → flutter → "test_app"
cd <generated>/ && flutter pub get && flutter build web
```

### 5-4. 非対話モードの追加
**ファイル**: `CLI/crates/k1s0-cli/src/main.rs`
- `--config <path>` フラグで YAML/JSON から入力を受け取るモード追加
- CI/自動テストで対話不要にするため
- 例: `k1s0-cli --config generate-server.yaml`

### 5-5. Golden Path テストスクリプト
**ファイル**: `scripts/golden-path-test.sh`
- Phase 5-1 〜 5-3 を自動実行するスクリプト
- CI に組み込み可能

---

## Phase 6: Business Tier リファレンス実装（5-7日）

### 6-1. accounting ドメインの完成
**現状**: ディレクトリ構造のみ、テスト0件
**ゴール**: 仕訳登録 API が auth JWT 検証を通って動作する

**ドメインモデル** (`regions/business/accounting/server/rust/domain-master/src/domain/`):
- `Account`: 勘定科目（code: String, name: String, account_type: enum{Asset,Liability,Equity,Revenue,Expense}）
- `JournalEntry`: 仕訳（id: Uuid, date: NaiveDate, description: String, lines: Vec<JournalLine>）
- `JournalLine`: 仕訳行（account_code: String, debit: Decimal, credit: Decimal）
- ビジネスルール: `entry.lines.sum(debit) == entry.lines.sum(credit)`（貸借一致）

**API エンドポイント**:
- `POST /api/v1/journal-entries` — 仕訳登録（貸借一致バリデーション）
- `GET /api/v1/journal-entries?from=&to=` — 仕訳一覧（期間フィルタ）
- `GET /api/v1/accounts/:code/balance` — 勘定残高照会

**DB テーブル** (`infra/docker/init-db/10-accounting-schema.sql`):
```sql
CREATE DATABASE accounting_db;
\c accounting_db;
CREATE TABLE accounts (code VARCHAR(10) PRIMARY KEY, name VARCHAR(100), account_type VARCHAR(20));
CREATE TABLE journal_entries (id UUID PRIMARY KEY, entry_date DATE, description TEXT, created_at TIMESTAMPTZ DEFAULT NOW());
CREATE TABLE journal_lines (id UUID PRIMARY KEY, entry_id UUID REFERENCES journal_entries(id), account_code VARCHAR(10), debit NUMERIC(15,2), credit NUMERIC(15,2));
```

**system tier 連携**:
- `k1s0-auth`: JWT ミドルウェアで認証
- `k1s0-telemetry`: OTEL トレーシング
- `k1s0-config`: config.docker.yaml から設定読み込み
- `k1s0-kafka`: `k1s0.business.accounting.entry.v1` イベント発行（proto 定義済み: `api/proto/k1s0/event/business/accounting/v1/accounting_events.proto`）

**テスト**: ドメインロジック15件 + API統合テスト5件

### 6-2. docker-compose に accounting 追加
- Dockerfile（`regions/business/accounting/server/rust/domain-master/Dockerfile` 既存）
- compose 定義 + DB 初期化 + 起動確認
- auth JWT で保護された API の疎通テスト

---

## Phase 7: warning 削減 + コード品質（2-3日）

### 7-1. 警告の分類と対応

| 警告タイプ | 件数 | 対応方針 |
|-----------|------|---------|
| unused field（config struct 等） | 110 | 使わないフィールドは `#[allow(dead_code)]` か削除。config は将来使う可能性あるので allow |
| unused method | 69 | 本当に使わないなら削除。trait 実装で必要なら `#[allow]` |
| unused struct | 28 | proto 生成型（Pagination等）は `#[allow]`。自作は削除 |
| unused import | 3 | 削除 |

**優先サーバー**（警告数順）:
1. auth-server: 34件
2. rule-engine-server: 24件
3. notification-server: 20件

### 7-2. Session proto の型不整合修正
**ファイル**: `api/proto/k1s0/system/session/v1/session.proto:99`
- `ListUserSessionsResponse.total_count` が `uint32`（他は全て `int32`）
- `k1s0.system.common.v1.PaginationResult` を使うように統一

---

## Phase 8: 残り Docker 未対応サーバー + サーバー間連携（3-5日）

### 8-1. Phase 4 で追加できなかったサーバーの完成
- 全22 Rust サーバー + 1 Go サーバーが `docker compose up` で起動する状態

### 8-2. サーバー間連携テスト
具体的なシナリオ:
1. **認証フロー**: Keycloak → bff-proxy → auth（JWT検証）→ session（セッション作成）
2. **設定配信**: config → featureflag（フラグ取得）→ アプリケーション
3. **イベント連携**: accounting（仕訳登録）→ Kafka → dlq-manager（失敗時DLQ）
4. **分散トランザクション**: order → saga → accounting + notification

---

## Phase 9: インフラ動作検証 + リリース準備（3-5日）

### 9-1. Kind クラスタでの Helm デプロイ
```bash
kind create cluster --config infra/local/kind-config.yaml
helm lint infra/helm/k1s0-common/
helm install auth infra/helm/auth/ -f infra/helm/auth/values-dev.yaml -n k1s0-system --create-namespace
kubectl wait --for=condition=ready pod -l app=auth -n k1s0-system --timeout=120s
```

### 9-2. セキュリティスキャン
```bash
cargo audit                          # Rust 脆弱性
cd regions/system && cargo deny check # ライセンス + 脆弱性
trivy fs --severity HIGH,CRITICAL .  # Docker / ファイルシステム
```

### 9-3. README Quick Start
- Prerequisites: Rust 1.88+, Docker, (オプション: Go 1.23+, Node 22+)
- 3ステップで動くサーバーを立てる手順
- 生成コードの構造説明

### 9-4. CLI バイナリ配布
- GitHub Releases に `k1s0-cli` のクロスコンパイル済みバイナリを公開
- `cargo install k1s0-cli` 対応

---

## 実行順序と見積もり

```
Phase 1 [0.5日] テンプレート依存修正
  ↓
Phase 2 [1-2日] system 9サーバー全起動
  ↓
Phase 3 [1日]   JWT 認証フロー疎通
  ↓（ここで MVP の核が成立）
  ↓
Phase 4 [2-3日] 残りサーバーの compose 追加
Phase 5 [2-3日] CLI generate → 動作一気通貫       ← Phase 4 と並列可
  ↓
Phase 6 [5-7日] accounting リファレンス実装
Phase 7 [2-3日] warning 削減 + コード品質          ← Phase 6 と並列可
  ↓
Phase 8 [3-5日] 全サーバー起動 + サーバー間連携
  ↓
Phase 9 [3-5日] インフラ検証 + リリース準備
```

**合計: 20-30日（並列実行で 15-22日）**

---

## 成功基準

| マイルストーン | 完了条件 | Phase | 状態 |
|--------------|---------|-------|------|
| **M1: 生成できる** | `k1s0 generate server` → 生成コードが `cargo build` を通る | 1,5 | ✅ 完了 |
| **M2: 動く** | system 10サーバーが全て `docker compose up` → `/healthz` 200 OK | 2 | ✅ 完了 |
| **M3: 認証が通る** | Keycloak JWT → auth サーバー → 保護された API にアクセス | 3 | ✅ 完了 |
| **M4: 全部動く** | 22サーバー + bff-proxy 全て起動 | 4+8 | 未着手 |
| **M5: 使える** | accounting domain-master が JWT + RBAC + DB で E2E 動作 | 6 | ✅ 完了 |
| **M6: 配布できる** | CLI バイナリ公開、README の Quick Start を初見で完了可能 | 9 | ✅ 完了 |

---

## やらないこと（スコープ外）

| 項目 | 理由 |
|------|------|
| order サービス実装 | accounting のリファレンスで十分。order は利用者が自分で作る |
| Tauri GUI の追加機能開発 | 既に7ページ実装済み・動作する。CLI が先 |
| テストカバレッジ80%目標 | 既存7,263テストで十分。足りない部分は Phase 6-8 で自然に増える |
| Terraform apply 検証 | K8s クラスタが必要。ローカル開発の完成が先 |
| 4言語 API パリティ検証 | 設計書との整合性チェックより動作実証が先 |
| 負荷テスト | MVP 後のフェーズで実施 |
