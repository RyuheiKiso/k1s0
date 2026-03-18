# system-navigation-server 実装設計

> **注記**: 本ドキュメントは navigation-server の実装仕様を含む。共通パターンは [Rust共通実装.md](../_common/Rust共通実装.md) を参照。

system-navigation-server（ナビゲーションサーバー）の Rust 実装仕様。概要・API 定義・アーキテクチャは [server.md](server.md) を参照。

---

## アーキテクチャ概要

Clean Architecture に基づく 4 層構成を採用する。

| レイヤー | 責務 | 依存方向 |
|---------|------|---------|
| domain | エンティティ・ドメインサービス | なし（最内層） |
| usecase | ビジネスロジック（ナビゲーション取得・フィルタリング） | domain のみ |
| adapter | REST/gRPC ハンドラー・ミドルウェア・プレゼンテーション | usecase, domain |
| infrastructure | 設定・ナビゲーション定義ローダー・起動シーケンス | 全レイヤー |

---

## Rust 実装 (regions/system/server/rust/navigation/)

### ディレクトリ構成

```
regions/system/server/rust/navigation/
├── src/
│   ├── main.rs                              # エントリポイント（startup::run() 委譲）
│   ├── domain/
│   │   ├── mod.rs
│   │   ├── entity/
│   │   │   ├── mod.rs
│   │   │   └── navigation.rs               # Route / RouteGuard / NavigationConfig エンティティ
│   │   └── service/
│   │       ├── mod.rs
│   │       └── navigation_filter.rs         # ロールベースルートフィルタリング
│   ├── usecase/
│   │   ├── mod.rs
│   │   └── get_navigation.rs               # ナビゲーション設定取得（ロールフィルタ付き）
│   ├── adapter/
│   │   ├── mod.rs
│   │   ├── handler/
│   │   │   ├── mod.rs
│   │   │   ├── navigation_handler.rs        # axum REST ハンドラー
│   │   │   └── health.rs                    # ヘルスチェック
│   │   ├── grpc/
│   │   │   ├── mod.rs
│   │   │   ├── navigation_grpc.rs           # gRPC サービス実装
│   │   │   └── tonic_service.rs             # tonic サービスラッパー
│   │   ├── middleware/
│   │   │   ├── mod.rs
│   │   │   ├── auth.rs                      # JWT 認証ミドルウェア
│   │   │   └── rbac.rs                      # RBAC ミドルウェア
│   │   └── presentation.rs                  # レスポンス変換
│   ├── infrastructure/
│   │   ├── mod.rs
│   │   ├── config.rs                        # 設定構造体・読み込み
│   │   ├── navigation_loader.rs             # ナビゲーション定義ファイルローダー
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

- **NavigationFilter**: 認証済みユーザーのロールに基づいてルート定義をフィルタリングする。`guard.roles` ベースでルート公開判定を行う。未認証時は公開ルートのみ返す

#### ユースケース

| ユースケース | 責務 |
|------------|------|
| `GetNavigationUseCase` | Bearer トークンからロールを抽出し、フィルタ済みナビゲーション設定を返す。トークン省略時は公開ルートのみ |

#### ナビゲーション定義

- ナビゲーション定義は設定ファイルから `NavigationLoader` でロードする
- 階層ルーティング（`children`）をサポートした再帰的なルート構造
- ルートガード: AUTH_REQUIRED / ROLE_REQUIRED / REDIRECT_IF_AUTHENTICATED の 3 種
- ページ遷移アニメーション: Fade / Slide / Modal

### エラーハンドリング方針

- ユースケース層で `anyhow::Result` を返却し、adapter 層で HTTP/gRPC ステータスコードに変換する
- 無効トークンは 401 Unauthorized を返す

### テスト方針

| テスト種別 | 対象 | 方針 |
|-----------|------|------|
| 単体テスト | ルートフィルタリング | テスト用ナビゲーション定義でロール別フィルタを検証 |
| 統合テスト | REST/gRPC ハンドラー | axum-test / tonic テストクライアント |

---

## 関連ドキュメント

- [server.md](server.md) -- 概要・API 定義・ルートガード設計
- [Rust共通実装.md](../_common/Rust共通実装.md) -- 共通起動シーケンス・Cargo 依存
