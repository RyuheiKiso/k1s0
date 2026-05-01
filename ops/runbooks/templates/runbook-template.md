---
runbook_id: RB-XXX-NNN
title: <短い件名>
category: API|DB|NET|SEC|OPS|MSG|AUTH|BKP|DR|AUD|WF|COMP|INC
severity: SEV1|SEV2|SEV3
owner: 起案者|協力者
automation: manual|argo-workflow|temporal
alertmanager_rule: <Alertmanager rule 名>
fmea_id: FMEA-NNN|間接対応
estimated_recovery: 暫定 N 分 / 恒久 N 時間
last_updated: YYYY-MM-DD
---

# RB-XXX-NNN: <Runbook タイトル>

<本 Runbook の目的を 2〜3 行で記述。対応する NFR / FMEA / ADR を必ず明記する。>

## 1. 前提条件

- 実行者の権限要件（Kubernetes RBAC、OpenBao policy 等）。
- 必要ツール（kubectl / argocd / cmctl / 等）とバージョン下限。
- kubectl context が `k1s0-prod` であることを `kubectl config current-context` で確認。
- 依存サービスの起動状態（Operator / 監視基盤）。
- staging で先に試した結果が手元にあること（プロダクション影響のある手順は staging 検証必須）。

## 2. 対象事象

- Alertmanager `<rule-name>` 発火（PromQL 条件: `<expr>`）、または
- 手動観測（`kubectl get ...` で X が Y 状態）、または
- 外部から報告（顧客 / セキュリティリサーチャー）。

検知シグナル:

```promql
# <監視対象の PromQL>
<query expression>
```

ダッシュボード: **Grafana → <ダッシュボード名>**。
通知経路: PagerDuty `<schedule>` → Slack `#<channel>`。

## 3. 初動手順（5 分以内）

最初の 5 分で <何を判定するか> を完了する。

```bash
# 状態確認
kubectl get <resource> -n <namespace> -o wide
```

```bash
# ログ確認
kubectl logs -n <namespace> <pod> --tail=100
```

ステークホルダー通知: Slack `#status` に「<状況>」を投稿（5 分以内）。SEV1 なら [`oncall/escalation.md`](../../oncall/escalation.md) を起動。

## 4. 原因特定手順

```bash
# 詳細ログ・イベント・メトリクス確認
kubectl describe <resource> -n <namespace>
kubectl get events -n <namespace> --sort-by='.lastTimestamp'
```

よくある原因:

1. **<原因 1>**: <観測方法> で確認。対処: <recovery への分岐>。
2. **<原因 2>**: ...
3. **<原因 3>**: ...

エスカレーション: 上記に該当しない場合は L3 起案者へ Slack で連絡。

## 5. 復旧手順

暫定復旧（業務影響を最小化、目標 N 分以内）:

```bash
# 暫定対応コマンド（例: failover、rolling restart、scale down）
<command>
```

恒久復旧（根本原因解消後の正常化、目標 N 時間以内）:

```bash
# 設定変更 / コード修正のデプロイ
<command>
```

各ステップは 3 分以内で完了するサイズに保つ（08_Runbook設計方式 §粒度設計 参照）。3 分超は分割。

## 6. 検証手順

復旧完了の判定基準（全項目満たして Resolved 遷移可能）:

- <指標 1> が <閾値> を 5 分間継続（Grafana で確認）。
- <指標 2> が <閾値>。
- 直近 10 分の Loki クエリ `{namespace="..."}|= "ERROR"` が 0 件。
- アプリケーションのヘルスチェック（`/healthz` 200）。
- 主要機能の動作確認（<具体テスト>）。

## 7. 予防策

- ポストモーテム起票（24 時間以内、`postmortems/<YYYY-MM-DD>-RB-XXX-NNN.md`）。
- 監視強化（閾値見直し、追加アラート）の Jira チケット化。
- 設定改善 / コード修正案を PR で提出。
- 訓練計画: 月次 Chaos Drill 対象に追加（`ops/chaos/workflows/monthly-game-day.yaml`）。
- NFR-A-REC-002 の MTTR ログを更新。

## 8. 関連 Runbook

- 関連設計書: `<path>`
- 関連 ADR: [ADR-XXX-NNN](../../../docs/02_構想設計/adr/ADR-XXX-NNN-<slug>.md)
- 関連 NFR: [NFR-X-XXX-NNN](../../../docs/03_要件定義/30_非機能要件/X_<カテゴリ>.md)
- 関連 FMEA: [FMEA-NNN](../../../docs/04_概要設計/55_運用ライフサイクル方式設計/06_FMEA分析方式.md)
- 連鎖 Runbook:
  - [`<RB-OTHER-NNN>.md`](<RB-OTHER-NNN>.md) — <連鎖条件>

---

> 本テンプレートは [`docs/04_概要設計/55_運用ライフサイクル方式設計/08_Runbook設計方式.md`](../../../docs/04_概要設計/55_運用ライフサイクル方式設計/08_Runbook設計方式.md) §「必須 8 セクション」と
> [`09_Runbook目録方式.md`](../../../docs/04_概要設計/55_運用ライフサイクル方式設計/09_Runbook目録方式.md) §「YAML front-matter」に基づく。
> 本テンプレートを使用する際は、`<...>` プレースホルダを実値で埋め、不要なコメント行を削除すること。
