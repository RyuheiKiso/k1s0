# 05. ops 配置（Runbook / Chaos / DR）

本ファイルは `ops/` 配下の配置を確定する。運用領域 ― Runbook / Chaos / DR / Oncall / Load test / Scripts を集約する。

## ops/ の役割

`infra/`（素構成）と `deploy/`（配信定義）の下流で、実際に運用する際の手順・シナリオ・スクリプトを置く場所。採用側の小規模運用前提（NFR-C-NOP-001）で、手順化・自動化の依存度が極めて高い。

- **Runbook**: インシデント対応手順、日常運用手順
- **Chaos**: Chaos Engineering シナリオ（Litmus）
- **DR**: Disaster Recovery 手順・自動スクリプト
- **Oncall**: オンコール体制・連絡先・エスカレーション
- **Load**: 負荷試験シナリオ（k6）
- **Scripts**: 一般運用スクリプト

## レイアウト

```text
ops/
├── README.md
├── runbooks/
│   ├── README.md                # Runbook 索引
│   ├── incidents/
│   │   ├── SEV1-tier1-outage.md
│   │   ├── SEV2-data-layer-degraded.md
│   │   ├── SEV3-observability-backend-down.md
│   │   └── SEV4-single-pod-crash.md
│   ├── daily/
│   │   ├── morning-health-check.md
│   │   ├── backup-verification.md
│   │   └── certificate-expiry-check.md
│   ├── weekly/
│   │   ├── chaos-experiment-review.md
│   │   └── slo-burn-rate-review.md
│   ├── monthly/
│   │   ├── patch-management.md
│   │   └── dr-drill.md
│   └── templates/
│       └── runbook-template.md   # 新 Runbook 作成テンプレ
├── chaos/
│   ├── README.md
│   ├── experiments/
│   │   ├── pod-delete/
│   │   │   ├── tier1-facade.yaml
│   │   │   └── tier2-service.yaml
│   │   ├── network-latency/
│   │   │   └── data-layer-delay.yaml
│   │   ├── cpu-hog/
│   │   └── disk-fill/
│   ├── workflows/                # ChaosWorkflow（シナリオ連結）
│   │   ├── monthly-game-day.yaml
│   │   └── quarterly-region-failure.yaml
│   └── probes/
│       ├── http-probe-tier1.yaml
│       └── k8s-probe-dapr.yaml
├── dr/
│   ├── README.md
│   ├── scenarios/
│   │   ├── pg-restore.md
│   │   ├── kafka-topic-restore.md
│   │   ├── minio-tenant-restore.md
│   │   └── cluster-rebuild.md
│   ├── scripts/
│   │   ├── restore-pg-from-barman.sh
│   │   ├── restore-minio-from-archive.sh
│   │   └── rebuild-cluster-from-scratch.sh
│   └── drills/
│       └── DR-drill-YYYYMM.md    # 実施記録
├── oncall/
│   ├── README.md
│   ├── rotation/
│   │   └── 2026-Q2.yaml          # PagerDuty Schedule export
│   ├── escalation.md
│   ├── contacts.md
│   └── sops-key/                 # SOPS 暗号化鍵（AGE）
│       └── age.txt.enc
├── load/
│   ├── README.md
│   ├── k6/
│   │   ├── tier1-state-api.js
│   │   ├── tier1-pubsub-api.js
│   │   ├── tier3-web-portal.js
│   │   └── helpers/
│   └── scenarios/
│       ├── steady-state.md       # 恒常負荷試験
│       ├── spike.md
│       └── soak.md
└── scripts/
    ├── bootstrap-cluster.sh      # k8s クラスタ初期化
    ├── rotate-tls-certs.sh
    ├── harvest-logs.sh
    └── lib/
        └── common.sh
```

## runbooks/ の構造

### incidents/

Sev1〜Sev4 のインシデント対応手順。タイプ D（技術説明書）形式に従い、以下の構成:

1. **該当事象の症状**: 何が観測されたら該当するか
2. **初動対応**（5 分以内）: ダメコン
3. **原因調査**: 既知パターンのチェックリスト
4. **復旧手順**: コマンドレベルで具体的に
5. **事後対応**: ポストモーテム記録先

### daily / weekly / monthly

定期的な運用 Runbook。

- **daily**: 毎朝の health check（SLO burn rate、バックアップ成功、証明書期限）
- **weekly**: Chaos 実験結果レビュー、SLO 週次サマリ
- **monthly**: パッチ適用、DR ドリル、セキュリティ監査

## chaos/ の構造

LitmusChaos（OSS Chaos Engineering）を採用。**バージョンは v3.x を pin する**（v1.13 系の `apiVersion: litmuschaos.io/v1alpha1` は Maintenance-only で新 API が凍結されているため）。以下のサンプルは v3.11 time のシェイプに対応する。`apiVersion` と `probe` の記法は v1alpha1 / v3 で互換性があるが、`ChaosResult` / `ChaosCenter` など新 CRD は v3 以降でのみ有効。v4 以降のリリースで非互換変更が入る場合は ADR-OPS-002 を改訂する。

### experiments/

単体実験（Pod delete、Network latency、CPU hog、Disk fill）。

```yaml
# chaos/experiments/pod-delete/tier1-facade.yaml
# LitmusChaos v3.x 向け。ChaosHub の experiment を hub reference で取り込む方式も併用可能
apiVersion: litmuschaos.io/v1alpha1
kind: ChaosEngine
metadata:
  name: tier1-facade-pod-delete
  namespace: k1s0-tier1
spec:
  appinfo:
    appns: k1s0-tier1
    applabel: "app=tier1-facade-service-invoke"
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
              url: http://tier1-facade-service-invoke.k1s0-tier1.svc:8080/healthz
              method:
                get:
                  criteria: ==
                  responseCode: "200"
            mode: Continuous
```

v3 系では probe は `spec.experiments[].spec.probe` の位置（experiment 配下）に置くのが最新の推奨。v1alpha1 の `spec.probe`（ChaosEngine 直下）でも互換動作するが、新規記述は experiment 配下に揃える。

### workflows/

月次 game day シナリオ、四半期 region failure シナリオを ChaosWorkflow で連結。

## dr/ の構造

### scenarios/

典型的な DR シナリオを Markdown で文書化。

- PG 全損からの Barman 復元
- Kafka Topic 消失からの MirrorMaker 復元
- MinIO Tenant 消失からの archive 復元
- クラスタ全損からのベアメタル再構築（OpenTofu + `dr/scripts/rebuild-cluster-from-scratch.sh`）

### drills/

DR ドリル実施記録。`DR-drill-YYYYMM.md` として月次 / 四半期毎に保存。

## oncall/ の構造

### rotation/

PagerDuty Schedule の export（Terraform で管理）。

### escalation.md / contacts.md

Sev1 発生時の escalation path、連絡先リスト。contacts.md は個人情報を含むため Kyverno policy で該当 namespace の読み取り制限。

### sops-key/

SOPS AGE 鍵。オンコール担当のみが復号可能。

## load/ の構造

k6 でシナリオを JavaScript で記述。

```javascript
// load/k6/tier1-state-api.js
import http from 'k6/http';
import { check, sleep } from 'k6';

export const options = {
  stages: [
    { duration: '5m', target: 100 },   // ramp-up
    { duration: '30m', target: 100 },  // steady
    { duration: '5m', target: 0 },     // ramp-down
  ],
  thresholds: {
    http_req_duration: ['p(99)<500'],
    http_req_failed: ['rate<0.01'],
  },
};

export default function () {
  const res = http.get('https://api.k1s0.internal/v1/state/users/123');
  check(res, { 'status is 200': (r) => r.status === 200 });
  sleep(1);
}
```

## scripts/ の構造

運用スクリプト。POSIX sh 原則、依存は `lib/common.sh` に集約。

## 対応 IMP-DIR ID

- IMP-DIR-OPS-095（ops 配置 Runbook/Chaos/DR）

## 対応 ADR / DS-SW-COMP / 要件

- ADR-OPS-001（Runbook 標準化）
- ADR-OPS-002（LitmusChaos 採用）
- NFR-C-NOP-\* / NFR-A-AVL-\*
