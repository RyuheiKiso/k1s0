# ops — 運用領域（Runbook / Chaos / DR / Oncall / Load）

ADR-DIR-001/002 と Runbook 設計（タイプ C: 検出 / 初動 / 復旧 / 原因調査 / 事後処理 5 段構成）に基づき、
**運用ライフサイクルの実体ファイル** を集約する。
詳細設計は [`docs/05_実装/00_ディレクトリ設計/60_operationレイアウト/05_ops配置_Runbook_Chaos_DR.md`](../docs/05_実装/00_ディレクトリ設計/60_operationレイアウト/05_ops配置_Runbook_Chaos_DR.md)。

## 配置

```text
ops/
├── runbooks/                                       # 5 段構成 Runbook（タイプ C）
│   ├── daily/                                      # 日次運用（バックアップ確認 / 容量チェック）
│   ├── weekly/                                     # 週次運用（fuzz nightly / chaos drill）
│   ├── monthly/                                    # 月次運用（DR 演習 / 棚卸し）
│   ├── incidents/                                  # インシデント対応 Runbook
│   ├── secret-rotation.md                          # OpenBao 経由のキーローテ手順
│   └── templates/                                  # Runbook 雛形
├── chaos/                                          # LitmusChaos
│   ├── experiments/                                # 個別実験定義
│   ├── probes/                                     # SLO 連動 probe
│   └── workflows/                                  # 連続実験ワークフロー
├── dr/                                             # DR（Disaster Recovery）
│   ├── drills/                                     # 半期 1 回の DR 演習記録
│   ├── scenarios/                                  # 想定シナリオ集（cluster ロス / data loss）
│   └── scripts/                                    # 復旧自動化スクリプト
├── oncall/
│   ├── rotation/                                   # PagerDuty 連動の当番表
│   └── sops-key/                                   # SOPS 暗号鍵の運用手順（OpenBao 経由）
├── load/
│   ├── k6/                                         # k6 負荷テスト（NFR-B-PERF 連動）
│   └── scenarios/                                  # 採用組織別シナリオ
└── scripts/lib/                                    # 共通シェルライブラリ
```

## Runbook の 5 段構成（タイプ C）

各 Runbook は以下を必ず含む。空セクションは認めない（`docs-postmortem` Skill と整合）:

1. **検出（Detection）**: アラートトリガ / Grafana ダッシュボード / 異常パターン
2. **初動（Triage）**: 5 分以内の暫定対応 / トラフィック切替 / scale up
3. **復旧（Recovery）**: ロールバック / 切戻し / 完全復旧
4. **原因調査（Root Cause）**: ログ / トレース / メトリクスの収集ポイント
5. **事後処理（Post-mortem）**: ポストモーテム作成 / 再発防止アクション

## 主要 Runbook（リリース時点 整備対象）

FMEA で RPN ≥ 30 とされた 10 故障モードに対応する Runbook を **リリース時点までに整備** する
（[docs/04_概要設計/55_運用ライフサイクル方式設計/06_FMEA分析方式.md](../docs/04_概要設計/55_運用ライフサイクル方式設計/06_FMEA分析方式.md)）。

## 関連設計

- [docs/04_概要設計/55_運用ライフサイクル方式設計/03_環境構成管理方式.md](../docs/04_概要設計/55_運用ライフサイクル方式設計/03_環境構成管理方式.md)
- [docs/04_概要設計/55_運用ライフサイクル方式設計/06_FMEA分析方式.md](../docs/04_概要設計/55_運用ライフサイクル方式設計/06_FMEA分析方式.md)
- [docs/04_概要設計/55_運用ライフサイクル方式設計/08_Runbook設計方式.md](../docs/04_概要設計/55_運用ライフサイクル方式設計/08_Runbook設計方式.md)
- [docs/40_運用ライフサイクル/](../docs/40_運用ライフサイクル/) — Runbook 索引
