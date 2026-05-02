# runbooks — Runbook 索引（タイプ C / 必須 8 セクション）

本ディレクトリは k1s0 の Runbook 集を集約する索引である。
[`docs/04_概要設計/55_運用ライフサイクル方式設計/08_Runbook設計方式.md`](../../docs/04_概要設計/55_運用ライフサイクル方式設計/08_Runbook設計方式.md)（必須 8 セクション、命名規則）と
[`docs/04_概要設計/55_運用ライフサイクル方式設計/09_Runbook目録方式.md`](../../docs/04_概要設計/55_運用ライフサイクル方式設計/09_Runbook目録方式.md)（RB-* 16 本目録、所有者割当、月次レビュー）に対応する。

## 配置

```text
runbooks/
├── README.md                      # 本ファイル（索引）
├── secret-rotation.md             # Secret Rotation 共通手順（RB-SEC-004 兼用）
├── incidents/                     # インシデント対応 Runbook（RB-* + ops 独自）
├── daily/                         # 日次運用 Runbook
├── weekly/                        # 週次運用 Runbook
├── monthly/                       # 月次運用 Runbook
├── templates/                     # Runbook 雛形
└── postmortems/                   # ポストモーテム記録（インシデント発生時）
```

## RB-* 目録（[09_Runbook目録方式.md](../../docs/04_概要設計/55_運用ライフサイクル方式設計/09_Runbook目録方式.md) 16 本）

| RB-ID | 対象事象 | Severity | 所有者 | FMEA | 状態 | ファイル |
|------|------|------|------|------|------|------|
| RB-API-001 | tier1 API レイテンシ劣化 | SEV2〜3 | 起案者 | 間接 | 未整備 | `incidents/RB-API-001-tier1-latency-high.md`（予定） |
| RB-DB-001 | Valkey クラスタノード障害 | SEV3 | 協力者 | FMEA-007 | 未整備 | `incidents/RB-DB-001-valkey-node-failover.md`（予定） |
| RB-DB-002 | PostgreSQL Primary 障害 | SEV1 | 起案者 | FMEA-006 | ✅ 整備済 | [`incidents/RB-DB-002-postgres-primary-failover.md`](incidents/RB-DB-002-postgres-primary-failover.md) |
| RB-MSG-001 | Kafka ブローカー障害 | SEV2 | 協力者 | FMEA-003 | ✅ 整備済 | [`incidents/RB-MSG-001-kafka-broker-failover.md`](incidents/RB-MSG-001-kafka-broker-failover.md) |
| RB-SEC-001 | OpenBao Raft リーダ選出失敗 | SEV1 | 起案者 | FMEA-002 | 未整備 | `incidents/RB-SEC-001-openbao-raft-failover.md`（予定） |
| RB-AUTH-001 | Keycloak DB 障害 | SEV2 | 協力者 | FMEA-004 | 未整備 | `incidents/RB-AUTH-001-keycloak-db-failover.md`（予定） |
| RB-NET-001 | Istio ztunnel 障害 | SEV2 | 協力者 | FMEA-005 | 未整備 | `incidents/RB-NET-001-istio-ztunnel-failover.md`（予定） |
| RB-SEC-002 | 証明書期限切れ | SEV1 | 起案者 | FMEA-010 | ✅ 整備済 | [`incidents/RB-SEC-002-cert-expiry.md`](incidents/RB-SEC-002-cert-expiry.md) |
| RB-NET-002 | Envoy Gateway DoS | SEV1 | 起案者 | 間接 | 未整備 | `incidents/RB-NET-002-envoy-dos.md`（予定） |
| RB-SEC-003 | 監査ログ改ざん検知 | SEV1 | 起案者 | FMEA-009 | 未整備 | `incidents/RB-SEC-003-audit-tampering.md`（予定） |
| RB-SEC-004 | Secret 漏洩発覚 | SEV1 | 起案者 | 間接 | 部分整備 | [`secret-rotation.md`](secret-rotation.md) §「漏洩発生時」（要 RB-SEC-004 専用化） |
| RB-OPS-001 | CI/CD パイプライン停止 | SEV3 | 協力者 | 間接 | 未整備 | `incidents/RB-OPS-001-cicd-pipeline-down.md`（予定） |
| RB-OPS-002 | Argo CD Out-of-Sync 長期化 | SEV3 | 協力者 | 間接 | 未整備 | `incidents/RB-OPS-002-argocd-out-of-sync.md`（予定） |
| RB-BKP-001 | Backup 失敗 | SEV2 | 協力者 | 間接 | 未整備 | `incidents/RB-BKP-001-backup-failure.md`（予定） |
| RB-DR-001 | Disaster Recovery 発動 | SEV1 | 起案者 | FMEA-008 | ✅ 整備済 | [`../dr/scenarios/RB-DR-001-cluster-rebuild.md`](../dr/scenarios/RB-DR-001-cluster-rebuild.md) |
| RB-AUD-001 | 監査証跡ハッシュチェーン整合性月次検証 | SEV1 | 起案者 | FMEA-009 系統 | 未整備 | `monthly/RB-AUD-001-audit-hash-monthly.md`（予定） |

リリース時点 整備済 4 / 16 本。残 12 本は順次整備（[09_Runbook目録方式.md](../../docs/04_概要設計/55_運用ライフサイクル方式設計/09_Runbook目録方式.md) §「Runbook 総数の推移計画」に従う）。

## ops 独自 Runbook（採番拡張対象）

正典 16 本に含まれないが、運用上重要であり、リリース時点で 09_Runbook目録方式.md への追加採番候補としているもの。

| 候補 RB-ID | 対象事象 | Severity | ファイル | 採番方針 |
|------|------|------|------|------|
| RB-AUTH-002 | 認証悪用 / Secret 大量読取検知 | SEV1〜2 | [`incidents/auth-abuse-detection.md`](incidents/auth-abuse-detection.md) | docs 目録追加待ち |
| RB-MSG-002 | DLQ 滞留 | SEV2 | [`incidents/dlq-backlog.md`](incidents/dlq-backlog.md) | docs 目録追加待ち |
| RB-COMP-001 | 法的開示対応 | SEV1 | [`incidents/legal-disclosure.md`](incidents/legal-disclosure.md) | 新カテゴリ COMP 追加待ち |
| RB-SEC-005 | PII 漏えい検知 | SEV1 | [`incidents/pii-leak-detection.md`](incidents/pii-leak-detection.md) | docs 目録追加待ち |
| RB-COMP-002 | PII 漏えい規制報告 | SEV1 | [`incidents/pii-regulatory-disclosure.md`](incidents/pii-regulatory-disclosure.md) | 新カテゴリ COMP 追加待ち |
| RB-INC-001 | Severity 判定木（ヘルパー） | 横断 | [`incidents/severity-decision-tree.md`](incidents/severity-decision-tree.md) | ヘルパーとして RB-INC-* 新採番 |
| RB-WF-001 | Temporal NonDeterministicWorkflowError | SEV2 | [`incidents/temporal-determinism-error.md`](incidents/temporal-determinism-error.md) | DS-OPS-RB-012 で言及済、新採番 |
| RB-SEC-006 | テナント越境検知 | SEV1 | [`incidents/tenant-boundary-breach.md`](incidents/tenant-boundary-breach.md) | docs 目録追加待ち |

これら 8 本は内容品質は高い（5 段構成、PromQL、kubectl 完備）が、現状は旧形式（5 段）であり、リリース時点までに必須 8 セクション形式へ移行。
[09_Runbook目録方式.md](../../docs/04_概要設計/55_運用ライフサイクル方式設計/09_Runbook目録方式.md) の「Runbook 総数の推移計画」では リリース時点 20 本がストレッチゴール。8 本追加で 16 + 8 = 24 本となるため、棚卸しで重複統合する余地がある。

## daily / weekly / monthly Runbook

| 周期 | ファイル | 状態 |
|------|------|------|
| daily | `daily/morning-health-check.md` | 未整備 |
| daily | `daily/backup-verification.md` | 未整備 |
| daily | `daily/certificate-expiry-check.md` | 未整備 |
| daily | [`daily/error-code-alert-policy.md`](daily/error-code-alert-policy.md) | ✅ 整備済（正典外、エラーコード × 閾値マトリクス） |
| weekly | `weekly/chaos-experiment-review.md` | 未整備 |
| weekly | `weekly/slo-burn-rate-review.md` | 未整備 |
| monthly | `monthly/patch-management.md` | 未整備 |
| monthly | `monthly/dr-drill.md` | 未整備 |
| monthly | [`monthly/infra-disposal.md`](monthly/infra-disposal.md) | ✅ 整備済（正典外、インフラ廃棄・暗号消去） |
| monthly | `monthly/RB-AUD-001-audit-hash-monthly.md` | 未整備（DS-OPS-RB-056） |

## ファイル命名規則

[`docs/04_概要設計/55_運用ライフサイクル方式設計/08_Runbook設計方式.md`](../../docs/04_概要設計/55_運用ライフサイクル方式設計/08_Runbook設計方式.md) §「Runbook の形式」:

- インシデント Runbook: `RB-<カテゴリ>-<通番>-<簡潔名>.md`（例: `RB-DB-002-postgres-primary-failover.md`）
- カテゴリ: API / DB / NET / SEC / OPS の 5 種が正典。MSG / AUTH / BKP / DR / AUD / WF / COMP / INC は採用後の運用拡大時で正典に追加予定。
- 雛形は [`templates/runbook-template.md`](templates/runbook-template.md) を参照。

## YAML front-matter（必須）

```yaml
---
runbook_id: RB-DB-002
title: PostgreSQL Primary 障害対応（CNPG failover）
category: DB
severity: SEV1
owner: 起案者
automation: manual                    # or argo-workflow / temporal
alertmanager_rule: PostgresPrimaryDown
fmea_id: FMEA-006
estimated_recovery: 暫定 10 分 / 恒久 4 時間
last_updated: 2026-05-02
---
```

## 必須 8 セクション

[`docs/04_概要設計/55_運用ライフサイクル方式設計/08_Runbook設計方式.md`](../../docs/04_概要設計/55_運用ライフサイクル方式設計/08_Runbook設計方式.md) §「必須セクション」:

1. 前提条件（権限・ツール・環境）
2. 対象事象（アラート発火条件 / 観測症状）
3. 初動手順（5 分以内）
4. 原因特定手順
5. 復旧手順（暫定 + 恒久）
6. 検証手順（復旧後の正常稼働確認）
7. 予防策（再発防止）
8. 関連 Runbook

## ポストモーテム

ポストモーテム記録は [`postmortems/`](postmortems/) 配下に `<YYYY-MM-DD>-RB-*-<slug>.md` で配置（`docs-postmortem` Skill 経由で起票）。

## 関連

- 関連設計書: [`docs/04_概要設計/55_運用ライフサイクル方式設計/06_FMEA分析方式.md`](../../docs/04_概要設計/55_運用ライフサイクル方式設計/06_FMEA分析方式.md), [`08_Runbook設計方式.md`](../../docs/04_概要設計/55_運用ライフサイクル方式設計/08_Runbook設計方式.md), [`09_Runbook目録方式.md`](../../docs/04_概要設計/55_運用ライフサイクル方式設計/09_Runbook目録方式.md)
- 関連 NFR: [NFR-A-REC-002（Runbook 15 本整備）](../../docs/03_要件定義/30_非機能要件/A_可用性.md)
- 関連 ADR: [ADR-OBS-003 Incident Taxonomy](../../docs/02_構想設計/adr/ADR-OBS-003-incident-taxonomy.md)
- 関連スキル: [`docs-postmortem`](../../.claude/skills/docs-postmortem/)（ポストモーテム起票時）
- 索引: 本 README 配下の RB-* 一覧テーブル
