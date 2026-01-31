## ディレクトリ構成図

（詳細版。テンプレ/共通サービスの形を具体化するための参考）

```text
(理想形 / プレースホルダ: {feature_name}, {service_name}, {env})
(※ 各言語のビルド成果物（target/ node_modules/ build/ 等）は省略)

k1s0/
├─ .github/
│  └─ workflows/
│     ├─ ci.yaml
│     └─ release.yaml
├─ .vscode/
│  ├─ settings.json
│  └─ extensions.json
├─ .editorconfig
├─ README.md
├─ docs/
│  ├─ adr/
│  ├─ architecture/
│  ├─ operations/
│  └─ conventions/
├─ work/
│  ├─ プラン.md
│  └─ 構想.md
│
├─ scripts/                           # 開発補助（lint/test/dev等）
│  ├─ fmt.ps1
│  ├─ lint.ps1
│  ├─ test.ps1
│  ├─ dev-up.ps1
│  ├─ dev-down.ps1
│  ├─ dev-seed.ps1
│  └─ dev-check.ps1
│
├─ CLI/                               # 雛形生成・導入・アップグレード支援
│  ├─ crates/
│  │  ├─ k1s0-cli/                    # 実行CLI (clap)
│  │  │  └─ src/
│  │  │     ├─ commands/              # init / new-feature / upgrade ...
│  │  │     ├─ main.rs
│  │  │     └─ lib.rs
│  │  └─ k1s0-generator/              # テンプレ展開・差分適用ロジック
│  │     └─ src/
│  │        ├─ renderer/
│  │        ├─ diff/
│  │        └─ lib.rs
│  └─ templates/                      # 生成テンプレ群
│     ├─ backend-rust/
│     │  ├─ project/                  # リポジトリ初期化テンプレ（共通設定/CIなど）
│     │  └─ feature/                  # 機能（=1マイクロサービス）雛形
│     │     ├─ Cargo.toml
│     │     ├─ README.md
│     │     ├─ config/
│     │     │  ├─ default.yaml
│     │     │  ├─ dev.yaml
│     │     │  ├─ stg.yaml
│     │     │  └─ prod.yaml
│     │     ├─ openapi/
│     │     │  └─ openapi.yaml
│     │     ├─ deploy/
│     │     │  ├─ base/
│     │     │  └─ overlays/
│     │     │     ├─ dev/
│     │     │     ├─ stg/
│     │     │     └─ prod/
│     │     ├─ migrations/
│     │     │  └─ 0001_init.sql
│     │     └─ src/                   # Clean Architecture
│     │        ├─ application/
│     │        │  ├─ usecases/
│     │        │  ├─ services/
│     │        │  └─ mod.rs
│     │        ├─ domain/
│     │        │  ├─ entities/
│     │        │  ├─ value_objects/
│     │        │  ├─ repositories/     # traits (ports)
│     │        │  ├─ error.rs
│     │        │  └─ mod.rs
│     │        ├─ infrastructure/
│     │        │  ├─ db/               # repository implementations
│     │        │  ├─ cache/
│     │        │  ├─ messaging/
│     │        │  ├─ config/
│     │        │  ├─ logging/
│     │        │  └─ mod.rs
│     │        ├─ presentation/
│     │        │  ├─ http/             # axum routers/handlers/middlewares
│     │        │  ├─ grpc/
│     │        │  └─ mod.rs
│     │        ├─ error.rs
│     │        └─ main.rs
│     ├─ backend-go/
│     ├─ frontend-react/
│     └─ frontend-flutter/
│
├─ framework/                         # 開発基盤チームが整備する共通部品
│  ├─ backend/
│  │  ├─ rust/
│  │  │  ├─ crates/
│  │  │  │  ├─ k1s0-auth/
│  │  │  │  ├─ k1s0-logging/
│  │  │  │  ├─ k1s0-config/
│  │  │  │  ├─ k1s0-db/
│  │  │  │  ├─ k1s0-cache/
│  │  │  │  ├─ k1s0-error/
│  │  │  │  └─ k1s0-endpoint/
│  │  │  ├─ services/                 # frameworkが提供する共通マイクロサービス（構成は固定）
│  │  │  │  ├─ auth-service/
│  │  │  │  ├─ config-service/
│  │  │  │  └─ endpoint-service/
│  │  │  └─ README.md
│  │  └─ go/
│  │     └─ (同上)
│  ├─ frontend/
│  │  ├─ react/
│  │  │  ├─ README.md
│  │  │  ├─ package.json               # フロント共通のビルド/検証（必要に応じて）
│  │  │  ├─ pnpm-workspace.yaml        # React 共通パッケージを monorepo 管理
│  │  │  ├─ tsconfig.base.json
│  │  │  ├─ eslint.config.mjs
│  │  │  ├─ vitest.workspace.ts        # 任意（共通パッケージのテスト統合）
│  │  │  └─ packages/
│  │  │     ├─ k1s0-config/            # YAML設定の読み込み/型付け/バリデーション
│  │  │     │  ├─ src/
│  │  │     │  │  ├─ loaders/           # 取得方法（HTTP/埋め込み等）を統一
│  │  │     │  │  ├─ schema/            # zod 等で型/バリデーション
│  │  │     │  │  └─ index.ts
│  │  │     │  └─ package.json
│  │  │     ├─ k1s0-http/              # APIクライアント（fetch/axios抽象）+ retry/timeout 既定
│  │  │     │  ├─ src/
│  │  │     │  │  ├─ http/              # client/transport
│  │  │     │  │  ├─ retry/
│  │  │     │  │  ├─ timeout/
│  │  │     │  │  └─ index.ts
│  │  │     │  └─ package.json
│  │  │     ├─ k1s0-auth-client/       # 認証（token管理/refresh/権限制御の薄いSDK）
│  │  │     │  ├─ src/
│  │  │     │  │  ├─ client/            # login/logout/refresh
│  │  │     │  │  ├─ token/             # storage/expiry
│  │  │     │  │  ├─ permission/        # permission 判定の共通
│  │  │     │  │  └─ index.ts
│  │  │     │  └─ package.json
│  │  │     ├─ k1s0-observability/     # OTel（web）/ログ/trace_id 相関の共通
│  │  │     │  ├─ src/
│  │  │     │  │  ├─ logging/
│  │  │     │  │  ├─ tracing/
│  │  │     │  │  └─ index.ts
│  │  │     │  └─ package.json
│  │  │     ├─ k1s0-ui/                # 共通UI（Design System）
│  │  │     │  ├─ src/
│  │  │     │  │  ├─ components/
│  │  │     │  │  ├─ primitives/
│  │  │     │  │  ├─ icons/
│  │  │     │  │  ├─ theme/
│  │  │     │  │  └─ index.ts
│  │  │     │  ├─ styles/
│  │  │     │  └─ package.json
│  │  │  │  ├─ k1s0-shell/             # AppShell（Header/Footer/Menu）とレイアウトの共通
│  │  │  │  │  ├─ src/
│  │  │  │  │  │  ├─ layout/
│  │  │  │  │  │  ├─ header/
│  │  │  │  │  │  ├─ footer/
│  │  │  │  │  │  ├─ menu/
│  │  │  │  │  │  └─ index.ts
│  │  │  │  │  └─ package.json
│  │  │  │  ├─ k1s0-navigation/        # 設定駆動ナビゲーション（routes/menu/flows）
│  │  │  │  │  ├─ src/
│  │  │  │  │  │  ├─ schema/            # zod 等でスキーマ/バリデーション
│  │  │  │  │  │  ├─ router/            # React Router 連携
│  │  │  │  │  │  ├─ menu/              # メニュー生成
│  │  │  │  │  │  ├─ flows/             # 遷移制御（許可遷移/条件）
│  │  │  │  │  │  └─ index.ts
│  │  │  │  │  └─ package.json
│  │  │     ├─ k1s0-state/             # 状態管理の薄いラッパ（採用方式を固定する場合）
│  │  │     │  ├─ src/
│  │  │     │  │  ├─ store/
│  │  │     │  │  ├─ query/
│  │  │     │  │  └─ index.ts
│  │  │     │  └─ package.json
│  │  │     ├─ eslint-config-k1s0/      # ルール固定（テンプレ/CLIが参照）
│  │  │     │  └─ package.json
│  │  │     └─ tsconfig-k1s0/           # TS設定固定（テンプレ/CLIが参照）
│  │  │        └─ package.json
│  │  └─ flutter/
│  │     ├─ README.md
│  │     ├─ melos.yaml                 # Dart/Flutter の monorepo 管理（推奨）
│  │     ├─ analysis_options.yaml      # lint ルール固定
│  │     ├─ pubspec.yaml               # workspace 管理用（必要に応じて）
│  │     └─ packages/
│  │        ├─ k1s0_config/            # YAML設定読み込み/型/バリデーション
│  │        │  ├─ lib/
│  │        │  │  └─ src/
│  │        │  │     ├─ loaders/        # 取得方法（asset/HTTP等）を統一
│  │        │  │     ├─ schema/
│  │        │  │     └─ k1s0_config.dart
│  │        │  └─ pubspec.yaml
│  │        ├─ k1s0_http/              # APIクライアント（dio/http抽象）+ retry/timeout 既定
│  │        │  ├─ lib/
│  │        │  │  └─ src/
│  │        │  │     ├─ http/
│  │        │  │     ├─ retry/
│  │        │  │     ├─ timeout/
│  │        │  │     └─ k1s0_http.dart
│  │        │  └─ pubspec.yaml
│  │        ├─ k1s0_auth/              # 認証（token管理/refresh/権限制御）
│  │        │  ├─ lib/
│  │        │  │  └─ src/
│  │        │  │     ├─ client/
│  │        │  │     ├─ token/
│  │        │  │     ├─ permission/
│  │        │  │     └─ k1s0_auth.dart
│  │        │  └─ pubspec.yaml
│  │        ├─ k1s0_observability/     # OTel/ログ/trace_id 相関の共通
│  │        │  ├─ lib/
│  │        │  │  └─ src/
│  │        │  │     ├─ logging/
│  │        │  │     ├─ tracing/
│  │        │  │     └─ k1s0_observability.dart
│  │        │  └─ pubspec.yaml
│  │        ├─ k1s0_ui/                # 共通UI（Design System）
│  │        │  ├─ lib/
│  │        │  │  └─ src/
│  │        │  │     ├─ widgets/
│  │        │  │     ├─ primitives/
│  │        │  │     ├─ icons/
│  │        │  │     ├─ theme/
│  │        │  │     └─ k1s0_ui.dart
│  │        │  └─ pubspec.yaml
│  │        └─ k1s0_state/             # 状態管理の薄いラッパ（採用方式を固定する場合）
│  │           ├─ lib/
│  │           │  └─ src/
│  │           │     ├─ store/
│  │           │     ├─ effects/
│  │           │     └─ k1s0_state.dart
│  │           └─ pubspec.yaml
│  └─ database/
│     ├─ schema/                      # 共通スキーマ方針/DDL束ねなど（任意）
│     └─ table/                       # framework共通テーブル定義
│        ├─ fw_m_setting.sql
│        ├─ fw_m_user.sql
│        ├─ fw_m_role.sql
│        ├─ fw_m_permission.sql
│        ├─ fw_m_user_role.sql
│        ├─ fw_m_role_permission.sql
│        └─ fw_m_endpoint.sql
│
├─ feature/                            # 個別機能チームの開発領域（機能単位=マイクロサービス）
│  ├─ backend/
│  │  ├─ rust/
│  │  │  └─ {feature_name}/
│  │  │     ├─ Cargo.toml
│  │  │     ├─ README.md
│  │  │     ├─ config/
│  │  │     │  ├─ default.yaml
│  │  │     │  ├─ dev.yaml
│  │  │     │  ├─ stg.yaml
│  │  │     │  └─ prod.yaml
│  │  │     ├─ src/
│  │  │     │  ├─ application/
│  │  │     │  ├─ domain/
│  │  │     │  ├─ infrastructure/
│  │  │     │  └─ presentation/
│  │  │     ├─ migrations/             # feature固有DB（必要な場合）
│  │  │     ├─ openapi/                # OpenAPI/GraphQLスキーマ等
│  │  │     ├─ tests/                  # 結合テスト/契約テスト
│  │  │     └─ deploy/                 # K8s（base + overlays/{env}）
│  │  └─ go/
│  │     └─ {feature_name}/
│  │        ├─ go.mod
│  │        ├─ README.md
│  │        ├─ config/
│  │        │  ├─ default.yaml
│  │        │  ├─ dev.yaml
│  │        │  ├─ stg.yaml
│  │        │  └─ prod.yaml
│  │        ├─ src/
│  │        │  ├─ application/
│  │        │  ├─ domain/
│  │        │  ├─ infrastructure/
│  │        │  └─ presentation/
│  │        ├─ migrations/             # feature固有DB（必要な場合）
│  │        ├─ openapi/                # OpenAPI/GraphQLスキーマ等
│  │        ├─ tests/                  # 結合テスト/契約テスト
│  │        └─ deploy/                 # K8s（base + overlays/{env}）
│  ├─ frontend/
│  │  ├─ react/
│  │  │  └─ {feature_name}/
│  │  │     ├─ package.json
│  │  │     ├─ README.md
│  │  │     ├─ config/
│  │  │     │  ├─ default.yaml
│  │  │     │  ├─ dev.yaml
│  │  │     │  ├─ stg.yaml
│  │  │     │  └─ prod.yaml
│  │  │     ├─ src/
│  │  │     │  ├─ presentation/
│  │  │     │  ├─ application/
│  │  │     │  ├─ domain/
│  │  │     │  ├─ infrastructure/
│  │  │     │  └─ main.tsx
│  │  │     └─ deploy/                 # K8s（base + overlays/{env}）
│  │  └─ flutter/
│  │     └─ {feature_name}/
│  │        ├─ pubspec.yaml
│  │        ├─ README.md
│  │        ├─ config/
│  │        │  ├─ default.yaml
│  │        │  ├─ dev.yaml
│  │        │  ├─ stg.yaml
│  │        │  └─ prod.yaml
│  │        ├─ lib/
│  │        │  ├─ main.dart
│  │        │  └─ src/
│  │        │     ├─ presentation/
│  │        │     ├─ application/
│  │        │     ├─ domain/
│  │        │     └─ infrastructure/
│  │        └─ deploy/                 # K8s（base + overlays/{env}）
│  └─ database/
│     ├─ schema/                       # feature固有スキーマ方針/集約（任意）
│     └─ table/                        # feature固有テーブル定義（必要な場合）
│
└─ bff/                                # （任意）フロント向け集約API層
   └─ {bff_name}/

(補足: サービス内の最小ファイル例は `CLI/templates/backend-rust/feature/` を基準とする（※ 現状リポジトリには未作成）)
```


