# src/platform — 横断ツール（Scaffold CLI / Analyzer / Backstage Plugins）

tier1〜tier3 のいずれにも所属しない、開発者体験（DX）を支える横断ツール群。
詳細設計は [`docs/05_実装/00_ディレクトリ設計/10_ルートレイアウト/02_src配下の層別分割.md`](../../docs/05_実装/00_ディレクトリ設計/10_ルートレイアウト/02_src配下の層別分割.md)。

## 配置

```text
platform/
├── scaffold/                                       # k1s0-scaffold CLI（Rust 実装、IMP-CODEGEN-SCF-030）
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs                                 # CLI（list / new / new --input json --dry-run）
│       ├── lib.rs                                  # engine 公開 API
│       ├── template.rs                             # Backstage Software Template v1beta3 パース
│       ├── engine.rs                               # Handlebars + walkdir 展開
│       └── error.rs
├── analyzer/                                       # .NET Roslyn Analyzer（IMP-DIR-ROOT-002）
│   ├── K1s0.DependencyDirection.Analyzer.sln
│   └── src/
│       ├── K1s0.DependencyDirection.Analyzer/      # 本体（DiagnosticAnalyzer）
│       ├── K1s0.DependencyDirection.Analyzer.Package/  # NuGet packaging
│       └── tests/                                  # xUnit + Microsoft.CodeAnalysis.Testing
└── backstage-plugins/                              # Backstage 開発者ポータル plugin（ADR-BS-001）
    ├── k1s0-catalog/                               # k1s0.io/* annotation 専用 catalog plugin
    └── k1s0-scaffolder/                            # k1s0-scaffold の Backstage Custom Action
```

## scaffold（k1s0-scaffold CLI）

```sh
# テンプレ一覧
k1s0-scaffold list

# 対話起動
k1s0-scaffold new tier2-go-service

# CI / Backstage 経由
k1s0-scaffold new --input '{"name":"my-svc","namespace":"k1s0"}' --dry-run
```

CLI と Backstage UI 経路（custom action `k1s0:scaffold-engine`）が**同一 engine** を呼び、生成バイトの一致を golden test で保証する（`tests/golden/scaffold-outputs/`）。

## analyzer（依存方向 Roslyn Analyzer）

`tier3 → tier2 → sdk → tier1` 一方向ルールを `using` 文レベルで強制する。
診断 ID 4 件（すべて Severity=Error）:

| Diagnostic ID | 違反 |
|---|---|
| `K1S0DEPDIR0001` | SDK → Tier2 |
| `K1S0DEPDIR0002` | SDK → Tier3 |
| `K1S0DEPDIR0003` | Tier2 → Tier3 |
| `K1S0DEPDIR0004` | Tier1 → 上位層（SDK 含む） |

`<ProjectReference>` レベルの違反は別途 NetArchTest（各 .sln 配下 tests/ で xUnit 化）が捕捉。

## backstage-plugins

`@k1s0/backstage-plugin-catalog` と `@k1s0/backstage-plugin-scaffolder` の 2 plugin。
採用組織の Backstage バージョンへの強い依存を避けるため、OSS 側は skeleton と annotation 定数のみを提供し、
採用組織が `@backstage/core-plugin-api` 等を import して `createPlugin` / `createTemplateAction` で接続する想定。

## 関連設計

- [ADR-DEV-001](../../docs/02_構想設計/adr/ADR-DEV-001-paved-road.md) — Paved Road
- [ADR-BS-001](../../docs/02_構想設計/adr/) — Backstage 採用
- [IMP-CODEGEN-SCF-030](../../docs/05_実装/20_コード生成設計/) — Scaffold CLI
- [IMP-DIR-ROOT-002](../../docs/05_実装/00_ディレクトリ設計/10_ルートレイアウト/05_依存方向ルール.md) — 依存方向強制
