# CLAUDE

## ポリシー
技術的な判断で悩んだ際は至高を目指す判断を優先すること。

## 変更履歴的な表現について
この文章が削除されるまで変更履歴を明文化しないでください。

## 11 Phase 実装順序（strict 直列 / enforcement first）

全フェーズを単一実装者が直列で完遂する。各 Phase の exit criteria は `release_gate.lock.yaml` の対応 cell が green になること。

| Phase | 名称 | 主要 deliverable | 前提 |
|---|---|---|---|
| P0 | Tooling | `tools/lock_yaml_generator/` — generate_release_gate.py を含む生成器群を実装 | なし |
| P1 | Meta | `axis_registry.lock.yaml`（19 軸 / cap 20 / 残 1）を手書き + validate。`00_軸登録適合仕様` の物理化 | P0 |
| P2 | B 層 lint | `tools/docs_lint/` に markdownlint / textlint / frontmatter_validator / xref_check を実装。CI 統合して全 .md が lint green になること | P1 |
| P3 | 物理 root | monorepo root の CI pipeline / Dockerfile / Makefile / .github/workflows/ の骨格。全 Phase の CI 基盤 | P2 |
| P4 | test scaffold | test 軸の `coverage_matrix.lock.yaml`（18 軸 × 5 verification_class = 90 cell）・`regression_corpus.lock.yaml` の scaffold | P3 |
| P5 | Formal 前置 | tier1 transport の TLA+ temporal_safety_proof + data Stainless invariant の 2 cell のみ先行 proof。残 93 cell は P11 | P4 |
| P6 | 物理 enforcement | Kyverno 25+ policy / cosign signing policy / Harbor admission / OpenBao Transit 設定 | P5 |
| P7 | crosscutting | 13 cross-cutting 適合仕様の compile 強制（protoc / Buf codegen / FSM codegen / HTTP/2 enforcement 等） | P6 |
| P8 | infra | infra 軸実装（cluster 位相 / 時刻整合 / PTP / eBPF / topology drill） | P7 |
| P9 | data | data 軸実装（PostgreSQL RLS / pgaudit / Barman / ClickHouse / Debezium CDC / PII cluster） | P8 |
| P10 | tier1→tier2→tier3→security→ops→client | アプリ軸 6 本を sequential に実装。tier1（9 適合仕様）→ tier2（テナント分離）→ tier3（クライアント状態）→ security（脅威モデル/provenance）→ ops（運用ループ）→ client（SDK 配布） | P9 |
| P11 | proof matrix + dual signoff | 残 93 proof cell + cross-cutting cell を全て verified or accepted_with_assumption にし、release_gate.lock.yaml の全 19 軸 AND-gate を green にして cosign signed tag を発行 | P10 |

P10 の 6 軸（tier1/tier2/tier3/security/ops/client）は "tier1 through 6番目の軸" の意。test 軸は P4 で scaffold 済み、formal 軸は P5/P11 で対応、infra/data は P8/P9 で対応済み。