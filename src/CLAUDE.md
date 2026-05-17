# src/ コーディングポリシー

## 一文方針

src/ 配下の全コードは「文章で守る」ではなく「物理機構（compile / lint / CI / runtime / E 層）で必ず止まる」設計で書く。技術的判断で悩んだ場合は至高を目指す判断を優先すること。

## 責務分担

| ドキュメント | 責務 |
|---|---|
| `/CLAUDE.md`（root）| project policy / 11 Phase / dual sign-off 制度 |
| `/ARCHITECTURE.md` | src/ 物理規約（allowlist / 依存方向 / 命名規則）|
| 本ファイル (`src/CLAUDE.md`) | **src/ 内コーディング禁止事項**（全軸共通）|
| `src/<axis>/CLAUDE.md` | 軸固有コーディング制約 |
| `docs/03_概要設計/<軸>設計方針/` | 言語選定・モジュール構成・設計パターン |
| `docs/04_詳細設計/02_強制機構/<軸>強制機構.md` | CI fail 条件の詳細仕様 |

## 軸構造と依存方向

src/ 直下は lint が物理 enforce する allowlist（`tools/docs_lint/run_lint.py` `check_repository_layout()`）:
- 10 実装軸: `tier1 tier2 tier3 infra data security ops client test formal`
- 2 補助 namespace: `_meta _crosscutting`
- 許可ファイル: `README.md` / `CLAUDE.md` のみ

**依存方向の制約（下位から上位への直接 import 禁止）:**
- tier3 → tier1 / OSS 直接 import 禁止（tier2 SDK 経由必須）
- tier2 → tier1 OSS crate の直接 public API 露出禁止（facade 経由必須）
- _crosscutting → 各軸実装への直接依存禁止（shared interface のみ）

## 言語スタック早見表

| 軸 | 言語 |
|---|---|
| tier1 Server | Rust stable |
| tier1 Operator | Go 1.22+ |
| tier1/2/3 Library | Rust / C# (.NET 8+) / Go / TypeScript（4 言語等価強度）|
| tier3 SPA | TypeScript + React |
| tier3 Desktop | Rust + Tauri / C# WPF |
| tier3 Legacy | .NET Framework 4.6.2+ |
| formal | TLA+ / Lean 4 / Scala(Stainless) / Dafny / Rust(Kani) / C(CBMC) |
| tooling | Python 3.12（`tools/` 配下）|

各軸の詳細は `docs/03_概要設計/<軸>設計方針/README.md` を単一の真とし、本ファイルでは再記述しない。

## 全軸共通コーディング禁止事項

### build artifact の手書き禁止

`src/<axis>/lock/*.lock.yaml` は全て `tools/lock_yaml_generator/` が生成する build artifact。手書き編集は CI fail。新規 lock.yaml が必要な場合は生成器に追加してから生成する。

### 設計文書を src/ に置くことの禁止

設計文書・仕様書・議論は全て `docs/` に書く。src/ 内では `README.md`（docs へのリンク 1 行）と `CLAUDE.md`（コーディングポリシー）のみ許可。

### LLM 単独 sign-off 禁止

LLM が生成したコード・proof・テストは human author の dual sign-off（human cosign × 1 + AI static analysis evidence × 1）なしに merge 禁止。詳細は `/CLAUDE.md` の dual sign-off 規約を参照。

### wall-clock TTL 禁止

`SystemTime::now()` / `DateTime.UtcNow` 等の wall clock を TTL・deadline 計算に使うことを禁止。全て HLC（Hybrid Logical Clock）を使う。`src/client/hlc_lib/` の言語別 wrapper を参照。

### dimension override 禁止

- src/ 直下に許可外 dir/file を追加しない（lint fail）
- 軸内で新しいサブシステムを増やす場合は既存の class bundle 内に収める
- 「新しいことをやりたい」ために新軸を src/ 直下に追加することは dimension override

### dead spec を残さない

参照されないモジュール・型・annotation は CI fail になる前に削除する。「使われていないが念のため残す」は採らない。

### production / development 区別禁止

セキュリティ・RLS・audit を `#[cfg(debug_assertions)]` / `#[cfg(test)]` で弱体化するコードは禁止。production / development で振る舞いが変わるコードは全て禁止。

### Phase gate の遵守

各軸の実装 Phase 前の先行実装禁止:

| 軸 | 実装 Phase |
|---|---|
| `tools/` tooling | P0 |
| `_meta` | P1 |
| `test` scaffold | P4 |
| `formal` 先行 | P5（2 cell）→ P11（残 93 cell）|
| `infra` | P8 |
| `data` | P9 |
| `tier1 → tier2 → tier3 → security → ops → client` | P10（sequential）|

### 各行コメント記載義務

src/ 配下の全コードは、各行の上に当該行の意図を説明する日本語コメントを記載すること。コメント欠落は CI fail（lint 強制）。

## 関連参照

- `/CLAUDE.md` — project policy / Phase 表 / dual sign-off 規約
- `/ARCHITECTURE.md` — src/ 物理規約の SoT
- `src/<axis>/CLAUDE.md` — 軸固有コーディング制約
- `docs/03_概要設計/` — 各軸設計方針
- `docs/04_詳細設計/02_強制機構/` — CI fail 条件の詳細仕様
- `docs/04_詳細設計/05_lock_yaml体系/README.md` — lock.yaml 軸別カタログ
