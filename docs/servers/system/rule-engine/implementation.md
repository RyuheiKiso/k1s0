# system-rule-engine-server 実装設計

> **注記**: 本ドキュメントは rule-engine-server の実装仕様を含む。共通パターンは [Rust共通実装.md](../../_common/Rust共通実装.md) を参照。

system-rule-engine-server（ルールエンジンサーバー）の Rust 実装仕様。概要・API 定義・アーキテクチャは [server.md](server.md) を参照。

---

## アーキテクチャ概要

Clean Architecture に基づく 4 層構成を採用する。

| レイヤー | 責務 | 依存方向 |
|---------|------|---------|
| domain | エンティティ・リポジトリトレイト・ドメインサービス | なし（最内層） |
| usecase | ビジネスロジック（ルール管理・ルールセット管理・評価・バージョン管理） | domain のみ |
| adapter | REST/gRPC ハンドラー・ミドルウェア・リポジトリ実装 | usecase, domain |
| infrastructure | 設定・DB接続・キャッシュ・Kafka・起動シーケンス | 全レイヤー |

---

## Rust 実装 (regions/system/server/rust/rule-engine/)

### ディレクトリ構成

```
regions/system/server/rust/rule-engine/
├── src/
│   ├── main.rs                              # エントリポイント（startup::run() 委譲）
│   ├── domain/
│   │   ├── mod.rs
│   │   ├── entity/
│   │   │   ├── mod.rs
│   │   │   ├── rule.rs                      # Rule エンティティ（条件式・アクション・優先度）
│   │   │   └── condition.rs                 # Condition エンティティ（条件式 AST）
│   │   ├── repository/
│   │   │   ├── mod.rs
│   │   │   ├── rule_repository.rs           # RuleRepository トレイト
│   │   │   ├── rule_set_repository.rs       # RuleSetRepository トレイト
│   │   │   ├── rule_set_version_repository.rs # RuleSetVersionRepository トレイト
│   │   │   └── evaluation_log_repository.rs # EvaluationLogRepository トレイト
│   │   └── service/
│   │       ├── mod.rs
│   │       ├── condition_parser.rs          # 条件式パーサー（文字列 → AST）
│   │       └── condition_evaluator.rs       # 条件式評価（AST → 真偽値）
│   ├── usecase/
│   │   ├── mod.rs
│   │   ├── create_rule.rs                   # ルール作成
│   │   ├── update_rule.rs                   # ルール更新
│   │   ├── delete_rule.rs                   # ルール削除
│   │   ├── get_rule.rs                      # ルール取得
│   │   ├── list_rules.rs                    # ルール一覧
│   │   ├── create_rule_set.rs               # ルールセット作成
│   │   ├── update_rule_set.rs               # ルールセット更新
│   │   ├── delete_rule_set.rs               # ルールセット削除
│   │   ├── get_rule_set.rs                  # ルールセット取得
│   │   ├── list_rule_sets.rs                # ルールセット一覧
│   │   ├── publish_rule_set.rs              # ルールセット公開（バージョン作成）
│   │   ├── rollback_rule_set.rs             # ルールセットロールバック
│   │   ├── evaluate.rs                      # ルール評価（first-match / all-match）
│   │   └── list_evaluation_logs.rs          # 評価監査ログ一覧
│   ├── adapter/
│   │   ├── mod.rs
│   │   ├── handler/
│   │   │   ├── mod.rs
│   │   │   ├── rule_handler.rs              # axum REST ハンドラー
│   │   │   └── health.rs                    # ヘルスチェック
│   │   ├── grpc/
│   │   │   ├── mod.rs
│   │   │   ├── rule_engine_grpc.rs          # gRPC サービス実装
│   │   │   └── tonic_service.rs             # tonic サービスラッパー
│   │   ├── middleware/
│   │   │   ├── mod.rs
│   │   │   ├── auth.rs                      # JWT 認証ミドルウェア
│   │   │   └── rbac.rs                      # RBAC ミドルウェア
│   │   └── repository/
│   │       ├── mod.rs
│   │       ├── rule_postgres.rs             # RuleRepository PostgreSQL 実装
│   │       ├── rule_set_postgres.rs         # RuleSetRepository PostgreSQL 実装
│   │       ├── rule_set_version_postgres.rs # RuleSetVersionRepository PostgreSQL 実装
│   │       └── evaluation_log_postgres.rs   # EvaluationLogRepository PostgreSQL 実装
│   ├── infrastructure/
│   │   ├── mod.rs
│   │   ├── config.rs                        # 設定構造体・読み込み
│   │   ├── database.rs                      # DB 接続プール
│   │   ├── cache.rs                         # moka キャッシュ（評価結果 TTL 60秒）
│   │   ├── kafka_producer.rs                # Kafka プロデューサー（ルール変更通知）
│   │   └── startup.rs                       # 起動シーケンス・DI
│   └── proto/                               # tonic-build 生成コード
├── config/
│   └── config.yaml
├── build.rs
├── Cargo.toml
└── Dockerfile
```

### 主要コンポーネント

#### ドメインサービス

- **ConditionParser**: JSON/YAML 形式の条件式を AST（Abstract Syntax Tree）にパースする。比較演算子・論理演算子・関数呼び出しをサポートする
- **ConditionEvaluator**: AST を入力データに対してインタプリタで評価し、真偽値を返す。OPA のような外部プロセスを使用しないため低レイテンシ

#### ユースケース

| ユースケース | 責務 |
|------------|------|
| `CreateRuleUseCase` 等 | ルール定義の CRUD |
| `CreateRuleSetUseCase` 等 | ルールセット（複数ルールのグループ）の CRUD |
| `PublishRuleSetUseCase` | ルールセットのバージョン公開 |
| `RollbackRuleSetUseCase` | 過去バージョンへのロールバック |
| `EvaluateUseCase` | ルール評価（first-match / all-match の 2 モード） |
| `ListEvaluationLogsUseCase` | 評価監査ログの一覧取得 |

#### policy-server との違い

- policy-server は Rego ベースの「アクセス制御ポリシー」（allow/deny）に特化する
- rule-engine は「業務判定ロジック」（税率計算、与信スコア、価格ティア判定等）に特化し、任意の構造化データを返却する

#### キャッシュ戦略

- moka で評価結果を TTL 60 秒キャッシュする
- Kafka `k1s0.system.rule_engine.rule_changed.v1` 通知受信時にキャッシュを即座に無効化する

### エラーハンドリング方針

- ユースケース層で `anyhow::Result` を返却し、adapter 層で HTTP/gRPC ステータスコードに変換する
- エラーコードプレフィックス: `SYS_RULE_`
- 条件式パースエラーは 400 Bad Request を返す

### テスト方針

| テスト種別 | 対象 | 方針 |
|-----------|------|------|
| 単体テスト | 条件式パーサー・評価エンジン | テスト用ルール定義で入出力を検証 |
| 統合テスト | REST/gRPC ハンドラー | axum-test / tonic テストクライアント |
| 評価ログテスト | 監査ログ記録 | mockall によるリポジトリモック |

---

## 関連ドキュメント

- [server.md](server.md) -- 概要・API 定義・ルール定義形式
- [Rust共通実装.md](../../_common/Rust共通実装.md) -- 共通起動シーケンス・Cargo 依存
