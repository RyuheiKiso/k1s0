---
id: arch.formal.proof_review_policy
axis: formal
phase: architecture
kind: policy
status: draft
depends_on:
  - arch.formal.formal_index
  - arch.formal.proof_artifact_policy
covered_by:
  defense_in_depth_layers: [B, D, E]
  proof_classes:
    - v1_temporal_safety_proof
    - v1_temporal_liveness_proof
    - v1_refinement_proof
    - v1_program_correctness_proof
    - v1_runtime_modelcheck_proof
lock_artifacts:
  - proof_review.lock.yaml
---

# formal proof_review 方針

## 一文方針
- 全 proof は 2 名 reviewer による dual sign-off を必須とし、reviewer の cosign signature を `proof_review.lock.yaml` に bind、reviewer のうち 1 名以上は LLM 補助でない人間、cryptographic proof は domain expert（暗号 specialist）の review を必須とする。

## 至高路線における立ち位置
- 単一 reviewer sign-off を禁止する。dual 必須。bias / blind spot を排除するための物理機構。
- LLM 単独 sign-off を禁止する。最大 1 名のみ + 人間 1 名必須。
- PR author の self-review を禁止する。
- cosign signature なし sign-off を禁止する。必ず cosign で artifact bind。

## dual review の根拠
- tool が verified と判定しても以下は人間 review が必須:
    - spec / model 自体の妥当性（tool が verified と判定しても、spec が実装の意味論と乖離していた場合 verify は無意味）
    - assumption の妥当性（cryptographic assumption が現実の implementation 設定と整合するか）
    - bound parameter の妥当性（Kani / CBMC の bound が小さすぎて real bug を見逃していないか）
    - fairness 仮定の妥当性（liveness proof の fairness が過大でないか）
    - proof の trivial 化（vacuous truth、有用性ゼロの proof）
- 単一 reviewer では bias / blind spot を排除できない、dual sign-off で物理 enforce。

## reviewer pool

### 人間 reviewer
- core team（4 人 minimum）+ domain expert（暗号 / 分散 system / proof assistant の specialist 各 2 名 minimum）
- reviewer pool は `proof_review_assignment.yaml` に登録、PIN（personal identification number）+ cosign keypair で identify

### LLM 補助 reviewer
- Claude / GPT-4 等の LLM を補助 reviewer として採用、ただし dual sign-off の最大 1 名のみ
- LLM の使用は `ai_generator_id` / `model_version` / `prompt_hash` を `proof_review.lock.yaml` に注釈
- LLM 単独 sign-off 禁止

### reviewer 選定
- round-robin 割当 + domain expertise match（暗号 proof は暗号 specialist）
- PR author 自身は reviewer 不可
- 同 PR で過去 90 day 内に他 PR を review した reviewer は cooldown（同一作者の連続 review を避ける）

## proof_review.lock.yaml schema
- 各 entry の field:
    - `obligation_id`: `proof_inventory` との bind
    - `reviewer_1_id`: PIN
    - `reviewer_1_kind`: `["human" | "llm"]`
    - `reviewer_1_sig`: cosign signature
    - `reviewer_1_signed_at`: ISO8601
    - `reviewer_2_id`: PIN
    - `reviewer_2_kind`: `["human" | "llm"]`
    - `reviewer_2_sig`: cosign signature
    - `reviewer_2_signed_at`: ISO8601
    - `llm_metadata`: `{ ai_generator_id, model_version, prompt_hash }`（LLM 使用時のみ）
    - `review_notes_pointer`: review notes artifact の path
    - `dual_signoff_complete`: bool（true は 2 名 sig 完備）

## review checklist（必須項目）

### 妥当性 review
- spec / model が axis spec の意味論を正しく表現しているか
- obligation が「証明されるべき invariant」として critical か（trivial proof でないか）
- bound parameter / fairness assumption が reasonable か
- `assumption.lock.yaml` の参照が正しいか
- `cross_axis_links` が完備か

### artifact review
- proof artifact 本体の可読性（spec の structure、comment の有無）
- tool log に warning / suspicious 出力が無いか
- certificate hash が expected と一致するか
- reproducibility（再 run で certificate hash が一致するか）

### 文書 review
- `cross_axis_links` の更新
- assumption の文献 pointer + DOI の妥当性
- close_kind の妥当性（counter-example closure 時）

## domain expert の特殊 review

### cryptographic proof（11 key / data01 crypto-erase / security15 threat）
- 暗号 specialist の sign-off 必須（reviewer pool の crypto-domain expert）
- assumption が現実の implementation 設定と整合するか（key size、hash function の collision resistance bound、PRP advantage 等）

### distributed system proof（07 transport / 08 migration / infra01 / infra16 / ops15）
- 分散 system specialist の sign-off
- fairness assumption の妥当性、liveness proof の意味論

### program correctness proof（Stainless / Dafny / Lean）
- proof assistant specialist の sign-off
- proof の trivial 化、tactics の悪用がないか

## break-glass review
- emergency（incident response 中）に proof を緊急書き換える場合:
    - OpenBao response wrapping で短期 cosign signing token を発行、`actor` / `reason` / `ticket_ref` / `incident_ref` を `audit_event` subject に emit
    - dual signoff の片方を後追い人間 sign-off に downgrade、24 hour 以内に retro review 完了
    - retro review 失敗時は break-glass entry を rollback（`proof_status` を `unverified_handled` に downgrade）
- break-glass 使用は 13 SLO error budget consumption として記録（`v1_proof_break_glass_count`、ops 14 と整合）

## ChatOps と review tooling
- PR に proof artifact 変更が含まれる場合、ChatOps bot が reviewer を round-robin で割当、各 reviewer に notification。
- reviewer dashboard: Perses で `proof_review.lock.yaml` の current state を可視化、自分の pending review を一覧表示（[formal 運用 UI](../../04_詳細設計/04_運用UI開発者体験/07_formal運用UI.md) 参照）。
- LLM 補助 review: bot が LLM に proof を投入、初期 review notes を生成（人間 reviewer の参考材料）、LLM 単独 sign-off は禁止。

## 採用しない設計
- 単一 reviewer sign-off: 禁止、dual 必須。
- LLM 単独 sign-off: 禁止、最大 1 名のみ + 人間 1 名必須。
- PR author の self-review: 禁止。
- proof artifact 変更なし PR の review skip: 許可（proof 自体に変更が無い場合は通常 PR review のみ）。
- cosign signature なし sign-off: 禁止、必ず cosign で artifact bind。

## 強制機構との bind
- 本方針は [formal 強制機構](../../04_詳細設計/02_強制機構/10_formal強制機構.md) の以下経路で物理 enforce される:
    - 層 B: Conftest による dual_signoff_complete check / human reviewer 含有 check / crypto-expert 含有 check / self-review check / cooldown check
    - 層 D: Kyverno `require-dual-reviewer-signoff` `require-human-reviewer-included` `require-crypto-expert-for-crypto-proof` `require-no-self-review` `require-cooldown-respected` admission policy
    - 層 E: dual reviewer の cosign signature の cryptographic 検証（single-actor forgery が cryptographic に impossible）

## CI 不変条件（proof_review 担当部分）
- 整合 1: `proof_review.lock.yaml` の全 obligation に `dual_signoff_complete=true` が成立、未完了 entry は関連 service deploy block
- 整合 2: reviewer のうち 1 名以上が human kind
- 整合 3: cryptographic proof の reviewer に crypto-domain expert が含まれる
- 整合 4: PR author 自身が reviewer に含まれない
- 整合 5: cooldown 違反（90 day 内の連続 review）が発生した場合 CI alert
- 整合 6: break-glass review の retro review が 24 hour 以内に完了

## 関連参照
- [formal 設計方針 index](README.md)
- [proof_artifact 方針](06_proof_artifact方針.md)
- [counter_example 方針](07_counter_example方針.md)
- [formal 強制機構](../../04_詳細設計/02_強制機構/10_formal強制機構.md)
