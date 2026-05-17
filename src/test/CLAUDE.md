# test コーディングポリシー

全軸共通ルールは `src/CLAUDE.md` を参照。本ファイルは test 固有の制約のみ記述する。

## 配置・構成

- **coverage_matrix**: `src/test/coverage_matrix/`（18 軸 × 5 verification_class = 90 cell）
- **playwright**: `src/test/playwright/`（製造業 9 stress test が ship blocker）
- **lock.yaml 配置先**: `src/test/lock/`（手書き禁止）

設計パターン・モジュール構成の詳細は `docs/04_詳細設計/01_適合仕様/19_検証規律適合仕様.md` を単一の真とする。

## コーディング制約

### cluster 上の操作制限

- **production cluster での chaos / load / scenario test 禁止**（shadow cluster か Testcontainers のみ）
- `src/test/litmus/` の chaos scenario に `shadow_cluster` 注釈が必須

### flaky test 管理

- flaky quarantine: `quarantine_due_at` は 14 日 cap（超過は CI fail）
- flaky test を修正せずに quarantine 期間を延長することを禁止

### LLM 生成テスト

- LLM が生成したテストコードには `ai_generator_id` / `model_version` / `prompt_hash` の注釈必須
- 注釈なしの LLM 生成テストは CI fail

### random seed・再現性

- test runner でのローカル random seed 使用禁止（seed は固定または `regression_corpus.lock.yaml` から取得）

### skip 注釈

- skip 注釈には justification + 新 verification_class 切替の記述が必須
- 「実装待ち」理由での skip は禁止（未実装なら test を書かない）

### defect 修正

- regression_corpus への bind なし defect 修正 PR → CI fail
- 修正した defect は対応する property test seed を `regression_corpus` に追加する

### performance

- performance baseline 引き下げ PR は dual review 必須
- mutation_score の quarter monotonic 違反 → CI fail

### 製造業 9 stress test

- `src/test/playwright/manufacturing_stress/` の 9 spec（01_〜09_）は全て pass が ship blocker
- `src/tier2/pack/manufacturing/stress_test/` の domain fixture を import して実行する

## 関連参照

- `docs/04_詳細設計/01_適合仕様/19_検証規律適合仕様.md` — 検証規律
- `docs/04_詳細設計/02_強制機構/09_test強制機構.md` — CI fail 条件の詳細
