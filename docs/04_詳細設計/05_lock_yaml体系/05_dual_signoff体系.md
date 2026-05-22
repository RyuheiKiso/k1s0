---
id: detail.meta.dual_signoff_system
axis: meta
phase: detail
kind: detail
status: draft
depends_on:
  - detail.formal.formal_conformance
  - detail.formal.release_gate_system
covered_by:
  defense_in_depth_layers: [A, B, E]
  proof_classes: []
lock_artifacts:
  - dual_review.lock.yaml
trace:
  fr_ids:
  - FR-meta-006

---

# dual signoff 体系

## 一文方針

単一実装者体制において「human author × 1 + (ai_static_analysis | cosign_history) × 1」の二重 sign を物理要件とし、LLM 単独 sign-off を禁止することで release_gate AND-gate の dual signoff cell を green にする。

## reviewer_kind enum

| kind | 定義 | evidence 要件 |
|---|---|---|
| `human` | 人間（実装者本人）の cosign 署名 | cosign_signature_uri（Fulcio OIDC keyless or HSM 鍵） |
| `ai_static_analysis` | AI モデルが実施した静的解析の証拠 | ai_evidence_uri（Tekton Pipeline `ai-static-analysis-run` の output artifact URI） |
| `cosign_history` | 同一 subject_digest 系列の過去 cosign 履歴 | cosign_history_uri（Ceph RGW path、predecessor ≥ 3 件必須） |

## 最低構成

dual signoff の最低要件は以下のいずれか:
- (a) `human × 1 + ai_static_analysis × 1`
- (b) `human × 1 + cosign_history × 1`

cryptographic proof は `kind=ai_static_analysis` のみでは不足とし、必ず `cosign_history` を (a) と並行して取得するか v2 で `human_external` kind を拡張する。

## LLM 単独 sign-off 禁止原則

- `kind=ai_static_analysis` のみ 2 名（human 0）は CI fail
- PR author 自身を human スロットに充てる（single-author 体制下では author = human reviewer）

## cooldown 規則

- human reviewer 間の 90 day cooldown 規則は v1 では事実上 vacuous（single-author 下では無条件満足）
- single-author 下での ai_evidence 使い回し防止: `cooldown_check.ai_evidence_diff_sha256 != cooldown_check.last_sign_by_same_author.ai_evidence_sha256` を CI assertion する

## dual_review.lock.yaml スキーマ

```yaml
subject_digest: "sha256:<hex>"
reviewers:
  - signer_id: "<github_handle or model_id>"
    reviewer_kind: human | ai_static_analysis | cosign_history
    signed_at: "ISO8601"
    cosign_signature_uri: "<uri>"
    ai_evidence_uri: "<uri>"         # kind=ai_static_analysis 時のみ
    cosign_history_uri: "<uri>"      # kind=cosign_history 時のみ
cooldown_check:
  last_sign_by_same_author: "<ai_evidence_sha256 or null>"
  ai_evidence_diff_sha256: "<sha256>"
```

詳細 jsonschema は `docs/00_format/dual_review_schema.yaml` を参照。

## ci check cell

| cell | 条件 |
|---|---|
| `meta.release_gate_dual_signoff_complete` | 全 PR の `dual_review.lock.yaml` が schema 適合 |
| `meta.dual_signoff_human_slot_nonzero` | reviewers に kind=human が 1 件以上 |
| `meta.dual_signoff_ai_evidence_fresh` | ai_evidence_uri が当 PR の subject_digest を含む |
| `meta.dual_signoff_cosign_history_depth_3` | cosign_history_uri で predecessor ≥ 3 件 |
| `meta.dual_signoff_no_ai_only_approval` | kind=ai_static_analysis のみ 2 名は reject |
| `meta.dual_signoff_no_evidence_reuse` | ai_evidence_diff_sha256 が前回と異なる |
| `meta.dual_signoff_schema_valid` | dual_review.lock.yaml が dual_review_schema.yaml に適合 |
| `meta.dual_signoff_subject_digest_bound` | ai_evidence 内 subject_digest が PR の subject と一致 |

## v2 拡張ルール

v2 では `human_external` kind を追加し、外部監査機関による cryptographic proof の peer review を可能にする。v1 では単一実装者 + AI evidence + cosign_history で代替。
