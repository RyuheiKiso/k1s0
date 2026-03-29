# CI未対応サービス一覧（H-02 監査対応）

## 背景

外部技術監査（H-02）の指摘により、個別CI（`*-ci.yaml`）が設定されていないサービスが20件確認された。
現在、`.github/workflows/ci.yaml`（汎用CIオーケストレーター）はカバーしているが、
パス変更検出・専用ビルドキャッシュ・サービス固有のテスト設定が行えていない状態にある。

## CI未設定サービス一覧

| # | サービス名 | 種別 | パス | 備考 |
|---|-----------|------|------|------|
| 1 | api-registry | Rust サーバー | `regions/system/server/rust/api-registry/` | スキーマレジストリ管理 |
| 2 | event-monitor | Rust サーバー | `regions/system/server/rust/event-monitor/` | イベント監視・アラート |
| 3 | event-store | Rust サーバー | `regions/system/server/rust/event-store/` | イベントソーシングストア |
| 4 | featureflag | Rust サーバー | `regions/system/server/rust/featureflag/` | フィーチャーフラグ管理 |
| 5 | file | Rust サーバー | `regions/system/server/rust/file/` | ファイルストレージサービス |
| 6 | graphql-gateway | Rust サーバー | `regions/system/server/rust/graphql-gateway/` | GraphQL API Gateway |
| 7 | master-maintenance | Rust サーバー | `regions/system/server/rust/master-maintenance/` | マスタデータ管理 |
| 8 | navigation | Rust サーバー | `regions/system/server/rust/navigation/` | ナビゲーション設定配信 |
| 9 | notification | Rust サーバー | `regions/system/server/rust/notification/` | プッシュ通知配信 |
| 10 | policy | Rust サーバー | `regions/system/server/rust/policy/` | ポリシー管理 |
| 11 | quota | Rust サーバー | `regions/system/server/rust/quota/` | クォータ制御 |
| 12 | ratelimit | Rust サーバー | `regions/system/server/rust/ratelimit/` | レートリミット管理 |
| 13 | rule-engine | Rust サーバー | `regions/system/server/rust/rule-engine/` | ルールエンジン |
| 14 | scheduler | Rust サーバー | `regions/system/server/rust/scheduler/` | スケジューラ |
| 15 | search | Rust サーバー | `regions/system/server/rust/search/` | 全文検索サービス |
| 16 | service-catalog | Rust サーバー | `regions/system/server/rust/service-catalog/` | サービスカタログ管理 |
| 17 | session | Rust サーバー | `regions/system/server/rust/session/` | セッション管理 |
| 18 | tenant | Rust サーバー | `regions/system/server/rust/tenant/` | マルチテナント管理 |
| 19 | vault-rust | Rust サーバー | `regions/system/server/rust/vault/` | シークレット管理 |
| 20 | workflow | Rust サーバー | `regions/system/server/rust/workflow/` | ワークフロー管理 |

## CI未設定の理由・背景

### 開発初期フェーズによる工数制約

k1s0 プロジェクトは多数のサービスを同時開発しているモノリポ構成であり、
初期開発フェーズでは汎用 CI（`ci.yaml`）による一括ビルド・テストを優先した。
個別 CI の整備は後回しになり、上記20サービスに CI が設定されていない状態が継続している。

### 現在の汎用 CI によるカバレッジ

- `ci.yaml` が PR の全パスを対象にビルド・リント・テストを実行する
- ただし `paths-ignore` は個別 CI のあるサービスのみを除外しており、
  未対応サービスの変更も `ci.yaml` で検出される
- サービス固有の Dockerfile ビルドや統合テストは実施されていない

### リスク

- サービス固有の設定変更がパス変更トリガーで検出されない場合がある
- Dockerfile の変更やサービス固有の依存更新がCI検証されずにマージされるリスク

## 対応計画（優先度付き）

### Priority 1: セキュリティ・コアインフラ（2026-Q2 目標）

以下のサービスは認証・データ整合性への影響が大きいため最優先で対応する。

| サービス | 理由 |
|---------|------|
| vault-rust | シークレット管理。CI漏れは機密情報漏洩リスクに直結 |
| session | セッション管理。認証フローに影響 |
| tenant | マルチテナント分離。データ漏洩リスク |
| ratelimit | DDoS対策。CI漏れは本番障害リスク |

対応方法: `_rust-service-ci.yaml` の再利用可能ワークフローを使用して個別CIを追加する。

```yaml
# 例: .github/workflows/vault-rust-ci.yaml
name: vault-rust CI
on:
  pull_request:
    paths:
      - 'regions/system/server/rust/vault/**'
jobs:
  ci:
    uses: ./.github/workflows/_rust-service-ci.yaml
    with:
      service_path: regions/system/server/rust/vault
```

### Priority 2: 高頻度変更サービス（2026-Q3 目標）

変更頻度が高いサービスを対象に CI を整備する。

| サービス | 理由 |
|---------|------|
| graphql-gateway | フロントエンドとの接点。型安全性の検証が重要 |
| notification | 外部配信。メッセージ形式変更の検証が必要 |
| featureflag | 全サービスに影響するフラグ管理 |
| scheduler | 定期実行ジョブ。タイミング変更の影響が広範 |
| search | 全文検索インデックス設計に変更が多い |

### Priority 3: 残サービス（2026-Q4 目標）

上記以外の10サービスを順次対応する。

| 対象 |
|------|
| api-registry, event-monitor, event-store, file |
| master-maintenance, navigation, policy, quota |
| rule-engine, service-catalog, workflow |

## 参考

- [CI/CD設計.md](CI-CD設計.md)
- `.github/workflows/_rust-service-ci.yaml`（再利用可能 Rust CI テンプレート）
- `.github/workflows/auth-ci.yaml`（個別CI実装の参考例）
- 外部技術監査報告書 H-02: "個別CI未設定のサービスが20件存在する"

## 更新履歴

| 日付 | 変更内容 | 変更者 |
|------|---------|--------|
| 2026-03-28 | 初版作成（H-02 監査対応） | 監査対応チーム |
