# ARCHITECTURE

## 一文方針

root から見て「どこに何があり、何を含まないか」は本書が単一の真とする。`docs/` 内部の番号体系・命名規約・frontmatter 規約は [`docs/00_format/conventions/numbering.md`](docs/00_format/conventions/numbering.md) に委譲し、本書では再記述しない。

---

## 至高路線における ARCHITECTURE.md の立ち位置

本書は root 直下の物理層規約を保持する。プロダクト価値の訴求は [`README.md`](README.md)、AI agent 向け policy は [`CLAUDE.md`](CLAUDE.md) が担う。3 ファイルの責務は disjoint であり、内容の重複は dimension override と同型のためそれぞれの専任 file を単一の真として扱う。

---

## root 4 本柱の役割分担

| ファイル | 主読者 | 責務 | 禁止事項 |
|---|---|---|---|
| `/CLAUDE.md` | AI agent | policy / 至高路線 / 変更履歴明文化禁止 | 構造記述 |
| `/README.md` | OSS 訪問者 / エンジニア | プロダクト価値 / 19 軸 matrix / 6 層 defense-in-depth / 5⁴ threat catalog | 物理層規約 |
| `/LICENSE` | 法務 | Apache 2.0 全文 | 説明文 |
| `/ARCHITECTURE.md` | 実装者 / contributor | top-level directory 戦略 / `src/` 19 軸射影 / defense-in-depth mapping | `docs/` 内部規約の再記述 |

OSS 受領者は `README.md` で価値を把握 → 本書で物理層を把握 → `docs/00_format/README.md` で規約を把握、の単方向で onboard できる。逆経路（本書から価値訴求、README.md から物理層規約）は dimension override に相当するため禁止する。

---

## top-level directory 戦略

各 directory の責務は allowlist で一意に決まる。例外配置・個別 override 経路を持たない。

| directory | 含む | 含まない | 規約参照 |
|---|---|---|---|
| `/.claude/` | Claude Code 専用 commands / skills / settings | プロダクト logic / 設計文書 | ローカル設定 |
| `/.claudeignore` | Claude 文脈除外パターン（`*.svg` `*.drawio` `docs/90_knowledge/`） | 一般 lint 除外（→ `.gitignore`） | 本書 |
| `/.github/workflows/` | CI workflow YAML のみ | スクリプト実体（→ `tools/`） | GitHub Actions 公式 schema |
| `/.gitignore` | git 除外パターン | Claude 除外（→ `.claudeignore`） | 空維持可 |
| `/CLAUDE.md` | AI agent policy | 構造記述 | 上記 4 本柱表 |
| `/LICENSE` | Apache 2.0 全文 | 説明文 | OSI Apache 2.0 |
| `/README.md` | プロダクト価値訴求 / 19 軸 matrix | 物理層規約 | 上記 4 本柱表 |
| `/ARCHITECTURE.md` | 本書 | `docs/` 内部規約 / 価値訴求 | 上記 4 本柱表 |
| `/docs/` | 全 markdown 設計成果物 + 規約層（`00_format`）+ 軸内 `img/` | コード / `tools/` スクリプト | [`docs/00_format/README.md`](docs/00_format/README.md) |
| `/img/` | repo メタアセット（README banner / 全体構成図）の `.svg` / `.drawio` pair | 軸別文書アセット（→ `docs/<軸>/img/`） | 本書「img/ 二重構造」章 |
| `/src/` | 実装コード（19 軸射影） | 設計文書（→ `docs/`） | 本書「src/ 19 軸射影」章 |
| `/tools/` | CI / build / lint スクリプト実体 | プロダクト logic / 設計文書 | 各 tool 配下 `README.md` |

**root 直下許可ファイル**: `CLAUDE.md` / `README.md` / `LICENSE` / `ARCHITECTURE.md` / `.claudeignore` / `.gitignore` / `Makefile` / `pyproject.toml` / `requirements.txt` / `uv.lock`。その他 `.md` / `.yaml` の root 散乱は `tools/docs_lint/` による B 層 lint で fail する。`Makefile` は build 基盤 entry point として root 直下を必須とする（convention）。`pyproject.toml` / `uv.lock` は Python tooling deps 管理用。

---

## top-level directory × defense-in-depth mapping

A=compile / B=lint / C=integration test / D=runtime / E=物理 / F=数学的（formal proof）

| directory | A | B | C | D | E | F |
|---|:-:|:-:|:-:|:-:|:-:|:-:|
| `/.github/workflows/` |   | ● | ● |   |   |   |
| `/tools/docs_lint/` |   | ● |   |   |   |   |
| `/tools/lock_yaml_generator/` | ● | ● |   |   |   |   |
| `/docs/00_format/linters/` |   | ● |   |   |   |   |
| `/src/tier1/` `/src/tier2/` `/src/tier3/` `/src/infra/` `/src/data/` | ● | ● | ● | ● |   |   |
| `/src/security/` |   | ● | ● | ● | ● |   |
| `/src/ops/` |   |   | ● | ● | ● |   |
| `/src/test/` |   |   | ● |   |   |   |
| `/src/formal/` |   |   |   |   |   | ● |
| `/src/_crosscutting/` | ● | ● | ● | ● | ● |   |
| `/src/_meta/` | ● | ● |   |   |   |   |

任意の単層が破れても他層が必ず止める（6 層 defense-in-depth の `src/` 射影）。

---

## src/ 19 軸射影

### 19 軸 換算定義

「19 軸」は `_meta/axis_registry.lock.yaml` が管理する **登録軸総数**（cap v1 = 20、現 19、残 1）であり、`src/` 物理 dir 数とは独立して定義される。

| 量 | 定義 | 値 |
|---|---|---|
| 登録軸総数 | `axis_registry.lock.yaml` の登録件数 | **19**（cap 20、残 1） |
| `src/` 実装軸数 | `src/` 直下 allowlist の軸 dir 数（`_meta` `_crosscutting` 除く） | **10**（tier1–formal） |
| cross-cutting 適合仕様数 | `src/_crosscutting/` 内 `NN_<slug>/` 数 | **13** |

`_meta/` と `_crosscutting/` は補助 namespace であり登録軸数にカウントしない。20 適合仕様（`docs/04_詳細設計/01_適合仕様/`）と 13 cross-cutting（`src/_crosscutting/`）は independent sets であり合算しない。登録軸 19 の内訳（primary 10 + sub-registered 9）は [`docs/00_format/conventions/numbering.md`](docs/00_format/conventions/numbering.md) を単一の真とする。

---

### 射影原則

19 軸を `src/` 直下に **1:1 unnumbered mapping** で物理化する。言語別・bounded context 別射影は dimension override 経路を作るため採用しない。axis 数字番号（`07_`, `08_`, ...）は言語識別子として合法でないため、axis 名 unnumbered を採用する。`_` prefix は「軸ではない補助 namespace」を物理表示し、axis cap 20 と衝突しない。

各 `src/<軸>/` ディレクトリは `docs/03_概要設計/<軸>設計方針/` と axis 名で 1:1 対応する。言語選定・モジュール構成は各軸の `docs/03_概要設計/<軸>設計方針/README.md` に委譲し、本書では言及しない。

### src/ 直下許可構造

```
/src/
├── README.md
├── tier1/          # SDK / Bidi / 認証 / 鍵管理 — シニア層
├── tier2/          # テナント分離 / atomic 三表書込 — 中堅層
├── tier3/          # クライアント状態 — ジュニア層
├── infra/          # クラスタ位相 / 時刻整合
├── data/           # データ保全 / WAL / migration
├── security/       # 脅威モデル / build_provenance
├── ops/            # 運用ループ / ops-edge
├── client/         # 9 言語 SDK lockstep
├── test/           # 18 軸 × 5 verification_class = 90 cell
├── formal/         # TLA+ / Stainless / Dafny / Lean 4 / Kani / CBMC 成果物
├── _meta/          # axis_registry.lock.yaml 生成体系（meta-axis 実装）
└── _crosscutting/  # 13 cross-cutting 適合仕様の実装（01_http2_enforcement/ ... 13_dotnet8_connect_inhouse/）
```

`src/` 直下に上記以外のサブディレクトリを配置することは dimension override に相当するため B 層 lint で fail する。

### src/ ↔ docs/03_概要設計/ 名前対応

| `src/` | `docs/03_概要設計/` |
|---|---|
| `tier1/` | `02_tier1設計方針/` |
| `tier2/` | `03_tier2設計方針/` |
| `tier3/` | `04_tier3設計方針/` |
| `infra/` | `05_infra設計方針/` |
| `data/` | `06_data設計方針/` |
| `security/` | `07_security設計方針/` |
| `ops/` | `08_ops設計方針/` |
| `client/` | `09_client設計方針/` |
| `test/` | `10_test設計方針/` |
| `formal/` | `11_formal設計方針/` |
| `_crosscutting/` | `12_クロスカッティング設計/` |
| `_meta/` | [`docs/00_format/conventions/numbering.md`](docs/00_format/conventions/numbering.md)（axis registry 仕様）|

---

## img/ 二重構造の境界規律

| 場所 | 格納対象 | 参照元 | 禁止 |
|---|---|---|---|
| `/img/` | repo メタアセット（README banner / アーキ全体構成図）のみ | `README.md` / 本書 | 軸別文書アセット |
| `/docs/<軸>/img/` | 当該軸文書から参照される drawio/svg pair | 当該軸文書のみ | 軸をまたいだ参照 |

共通ルール: `.drawio` と `.svg` は必ず pair 化。`.claudeignore` の `*.svg` `*.drawio` 除外設定と整合する。

---

## 物理 enforcement

> **pre-P0 現状**: 本節に記載の `tools/docs_lint/run_lint.py`（P2 deliverable）および `tools/lock_yaml_generator/generate_release_gate.py`（P0 deliverable）は、pre-P0 時点で実体ゼロ（未物理化）。

本書の内容は文章宣言に留まらず、以下の機構で **B 層 lint** として物理 enforce する。

### `tools/docs_lint/run_lint.py` — root layout check

既存の frontmatter / forbidden-expression / xref check と同一 chain に挿入する `check_repository_layout()` が以下を検証する:

1. root 直下許可ファイル allowlist 違反（想定外 `.md` / `.yaml` の散乱）→ fail
2. root top-level directory allowlist 違反（増減）→ fail
3. `src/` 直下ファイルが `README.md` / `CLAUDE.md` 以外 → fail、サブディレクトリが 10 軸 + `_meta` + `_crosscutting` 以外 → fail
4. `src/_crosscutting/` 配下が `NN_<slug>/` 形式（`NN` = 01〜13、`<slug>` = `[a-z][a-z0-9_-]+`）以外 → fail
5. `/img/` 直下のファイル拡張子が `.svg` / `.drawio` 以外 → fail

### `tools/lock_yaml_generator/generate_release_gate.py` — meta cell 追加

`meta.repository_layout_integrity` cell を AND-gate に組み込む。既存 `meta.docs_lint_green` / `meta.axis_registry_complete` と同型の meta cell として扱い、`source_lock_artifact: repository_layout.lock.yaml` を参照する。

---

## dimension override 禁止との同型確認

| 禁止事項 | 本書での対応 |
|---|---|
| 軸の class bundle が他 dimension を一意に導出（override 経路なし） | 各 directory の責務は allowlist で一意。例外配置経路なし |
| dead spec を CI で殺す | root layout check が B 層 lint で物理拒否 |
| build artifact は手書き禁止 | `release_gate.lock.yaml` は `generate_release_gate.py` の生成物 |
| 参照されない要素は merge fail | `docs_lint` の xref check が resolve 失敗で fail |
| 段階的 release 禁止 | `FORBIDDEN_EXPRESSIONS` で "Phase N で追記" 等の日付・段階記述を lint で物理拒否 |

---

## 関連参照

- [`docs/00_format/README.md`](docs/00_format/README.md) — 規約層 index（番号体系・crosslink 規約・frontmatter schema・linters を含む）
- [`docs/03_概要設計/README.md`](docs/03_概要設計/README.md) — 全軸設計方針 index（軸別 README へのリンクを含む）
- [`tools/docs_lint/run_lint.py`](tools/docs_lint/run_lint.py) — B 層 lint（root layout check を含む）
- [`tools/lock_yaml_generator/generate_release_gate.py`](tools/lock_yaml_generator/generate_release_gate.py) — release_gate.lock.yaml 生成
