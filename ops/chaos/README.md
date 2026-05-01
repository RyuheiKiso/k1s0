# chaos — Chaos Engineering（LitmusChaos v3.x）

本ディレクトリは k1s0 の Chaos Engineering 実験定義を集約する。
[`docs/05_実装/00_ディレクトリ設計/60_operationレイアウト/05_ops配置_Runbook_Chaos_DR.md`](../../docs/05_実装/00_ディレクトリ設計/60_operationレイアウト/05_ops配置_Runbook_Chaos_DR.md) §「chaos/ の構造」、
および [`docs/04_概要設計/55_運用ライフサイクル方式設計/06_FMEA分析方式.md`](../../docs/04_概要設計/55_運用ライフサイクル方式設計/06_FMEA分析方式.md) §「FMEA アウトプット活用 — テスト設計」に対応する。

## 採用 OSS

**LitmusChaos v3.x** を採用する（v1.13 系の `apiVersion: litmuschaos.io/v1alpha1` は Maintenance-only で新 API が凍結されているため）。
v3 系では `ChaosResult` / `ChaosCenter` 等の新 CRD が利用可能。本ディレクトリの YAML は v3.11 のシェイプに対応する。

## 配置

```text
chaos/
├── README.md                     # 本ファイル
├── experiments/                  # 単体実験定義（ChaosEngine）
│   ├── pod-delete/               # Pod 削除実験（FMEA-001/002/004/005/006/007 検証）
│   ├── network-latency/          # ネットワーク遅延実験（FMEA-008 検証）
│   ├── cpu-hog/                  # CPU 飽和実験
│   └── disk-fill/                # ディスク満杯実験（FMEA-001/003/006 検証）
├── probes/                       # SLO 連動 probe（httpProbe / k8sProbe）
└── workflows/                    # ChaosWorkflow（複数実験連結）
    ├── monthly-game-day.yaml     # 月次ゲームデー（毎月最終水曜日 14:00 JST）
    └── quarterly-region-failure.yaml  # 四半期リージョン障害シミュレーション
```

## Chaos Drill カデンス

[`docs/04_概要設計/55_運用ライフサイクル方式設計/08_Runbook設計方式.md`](../../docs/04_概要設計/55_運用ライフサイクル方式設計/08_Runbook設計方式.md) §「検証: Chaos Drill」に従う。

| カデンス | 種別 | 対象 | 起動方法 |
|------|------|------|------|
| 月次 | 計画的ドリル | RB-* 4〜5 本（四半期で全 16 本網羅） | `workflows/monthly-game-day.yaml` を ArgoCD で sync |
| 四半期 | 抜き打ちドリル | SEV1 系 Runbook（RB-DB-002 / RB-SEC-002 等） | 起案者または EM が PagerDuty で手動発火 |
| 週次 | 自動化ドリル（採用後の運用拡大時） | 軽微な障害（Pod 削除、ネットワーク遅延 50ms） | CronWorkflow で自動 |

実施結果は `ops/chaos/results/<YYYY-MM-DD>-<scenario>.md` に記録（リリース時点 はゼロ件、整備順次）。

## experiments/ の実装ガイドライン

各実験は ChaosEngine + 対象アプリラベル + probe（成否判定）の 3 要素で定義する。

```yaml
# 例: chaos/experiments/pod-delete/tier1-facade.yaml
apiVersion: litmuschaos.io/v1alpha1
kind: ChaosEngine
metadata:
  name: tier1-facade-pod-delete
  namespace: k1s0-tier1
spec:
  appinfo:
    appns: k1s0-tier1
    applabel: "app=tier1-facade"
    appkind: deployment
  chaosServiceAccount: litmus-admin
  experiments:
    - name: pod-delete
      spec:
        components:
          env:
            - name: TOTAL_CHAOS_DURATION
              value: "60"
            - name: CHAOS_INTERVAL
              value: "10"
            - name: FORCE
              value: "false"
        probe:
          - name: tier1-facade-http-probe
            type: httpProbe
            httpProbe/inputs:
              url: http://tier1-facade.k1s0-tier1.svc:8080/healthz
              method:
                get:
                  criteria: ==
                  responseCode: "200"
            mode: Continuous
```

## probes/ の責務

probe は実験中に SLO を継続監視し、SLO 違反時は実験を即時 abort する。

- httpProbe: tier1 facade `/healthz` を 1 秒間隔で polling、5xx 連続 3 回で abort。
- k8sProbe: 対象 Pod の Ready 状態を監視。
- promProbe: Mimir に PromQL クエリ送信、エラーバジェット消費率 > 20%/h で abort。

## workflows/ の責務

ChaosWorkflow は複数 ChaosEngine を順次・並列に組合せ、シナリオ全体を 1 リソースで管理する。

- `monthly-game-day.yaml`: 月次で 5 実験（pod-delete / network-latency / cpu-hog / disk-fill / cert-expiry）を 60 分間で実行。
- `quarterly-region-failure.yaml`: 四半期で AZ 1 つの全 Node を network-loss する。

## リリース時点までの整備状況

リリース時点では本 README 以下のサブディレクトリは `.gitkeep` のみで、実体ファイルは順次追加。
リリース時点で必須整備するのは以下 4 ファイル（FMEA RPN ≥ 30 のうち動的検証可能なもの）:

- `experiments/pod-delete/tier1-facade.yaml` — RB-API-001 / RB-DB-002 連動
- `experiments/pod-delete/cnpg-primary.yaml` — FMEA-006 検証
- `experiments/network-latency/tier1-tier2.yaml` — FMEA-008 検証
- `workflows/monthly-game-day.yaml` — 月次ドリル

## 関連

- 関連 ADR: [ADR-OBS-003 Incident Taxonomy](../../docs/02_構想設計/adr/ADR-OBS-003-incident-taxonomy.md)
- 関連設計書: [`docs/04_概要設計/55_運用ライフサイクル方式設計/06_FMEA分析方式.md`](../../docs/04_概要設計/55_運用ライフサイクル方式設計/06_FMEA分析方式.md), [`08_Runbook設計方式.md`](../../docs/04_概要設計/55_運用ライフサイクル方式設計/08_Runbook設計方式.md)
- 関連 NFR: [NFR-A-CONT-003（FMEA 実施）](../../docs/03_要件定義/30_非機能要件/A_可用性.md)
- 関連 Runbook: [`../runbooks/weekly/chaos-experiment-review.md`](../runbooks/weekly/chaos-experiment-review.md)（週次レビュー手順）
