---
id: req.non_functional.slo_requirement
axis: tier1
phase: requirement
kind: requirement
status: draft
depends_on:
  - req.non_functional.non_functional_index
  - detail.tier1.slo_conformance
covered_by:
  defense_in_depth_layers: []
  proof_classes: []
---

# SLO 要件

## 一文方針
- 6 slo_class（v1_request_availability / v1_request_latency_p99 / v1_request_latency_p99_cross_region / v1_event_freshness / v1_workflow_completion / v1_data_durability）の bundle で全 SLO instance を機械可読に固定。MWMBR alert + error budget 物理 freeze（Kyverno admission policy）を必須要件とする。

## 6 slo_class 要件
| class | sli_kind | target | window |
|---|---|---|---|
| `v1_request_availability` | availability | 99.9% | 30d |
| `v1_request_latency_p99` | latency | p99 ≤ class.target | 30d |
| `v1_request_latency_p99_cross_region` | latency | p99 ≤ class.target（典型 1500ms） | 30d |
| `v1_event_freshness` | freshness | p95 ≤ class.target | 7d |
| `v1_workflow_completion` | completion_rate | 99.5% | 30d |
| `v1_data_durability` | durability | 11n + freeze_on_any_loss | 365d |

## MWMBR alert
- 全 SLO に multi-window multi-burn-rate alert 必須
- false positive 抑制と早期検出の両立
- 単窓 alert 禁止

## error budget の物理 enforcement
- error budget 枯渇時に Kyverno admission policy で deploy 物理 freeze
- 文章運用ではなく物理機構

## 受入条件
- 6 slo_class × 全 instance の完全 conformance green
- Litmus chaos scenario で burn-rate alert fire 検証 green
- error budget 物理 freeze property test green

## 関連参照
- [非機能要件 index](README.md)
- [SLO 適合仕様](../../04_詳細設計/01_適合仕様/07_SLO適合仕様.md)
- [SLO protection layers](../../04_詳細設計/03_クロスカッティング適合仕様/05_SLO_protection_layers.md)
