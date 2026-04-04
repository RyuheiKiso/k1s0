# ADR-0081: 外部技術監査 v4 対応（2026-04-04）

## ステータス

承認済み

## コンテキスト

外部プリンシパルエンジニアによる第三者技術監査（2026-04-04 実施）で 24件（CRIT 3, HIGH 6, MED 7, LOW 8）の指摘事項が報告された。総合評価 B（良好だが重大な修正必須事項あり）。

主要な問題は以下の通りであった:
1. **CRIT-001/002**: event-store/event-monitor の Rust コンパイルエラー（ソースビルド不可・本番デプロイ阻害）
2. **CRIT-003**: buf/validate proto のローカルビルド失敗（BSR 依存のため protoc 単体では解決不可）
3. **HIGH-002〜006**: stale Docker イメージ問題（ソース変更がコンテナに未反映）
4. **MED-001**: config サービスの暗号化が本番環境でも無効化可能（機密設定の平文保存リスク）
5. **MED-005**: ai-gateway/ai-agent の Vault 認証設定が未完了（本番環境でのシークレット取得不可）

## 決定

以下の技術的決定を行う:

### 1. CRIT-003: build.rs 生成済みファイル優先コピー戦略の全サービス標準化

event-store と event-monitor の `build.rs` に graphql-gateway（ADR-0079）と同様の「生成済みファイル優先コピー」戦略を採用する。

`api/proto/gen/rust/` に `buf generate` で生成済みの `.rs` ファイルが存在する場合は、それを `OUT_DIR` にコピーして使用する。存在しない場合のみ `tonic-build` によるオンデマンド生成にフォールバックする。

### 2. MED-001: 本番環境での config 暗号化を必須化

`config/src/infrastructure/startup.rs` において、`config_server.encryption.enabled = false` かつ `app.environment` が dev/development/local/test 以外の場合は `anyhow::bail!()` で起動を拒否する。

### 3. MED-005: ai-gateway/ai-agent の Vault 個別ロール化

ai-gateway/ai-agent の Vault ロールを削除済みの "system" 共有ロールから個別ロール（"ai-gateway", "ai-agent"）に移行する。ServiceAccount、auth.tf ロール、vault/auth/ リファレンスファイルを同時に追加する。

## 理由

### CRIT-001/002 の修正方針

- CRIT-001: タプル式内に `let` 文は記述できない（Rust 文法違反）。`let timestamp` をタプル外に移動するのみ。最小変更で解消。
- CRIT-002: event-monitor の AppState は設計上 DB プールを直接保持しない（ユースケース経由のみ）。`Option<Arc<sqlx::PgPool>>` として追加し、in-memory モード（DB なし）では `None` として readyz でスキップする（`auth` サービスの既存パターンに準拠）。

### CRIT-003 の修正方針

graphql-gateway の build.rs 戦略（ADR-0079）を event-store/event-monitor に展開する。一貫性のある標準アプローチ。

### MED-001 の修正方針

`info!()` ログのみでは運用者が見落とす可能性が高い。本番環境での起動拒否はセキュリティ要件として妥当。環境判定は `app.environment` の文字列比較で行う（env var ではなく config file で管理される値）。

### MED-005 の修正方針

ai-gateway/ai-agent は試験段階のサービスであるが、Helm values.yaml に `vault.enabled: true` が設定されている。削除済みの "system" ロールを参照しているため、本番化時に Vault 認証が即座に失敗する。事前に個別ロールを定義しておく必要がある。

## 影響

**ポジティブな影響**:
- event-store/event-monitor がソースからビルド可能になり、本番デプロイが回復する
- ローカル開発環境での proto ビルド警告が解消される
- config サービスの本番暗号化が強制化され、機密設定の平文保存リスクが排除される
- ai-gateway/ai-agent が本番環境で Vault からシークレットを取得できるようになる
- E2E テストのシークレット管理がセキュアになる（ディスク平文書き込みの排除）
- Rust カバレッジ閾値が他言語と統一される（65%→75%）

**ネガティブな影響・トレードオフ**:
- config サービスの本番環境では `CONFIG_ENCRYPTION_KEY` 設定が必須となる（運用負担増）
- ai-gateway/ai-agent の Vault ポリシー（`ai-gateway.hcl`, `ai-agent.hcl`）は別途作成が必要

## 代替案

| 案 | 概要 | 採用しなかった理由 |
|----|------|-----------------|
| CRIT-002: AppState にユースケース経由でDB確認 | `list_events_uc` で trivial クエリを実行 | 副作用（RLS、UC ロジック）が readyz に持ち込まれるため |
| CRIT-003: buf BSR をローカルにミラー | BSR の proto をローカルキャッシュ | CI/CD インフラへの依存が増加するため |
| MED-001: 環境変数チェック | 環境変数 `APP_ENV` で判定 | 既存の `app.environment` config 値を使う方が一貫性がある |

## 対応一覧

| ID | 分類 | 対応内容 | 対応ファイル |
|----|------|---------|------------|
| CRIT-001 | コード修正 | `let timestamp` をタプル外に移動 | `event-store/src/adapter/handler/health.rs` |
| CRIT-002 | コード修正 | AppState に `db_pool: Option<Arc<PgPool>>` 追加 | `event-monitor/src/adapter/handler/{mod,health}.rs`, `startup.rs` |
| CRIT-003 | コード修正 | build.rs を生成済みファイル優先コピー戦略に変更 | `event-store/build.rs`, `event-monitor/build.rs` |
| HIGH-001 | ドキュメント | stale container 再起動手順を追記 | `docs/infrastructure/devenv/troubleshooting.md` |
| HIGH-002 | 確認（対応済み） | graphql-gateway readyz はソースで修正済み | stale image 問題 |
| HIGH-003 | 確認（対応済み） | featureflag readyz はソースで実装済み | stale image 問題 |
| HIGH-004 | ドキュメント修正 | featureflag ポート 8087→8187 に統一 | `.env.example`, `docker-compose設計.md`, `featureflag/deploy.md` |
| HIGH-005 | ドキュメント | WSL2 ポートフォワーディング問題を記載 | `troubleshooting.md` |
| HIGH-006 | 間接解消 | CRIT-001/002 修正でコンテナリビルド可能になる | - |
| MED-001 | コード修正 | 本番環境での暗号化未設定時に起動拒否 | `config/src/infrastructure/startup.rs` |
| MED-002 | CI修正 | E2E テストのシークレットを env: ブロックで渡す | `.github/workflows/e2e.yaml` |
| MED-003 | CI修正 | Rust カバレッジ閾値を 65%→75% に統一 | `.github/workflows/_rust-service-ci.yaml` |
| MED-004 | 確認（対応済み） | graphql-gateway readyz は `/healthz` 使用（HIGH-002 と同一） | - |
| MED-005 | 設定追加 | ai-gateway/ai-agent の SA + Vault ロール追加 | `service-accounts.yaml`, `auth.tf`, `values.yaml×2`, `vault/auth×2` |
| MED-006 | CI修正 | gitleaks PR コメントを有効化 | `.github/workflows/security.yaml` |
| MED-007 | 設定追加 | 23サービスの config.prod.yaml/config.staging.yaml 作成 | `regions/system/server/rust/*/config/` |
| LOW-001 | ドキュメント | Git 設定推奨値を troubleshooting.md に記載 | `troubleshooting.md` |
| LOW-002 | ドキュメント | CLI PATH インストール手順を記載 | `troubleshooting.md` |
| LOW-003 | 間接解消 | HIGH-005 と同一問題（WSL2 ネットワーク） | - |
| LOW-004 | 対応不要 | bff-proxy docs は正しく Go と記載済み | - |
| LOW-005 | ドキュメント新規 | Keycloak Realm 同期手順書を作成 | `docs/operations/keycloak-realm-sync.md` |
| LOW-006 | 確認済み | ADR-0080 CRITICAL 3件は全実装済みを確認 | featureflag_postgres.rs, realm-k1s0.json, kafka/mod.rs |
| LOW-007 | ドキュメント | SESSION_ENCRYPTION_KEY ローテーション手順を追記 | `docs/servers/system/bff-proxy/server.md` |
| LOW-008 | 設定追加 | rule-engine の PrometheusRule/ServiceMonitor 追加 | `infra/observability/rule-engine/`, `rule-engine/values.yaml` |

## 参考

- [ADR-0080: 外部技術監査 v3 対応](0080-external-audit-remediation-2026-04-03-v3.md)
- [ADR-0079: graphql-gateway build.rs 生成済みファイル戦略](0079-graphql-gateway-build-strategy.md)
- [ADR-0045: Vault per-service roles](0045-vault-per-service-roles.md)
- [外部技術監査報告書（2026-04-04）](../../../報告書.md)

## 更新履歴

| 日付 | 変更内容 | 変更者 |
|------|---------|--------|
| 2026-04-04 | 初版作成（24件全対応） | @k1s0-dev |
