# tier3 コーディングポリシー

全軸共通ルールは `src/CLAUDE.md` を参照。本ファイルは tier3 固有の制約のみ記述する。

## 言語・workspace

- **SPA**: TypeScript + React
- **Desktop**: C# WPF（.NET 8+）/ Rust + Tauri
- **Legacy**: .NET Framework 4.6.2+（WinForms / WPF）
- **pnpm workspace root**: `src/tier3/typescript/pnpm-workspace.yaml`
- **lock.yaml 配置先**: `src/tier3/lock/`（手書き禁止）
- **CRDT**: `src/tier3/typescript/packages/crdt/`（tier3 内部に閉じ込める。cross-cutting spec 不要）

設計パターン・モジュール構成の詳細は `docs/03_概要設計/04_tier3設計方針/README.md` を単一の真とする。

## コーディング制約

### 依存方向

- tier1 / OSS 直接 import 禁止 → tier2 SDK 経由必須（`eslint-plugin-import` / `depguard` / `PublicApiAnalyzers`）
- 業務管理 API（`/admin/v1/...` 等）を tier3 業務 UI から直接 import 禁止

### データ保護

- PII を `localStorage` / `IndexedDB` / `cookie` に平文保管禁止（暗号化 IndexedDB の Outbox のみ例外）
- 認可結果（`AuthorizationDenied`）の client-side cache 禁止
- 公開 type / form schema / URL routing / クエリパラメータに `tenant_id` フィールド禁止

### proto・型定義

- `.proto` ファイルを tier3 リポジトリ内に置くことの禁止
- 業務 entity / event の独自再宣言型禁止（tier2 generated stub のみ使用）

### セキュリティ

- CSP: `unsafe-inline` / `unsafe-eval` 禁止
- 第三者 CDN リソースに SRI（Subresource Integrity）必須
- contract test skip フラグ / assertion 弱体化 PR は CI fail

### アクセシビリティ・品質

- WCAG 2.1 AA 必須（`axe-core` CI で merge 阻止）
- Web Vitals: Lighthouse CI で merge 阻止
- i18n: 未定義 / 未使用 translation key → CI fail

### Tauri

- `src/tier3/companion/src-tauri/Cargo.toml` は Tauri framework が要求する frontend glue crate
- tier1 Cargo workspace member にはならない（standalone crate として扱う）
- sidecar exe の実装は `src/_crosscutting/07_tauri_companion_sidecar/sidecar/` が primary

## 関連参照

- `docs/03_概要設計/04_tier3設計方針/README.md` — 言語・設計パターン
- `docs/04_詳細設計/02_強制機構/03_tier3強制機構.md` — CI fail 条件の詳細
- `docs/04_詳細設計/01_適合仕様/11_クライアント状態適合仕様.md` — 4 layer state
