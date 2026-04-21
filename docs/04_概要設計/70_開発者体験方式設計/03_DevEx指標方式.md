# 03. DevEx 指標方式

本ファイルは k1s0 の開発者体験（Developer Experience、DevEx）を定量的に計測・改善するための指標体系を定める。対象は DORA Four Keys（Deployment Frequency / Lead Time / MTTR / Change Failure Rate）の 4 指標、SPACE Framework（Satisfaction / Performance / Activity / Communication / Efficiency）の 5 次元、計測基盤（GitHub API + Argo CD API + Grafana）、レポート頻度（週次自動 / 月次レビュー）、改善ループの 5 領域である。

## 本ファイルの位置付け

プラットフォーム製品は「使われて初めて価値が生じる」。DevEx を計測せず改善サイクルを回さないと、tier2 / tier3 開発者が「k1s0 を使うより自前で作った方が早い」と判断して離脱し、基盤投資が無駄になる。一方で計測のための計測に陥ると、運用 2 名体制を圧迫する。本設計は「自動収集 + 週次レポート + 月次改善」という最小構成で改善サイクルを維持する。

本設計は構想設計 ADR-OBS-001（Grafana LGTM 採用）と ADR-CICD-002（Argo CD GitOps）を前提として、DORA 4 指標の目標値（Deployment Frequency 1 日 10 回以上 / Lead Time 1h 以内 / MTTR 4h 以内 / CFR 15% 以下）を企画コミットとして Phase 1c で達成可能な水準に維持する設計に落とす。対応要件は DX-MET-001〜006、DX-GP-003（10 分ルール）、NFR-I-SLI-001 / NFR-I-SLO-001〜011（SLI/SLO 運用）である。

## DORA Four Keys の採用

DORA（DevOps Research and Assessment）の 4 指標は、世界中の開発組織のベンチマークとして確立している。k1s0 も同指標を採用することで、業界標準と比較可能な健康診断を実現する。

指標と目標値は以下の通りとする。

| 指標 | 定義 | 目標 | 業界 Elite 水準 | 計測方法 |
| --- | --- | --- | --- | --- |
| Deployment Frequency | prod への本番デプロイ頻度 | 1 日 10 回以上（Phase 1c 以降） | 1 日複数回 | Argo CD History |
| Lead Time for Changes | コミットから prod 到達までの時間 | 1 時間以内 | 1 時間以内 | GitHub + Argo CD |
| Mean Time To Recovery | インシデント発生から復旧までの時間 | 4 時間以内 | 1 時間以内 | Incident Log + Grafana |
| Change Failure Rate | prod デプロイのうち障害を引き起こした割合 | 15% 以下 | 5% 以下 | Incident Log |

Deployment Frequency の 1 日 10 回は、10 サービス × 1 日 1 回 = 10 回、または 1 サービス × 1 日 10 回のいずれでも達成できる水準である。Phase 1c（2 サービス × 5 回）で達成し、Phase 2 で 5 サービス以上に拡大する計画である。業界 Elite 水準（1 日複数回）に合致する設計とする。

Lead Time の 1 時間は [01_CI_CD方式.md](01_CI_CD方式.md) のパイプライン時間予算（PR 5 分 + レビュー 15 分 + main 10 分 + Argo CD 3 分 + Rollouts 30 分 = 63 分、緩衝込みで 1 時間）と整合する。この内訳を全設計書で共有し、予算超過時の改善対象を明確化する。

MTTR の 4 時間は企画コミットの RTO 4 時間と同じ水準で、[../55_運用ライフサイクル方式設計/02_インシデント対応方式.md](../55_運用ライフサイクル方式設計/02_インシデント対応方式.md) の Runbook 実行時間と整合する。MTTR を 4 時間以内に抑えるため、第 1 階層（Feature Flag OFF）と第 2 階層（Argo Rollouts ロールバック）を 5 分以内で実行できる設計を維持する。

CFR の 15% は業界 Elite 水準（5%）には届かないが、Phase 1c 着手時点での現実的な水準とする。Phase 2 で Canary 戦略と Chaos テストを導入し、Phase 2 末で 5% 到達を目標とする。

## SPACE Framework の補完

DORA 4 指標だけでは「開発者が実際に満足しているか」が捕捉できない。SPACE Framework（Satisfaction / Performance / Activity / Communication / Efficiency）で補完する。

Satisfaction（満足度）は四半期アンケートで計測する。「k1s0 基盤は業務で役立っているか」（5 段階）、「tier1 API は使いやすいか」（5 段階）、「Golden Path 10 分ルールは実現されているか」（はい / いいえ）、「改善してほしい点」（自由記述）の 4 項目を標準とする。目標は平均 3.5 以上、Phase 1c で 3.0 到達、Phase 2 で 3.5 到達とする。

Performance（成果）は DORA の Lead Time と Deployment Frequency が該当するため、重複を避けて本次元は「開発者 1 人当たりのサービスリリース数」を計測する。目標は月 5 サービス以上（Phase 2 時点、5 名チーム想定）である。

Activity（活動量）は GitHub 上の PR 数・コミット数・レビュー数を集計する。過度なアクティビティは燃え尽きのサインとなるため、「1 人当たり週 10〜20 PR」を健全範囲とし、これを逸脱するチームを早期発見する。

Communication（コミュニケーション）は PR レビューの平均応答時間、Slack / Teams のスレッド応答時間を計測する。目標は PR レビュー 24 時間以内、Slack 4 時間以内（営業時間内）である。応答遅延が多い場合、チームの分担や通知設定を見直す。

Efficiency（効率性）は中断（interruption）頻度、コンテキストスイッチの頻度を計測する。Phase 2 で導入予定の「フォーカスタイム」指標（会議以外の連続 2 時間以上のブロック）を個人レベルで可視化し、深い作業時間を確保する。

## 計測基盤の設計

指標は全自動で収集することを必須とする。手動エクスポートや Excel 集計は運用 2 名体制を圧迫するため禁止する。

計測ソースは以下の 5 系統である。第 1 に GitHub API（PR 数・レビュー時間・コミット）、第 2 に Argo CD API（デプロイ数・デプロイ時刻）、第 3 に Grafana（5xx エラー率・MTTR 計算のための障害期間）、第 4 に Incident Log（Backstage プラグイン、障害発生・解消時刻）、第 5 に四半期アンケート（Backstage Survey プラグイン）、である。

計測パイプラインは以下の通り。日次バッチで GitHub API / Argo CD API からイベントを取得し、PostgreSQL の `devex_metrics` テーブルに格納する。Grafana の Prometheus データソースを介して DORA 指標を集計する。Incident Log は PagerDuty 代替の自作システム（Phase 1c）で管理し、障害時刻と復旧時刻をシステムが自動記録する。Backstage ダッシュボードで 4 指標 + 5 次元を統合表示する。

ダッシュボードの公開範囲は全社員とする。開発者自身のチームの指標だけでなく、他チームの指標も参照できる。これは「比較による自発的改善」を促す設計である。ただし、個人レベルの指標（Activity / Efficiency の内訳）はプライバシー配慮でチーム長のみ参照可能とする。

## レポート頻度と改善ループ

週次レポートは毎週月曜朝に自動生成し、Slack / Teams の `#devex-weekly` チャンネルに配信する。内容は前週の 4 指標 + 目標比・主要な障害・目標未達項目のリスト、である。配信は人間の介在なく自動で実行する。

月次レビューは Product Council（起案者 + アーキテクト + 運用担当者 + 各チームリード）で実施する。アジェンダは以下の 5 項目で 30 分以内に収める。第 1 に DORA 4 指標の月次推移（グラフ 1 枚）、第 2 に SPACE 5 次元のサマリ（満足度アンケートは四半期のため適用月のみ）、第 3 に目標未達の原因分析（ブロッカー特定）、第 4 に次月の改善タスク（担当者付き）、第 5 にクローズした改善タスクの効果測定、である。

改善タスクは GitHub Issue で管理し、Backstage の TechDocs でガイドライン化する。例えば「Lead Time 1h 未達の場合、PR レビュー待ち時間を 24 時間 → 12 時間に短縮する」「CFR 15% 超過の場合、Canary 戦略の分析指標を強化する」などの標準的な対処を事前定義する。

## 計測粒度とプライバシー

4 指標 + 5 次元は全社集約・チーム別・サービス別・個人別の 4 粒度で集計する。全社集約は経営層向け、チーム別は Product Council 向け、サービス別は運用チーム向け、個人別はチーム長のみ参照可能、とする。

個人別の Activity（PR 数・コミット数）は「監視指標」ではなく「自己把握指標」として位置付ける。チーム長が個人の生産性評価にこれらを使うことを禁止する規範を [../75_事業運用方式設計/10_ガバナンス運用方式.md](../75_事業運用方式設計/10_ガバナンス運用方式.md) で明文化する。これは「指標のグッドハート化」（指標を追うあまり本来の目的が失われる）を防ぐ設計である。

## 目標未達時のアクション

DORA 4 指標のいずれかが目標未達となった場合、以下の標準アクションを起動する。

Deployment Frequency 未達（1 日 10 回未満が 2 週連続）の場合、原因を「CI/CD の不安定さ」「レビュー待ち時間過大」「テスト長時間化」の 3 カテゴリに分類し、該当カテゴリの改善タスクを優先する。

Lead Time 未達（1h 超過が 2 週連続）の場合、パイプライン時間予算（PR 5 分 / main 10 分 / Argo CD 3 分 / Rollouts 30 分）を計測し、超過段階を特定して改善する。

MTTR 未達（4h 超過が 1 件以上）の場合、該当インシデントのポストモーテムを必須化し、Runbook に追記する。ポストモーテムは [../55_運用ライフサイクル方式設計/02_インシデント対応方式.md](../55_運用ライフサイクル方式設計/02_インシデント対応方式.md) で詳述する。

CFR 未達（15% 超過が月次で発生）の場合、直近のリリース内容を Review し、テスト漏れ・Canary 戦略の不備・依存パッチ適用ミスの 3 カテゴリに分類して再発防止策を定義する。

## 設計 ID 一覧

| 設計 ID | 設計項目 | 確定フェーズ | 対応要件 |
| --- | --- | --- | --- |
| DS-DEVX-MET-001 | DORA Four Keys の採用 | Phase 1b | DX-MET-001 |
| DS-DEVX-MET-002 | Deployment Frequency 目標（1 日 10 回以上） | Phase 1c | DX-MET-002 |
| DS-DEVX-MET-003 | Lead Time 目標（1h 以内） | Phase 1b | DX-MET-003 |
| DS-DEVX-MET-004 | MTTR 目標（4h 以内） | Phase 1c | DX-MET-004 |
| DS-DEVX-MET-005 | CFR 目標（15% 以下 → 5% 以下） | Phase 1c | DX-MET-005 |
| DS-DEVX-MET-006 | SPACE 5 次元補完 | Phase 2 | DX-MET-006 |
| DS-DEVX-MET-007 | 計測基盤（GitHub + Argo + Grafana） | Phase 1b | DX-MET-001 |
| DS-DEVX-MET-008 | Backstage ダッシュボード統合 | Phase 1c | DX-BS-003 |
| DS-DEVX-MET-009 | 週次自動レポート | Phase 1b | DX-MET-001 |
| DS-DEVX-MET-010 | 月次 Product Council レビュー | Phase 1c | DX-MET-001 |
| DS-DEVX-MET-011 | 4 粒度集計（全社 / チーム / サービス / 個人） | Phase 1c | DX-MET-006 |
| DS-DEVX-MET-012 | グッドハート化防止規範 | Phase 1c | DX-MET-006 |

## 対応要件一覧

本ファイルは要件定義書 50_開発者体験 DX-MET-001〜006（DevEx 指標）に直接対応する。DX-GP-003（10 分ルール）、NFR-A-REC-002（Runbook 充足率を通じた MTTR 担保）、NFR-C-NOP-001（通常運用における開発者体験）、NFR-I-SLI-001 / NFR-I-SLO-001〜011（SLO 運用）とも連動する。構想設計 ADR-OBS-001（Grafana LGTM）、ADR-CICD-002（Argo CD GitOps）を前提とする。企画書で合意した DORA 4 指標の数値コミット（Deployment Frequency 1 日 10 回・Lead Time 1h・MTTR 4h・CFR 15%）を本設計が機械的に達成可能な仕組みとして具体化する。
