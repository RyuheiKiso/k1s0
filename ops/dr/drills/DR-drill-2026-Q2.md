# DR-drill-2026-Q2: 四半期 table-top 演習記録（雛形）

> **状態**: 雛形（リリース時点の運用開始時に初回演習を実施し、本ファイルを記録として確定する）

## メタデータ

| 項目 | 値 |
|---|---|
| 開催日時 | YYYY-MM-DD HH:MM JST |
| 種別 | Table-top exercise |
| シナリオ ID | RB-DR-001（クラスタ全壊） |
| 想定 RTO | 4 時間 |
| 想定 RPO | 24 時間 |
| 参加者 | 起案者、協力者、EM |
| ファシリテーター | 起案者 |
| 期間 | 2 時間（00:00〜02:00） |

## シナリオ

提示された災害想定を `<...>` プレースホルダで埋めて記述:

> 例: 2026-XX-XX 03:42 JST、本番 GKE クラスタ `k1s0-prod`（asia-northeast1）の Control Plane 全 3 ノードが応答喪失。
> Cloud Provider Status Page で「asia-northeast1 全体障害」を確認。等

## 実行ステップと所要時間

| ステップ | 想定 | 実測 | 備考 |
|---|---|---|---|
| 災害宣言 + IC 指名 | 5 分 | - 分 | |
| Phase 1: k8s 作成 | 60 分 | - 分 | gcloud CLI でリージョン切替 |
| Phase 2: CNPG リストア | 90 分 | - 分 | Barman archive 取得時間 |
| Phase 3: アプリ復旧 | 30 分 | - 分 | argocd app sync × 4 |
| Phase 4: DNS 切替 | 15 分 | - 分 | Cloudflare API |
| 検証 | 30 分 | - 分 | RB-DR-001 §6 全項目確認 |
| **合計** | 240 分 | - 分 | RTO 4h 達成 ?: <Yes/No> |

## 発見された問題

1. <問題 1: 例: gcloud CLI のサービスアカウント鍵が手元になく、Phase 1 で 15 分ロス>
2. <問題 2: 例: ArgoCD App of Apps の bootstrap 手順が infra/k8s/bootstrap/ に未文書化>

## 改善 Action Item

| # | 内容 | 担当 | 期限 | Issue/PR |
|---|---|---|---|---|
| 1 | <例: gcloud SA 鍵を SOPS で `ops/dr/scripts/lib/` に保管> | 起案者 | YYYY-MM-DD | PR-XXX |

## 参考リンク

- [RB-DR-001 シナリオ](../scenarios/RB-DR-001-cluster-rebuild.md)
- [DS-OPS-ENV-012](../../../docs/04_概要設計/55_運用ライフサイクル方式設計/03_環境構成管理方式.md)
- [NFR-A-REC-001 / NFR-A-REC-002](../../../docs/03_要件定義/30_非機能要件/A_可用性.md)
