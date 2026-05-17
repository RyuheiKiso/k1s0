# formal コーディングポリシー

全軸共通ルールは `src/CLAUDE.md` を参照。本ファイルは formal 固有の制約のみ記述する。

## 配置・構成

- **lock.yaml 配置先**: `src/formal/lock/`（手書き禁止）
- **Kani harness**: `src/formal/kani/harness/`（`#[kani::proof]` を scatter しない）

proof_class 別ツール対応:

| proof_class | ツール | スペック数 |
|---|---|---|
| `v1_temporal_safety_proof` | TLA+ / Apalache | 19 |
| `v1_temporal_liveness_proof` | TLA+ / Apalache | 12 |
| `v1_refinement_proof` | TLA+ + Stainless | 35 pair |
| `v1_program_correctness_proof` | Stainless / Dafny / Lean 4 | 19 |
| `v1_runtime_modelcheck_proof` | Kani / CBMC | 14 |

設計方針の詳細は `docs/04_詳細設計/01_適合仕様/20_形式検証適合仕様.md` を単一の真とする。

## コーディング制約

### sign-off

- **LLM 単独 proof 禁止**（dual sign-off 必須: human × 1 + AI static analysis evidence × 1）
- **PR author self-review 禁止**
- reviewer cooldown 90 日遵守（同じ reviewer が 90 日以内に同一 proof を再 review 禁止）

### reproducibility

- SMT solver random seed 固定必須（`certificate_hash` で再現性を検証）
- `certificate_hash` 不一致 → CI fail（proof が再現しない）
- TLA+ / Lean 4 / Kani / CBMC の version pin 必須（`*_pin.lock.yaml` を参照）

### cap 制約

- `accepted_as_bug` cap ≤ 10（超過は破壊的変更扱い、dual sign-off 必須）
- `assumption` cap ≤ 20（超過禁止）
- `termination_exempt` cap ≤ 10

### proof の追加・変更

- `proof_inventory.lock.yaml` 95 cell から外れる proof の追加禁止（新 cell 追加は dual sign-off）
- Kani / CBMC bound 引き下げ PR は dual review 必須
- LLM が生成した proof には `ai_generator_id` / `model_version` / `prompt_hash` 注釈必須

### harness

- Kani harness は `src/formal/kani/harness/` に集約
- 対象 crate のソースに `#[kani::proof]` を scatter しない（harness crate から target crate を依存で参照する形式）

## 関連参照

- `docs/04_詳細設計/01_適合仕様/20_形式検証適合仕様.md` — 形式検証
- `docs/04_詳細設計/02_強制機構/10_formal強制機構.md` — CI fail 条件の詳細
- `docs/04_詳細設計/05_lock_yaml体系/01_proof_artifact体系.md` — proof artifact lifecycle
- `docs/04_詳細設計/05_lock_yaml体系/05_dual_signoff体系.md` — dual sign-off スキーマ
