---
id: format.meta.frontmatter_convention
axis: meta
phase: format
kind: convention
status: draft
depends_on: []
covered_by:
  defense_in_depth_layers: []
  proof_classes: []
---

# frontmatter 規約

## 一文方針
- 全 `.md` ドキュメントは YAML frontmatter を必須とし、`id` / `axis` / `phase` / `kind` / `status` / `depends_on` / `covered_by` を required field として持つ。`changelog` / `last_updated` / `last_modified` / `author` / `reviewers` は forbidden field として CI で物理拒否する。

## 至高路線における原則
- 構造的整合性は文章ではなく schema で物理 enforce する（CLAUDE.md「文章運用に頼らず物理 enforce」）。
- frontmatter は build artifact 的に扱う：`id` は配置パスから一意に導出可能であり、不一致は CI fail。手書きで矛盾を作る経路を持たない。
- 変更履歴の明文化を禁止する（リポジトリ root `CLAUDE.md`）。`changelog` / `last_updated` 等の時間軸 field を schema レベルで forbidden とすることで、文書側に履歴を持ち込む経路を物理的に塞ぐ。git 履歴が時間軸の単一の真である。
- frontmatter は読者ではなく機械への宣言である。読者向けの metadata（owner / contact / version 等）はここに置かない。

## required field

### `id`
- 形式: `<phase_short>.<axis>.<slug>`（小文字 ASCII + ドット区切り）
- 導出規則: 配置パスから機械的に導出可能であること
  - 例: `docs/04_詳細設計/01_適合仕様/20_形式検証適合仕様.md` → `id: detail.formal.formal_conformance`
  - 例: `docs/03_概要設計/11_formal設計方針/05_時相安全性方針.md` → `id: arch.formal.temporal_safety_policy`
  - 例: `docs/02_要件定義/04_技術選定/01_OSS採用一覧.md` → `id: req.overview.oss_catalog`
- `phase_short` enum: `plan` / `req` / `arch` / `detail` / `format`（00_format 配下）/ `env`（05_環境構築 配下）
- `axis` enum: `tier1` / `tier2` / `tier3` / `infra` / `data` / `security` / `ops` / `client` / `test` / `formal` / `cross_http2` / `cross_kek` / `cross_schema` / `cross_fsm` / `cross_slo` / `cross_bff` / `cross_pii` / `cross_edge` / `meta` / `overview`
- cross_* spec の id 導出規則: `detail.cross_<sub>.<slug>` 形式（例: `detail.cross_http2.http2_enforcement`）
- 不一致は CI fail。手書きを許容しない。

### `axis`
- enum: `tier1` / `tier2` / `tier3` / `infra` / `data` / `security` / `ops` / `client` / `test` / `formal` / `cross_http2` / `cross_kek` / `cross_schema` / `cross_fsm` / `cross_slo` / `cross_bff` / `cross_pii` / `cross_edge` / `meta` / `overview`
- meta = 軸登録 / 19 軸論など軸自体の管掌、overview = 直下 4 ファイル等の cross-cutting 概論。
- cross_http2 〜 cross_edge は `04_詳細設計/03_クロスカッティング適合仕様/` 配下の 13 spec を 8 cluster に canonical bind する軸。それ以外の spec には原則 cross_* を使用しない。

### `phase`
- enum: `plan` / `requirement` / `architecture` / `detail` / `cross_cutting` / `format` / `env_setup`
- `cross_cutting` は `04_詳細設計/03_クロスカッティング適合仕様/` および `03_概要設計/12_クロスカッティング設計/` 配下のみ。
- `format` は `00_format/` 配下のみ。
- `env_setup` は `05_環境構築/` 配下のみ。id 先頭は `env`。

### `kind`
- enum:
  - `responsibility`（責務）
  - `positioning`（位置づけ）
  - `non_scope`（非提供スコープ）
  - `oss`（採用 OSS）
  - `policy`（〇〇方針）
  - `ops_dx`（運用 UI と開発者体験）
  - `enforcement`（強制機構）
  - `conformance_spec`（適合仕様）
  - `cross_cut_spec`（クロスカッティング適合仕様）
  - `index`（README.md 等のフォルダ index）
  - `plan_doc`（企画フェーズの独自ドキュメント）
  - `requirement`（要件定義の独自ドキュメント）
  - `architecture`（概要設計の独自ドキュメント）
  - `detail`（詳細設計の独自ドキュメント）
  - `template`（00_format/templates 配下）
  - `convention`（00_format/conventions 配下）
  - `style`（00_format/style_guide.md）
  - `linter`（00_format/linters 配下）
  - `glossary`（用語集）

### `status`
- enum: `draft` / `locked` / `archived`
- `locked` の意味: 1.0.0 ship blocker 対象。`TBD` / `未定` / `あとで書く` / `TODO` / `後述` の禁止表現が残存していれば CI fail。
- `draft` は WIP 段階。lint 規約は緩和されるが、release branch には ship できない。
- `archived` の意味: 本流 OSS stack と drift した過去構想の保存。lint の forbidden 表現 / 空セクション check を除外、frontmatter required fields は維持。本文冒頭に「> NOTE: 本ドキュメントは v1 採用 OSS と drift しているため archive 化された。」ヘッダブロック必須。

### `depends_on`
- 同一 repo 内の他 `.md` の `id` の配列。空配列 `[]` 許容。
- dangling 参照（指す先が存在しない）は CI fail。

### `covered_by`
- defense-in-depth 6 層 + 5 proof_class への bind を宣言する map。
- 構造:
  ```yaml
  covered_by:
    defense_in_depth_layers: [A, B, C, D, E, F]   # 該当する層のみ列挙
    proof_classes: [v1_temporal_safety_proof]    # formal 関連時のみ、空配列許容
  ```
- 6 層の対応:
  - A: compile（type check / schema check）
  - B: lint（policy check / Conftest / Semgrep）
  - C: integration test（実行検証）
  - D: runtime（admission webhook / drill / circuit breaker）
  - E: 物理（cosign / Kyverno / Object Lock / SBOM）
  - F: 数学的（proof certificate / TLA+ / Lean 等）
- 5 proof_class の対応（SoT: `docs/00_format/frontmatter_schema.yaml` の `proof_classes` enum 5 値）:
  - `v1_temporal_safety_proof`
  - `v1_temporal_liveness_proof`
  - `v1_refinement_proof`
  - `v1_program_correctness_proof`
  - `v1_runtime_modelcheck_proof`
- **注意**: `v1_property_axiom_proof` は存在しない。これは test 軸 `verification_class` の `v1_property_axiom`（corpus での確率的 property 検証）と混同した誤記であり、formal 軸の proof_class ではない。不変量 / contract / DSL 論理の proof には `v1_program_correctness_proof` を使用する。

## optional field

### `related_axes`
- 当 spec が論理的に依存する axis の配列（軸単位の補助 link）。
- `depends_on` は spec 単位の参照、`related_axes` は軸単位の参照。両者は直交して使用可。
- 主に cross_* spec が、関係する従来 10 軸を宣言するために使用する。
- cross_* 以外の spec でも軸間依存を明示する場合に使用可。
- 例: `related_axes: [tier1, data, security, infra]`

### `lock_artifacts`
- 当ドキュメントが宣言する `*.lock.yaml` の名前配列。
- 例: `[proof_inventory.lock.yaml, proof_status.lock.yaml]`
- `04_詳細設計/05_lock_yaml体系/` の catalog と整合する。

## forbidden field（schema レベルで CI 拒否）

| field | 理由 |
|---|---|
| `changelog` | リポジトリ root `CLAUDE.md`「変更履歴を明文化しない」を物理 enforce |
| `last_updated` | 同上。git 履歴が時間軸の単一の真 |
| `last_modified` | 同上 |
| `author` | reviewer / author は proof artifact 側 (`proof_review.lock.yaml`) が持つ。文書側は持たない |
| `reviewers` | 同上 |
| `version` | semver は build artifact 側で管理。文書 frontmatter に持たない |
| `created_at` | git 履歴が単一の真 |
| `tags` | 検索性は `axis` / `kind` / `phase` の組合せで充分。tag による diffuse な分類は drift 源 |

## frontmatter の最小例

```yaml
---
id: detail.formal.formal_conformance
axis: formal
phase: detail
kind: conformance_spec
status: draft
depends_on:
  - arch.formal.responsibility
  - arch.test.verification_conformance
covered_by:
  defense_in_depth_layers: [A, B, C, D, E, F]
  proof_classes:
    - v1_temporal_safety_proof
    - v1_temporal_liveness_proof
    - v1_refinement_proof
    - v1_program_correctness_proof
    - v1_runtime_modelcheck_proof
lock_artifacts:
  - proof_inventory.lock.yaml
  - proof_status.lock.yaml
  - proof_matrix.lock.yaml
  - counter_example.lock.yaml
  - proof_review.lock.yaml
---
```

## CI 実装の規約宣言（Phase 5 で実装）

- `tools/docs_lint/frontmatter_validator.py`（仮称）が `docs/00_format/frontmatter_schema.yaml` を読み、全 `.md` の frontmatter を検証する。
- 検証項目:
  - required field 完備
  - forbidden field 不在
  - enum 整合（`axis` / `phase` / `kind` / `status` / `defense_in_depth_layers` / `proof_classes`）
  - `id` と配置パスの導出整合
  - `depends_on` の dangling check
  - `lock_artifacts` の catalog 整合（04_詳細設計/05_lock_yaml体系）
- 違反 1 件で CI fail（warning 経路を持たない）。
