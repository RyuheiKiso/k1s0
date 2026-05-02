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
│   ├── README.md                # Runbook 索引（RB-* 全件）
│   ├── secret-rotation.md       # RB-SEC-004 兼用、定期 + 漏洩時即応
│   ├── incidents/               # RB-* incident Runbooks（命名規則: RB-<カテゴリ>-<NNN>-<短縮名>.md）
│   │   ├── RB-API-001-tier1-latency-high.md
│   │   ├── RB-DB-001-valkey-node-failover.md
│   │   ├── RB-DB-002-postgres-primary-failover.md
│   │   ├── RB-MSG-001-kafka-broker-failover.md
│   │   ├── RB-MSG-002-dlq-backlog.md
│   │   ├── RB-SEC-001-openbao-raft-failover.md
│   │   ├── RB-SEC-002-cert-expiry.md
│   │   ├── RB-SEC-003-audit-tampering.md
│   │   ├── RB-SEC-005-pii-leak-detection.md
│   │   ├── RB-SEC-006-tenant-boundary-breach.md
│   │   ├── RB-AUTH-001-keycloak-db-failover.md
│   │   ├── RB-AUTH-002-auth-abuse-detection.md
│   │   ├── RB-NET-001-istio-ztunnel-failover.md
│   │   ├── RB-NET-002-envoy-dos.md
│   │   ├── RB-OPS-001-cicd-pipeline-down.md
│   │   ├── RB-OPS-002-argocd-out-of-sync.md
│   │   ├── RB-OPS-003-ntp-drift.md
│   │   ├── RB-COMP-001-legal-disclosure.md
│   │   ├── RB-COMP-002-pii-regulatory-disclosure.md
│   │   ├── RB-INC-001-severity-decision-tree.md
│   │   └── RB-WF-001-temporal-nondeterminism.md
│   ├── daily/                   # 定常運用（DAILY-*）
│   │   ├── morning-health-check.md
│   │   ├── backup-verification.md
│   │   ├── certificate-expiry-check.md
│   │   └── error-code-alert-policy.md
│   ├── weekly/                  # 定常運用（WEEKLY-*）
│   │   ├── chaos-experiment-review.md
│   │   └── slo-burn-rate-review.md
│   ├── monthly/                 # 定常運用（MONTHLY-*）+ AUD
│   │   ├── patch-management.md
│   │   ├── dr-drill.md
│   │   ├── infra-disposal.md
│   │   └── RB-AUD-001-audit-hash-monthly.md
│   ├── postmortems/             # ポストモーテム記録（YYYY-MM-DD-RB-*.md）
│   │   └── README.md
│   └── templates/
│       └── runbook-template.md  # 新 Runbook 作成テンプレ（必須 8 セクション）
├── chaos/
│   ├── README.md
│   ├── experiments/             # 単体実験（ChaosEngine）
│   │   ├── pod-delete/
│   │   │   ├── tier1-facade.yaml
│   │   │   ├── cnpg-primary.yaml
│   │   │   └── kafka-broker.yaml
│   │   ├── network-latency/
│   │   │   └── tier1-tier2.yaml
│   │   ├── cpu-hog/
│   │   │   └── tier1-facade.yaml
│   │   └── disk-fill/
│   │       └── cnpg-primary.yaml
│   ├── workflows/               # ChaosWorkflow（シナリオ連結）
│   │   ├── monthly-game-day.yaml
│   │   └── quarterly-region-failure.yaml
│   └── probes/                  # 共通 probe 定義
│       ├── http-probe-tier1.yaml
│       └── k8s-probe-dapr.yaml
├── dr/
│   ├── README.md
│   ├── scenarios/
│   │   ├── RB-DR-001-cluster-rebuild.md  # クラスタ全壊 RB-DR-001 兼用
│   │   ├── pg-restore.md
│   │   ├── kafka-topic-restore.md
│   │   └── minio-tenant-restore.md
│   ├── scripts/
│   │   ├── restore-pg-from-barman.sh
│   │   ├── restore-minio-from-archive.sh
│   │   └── rebuild-cluster-from-scratch.sh
│   └── drills/
│       └── DR-drill-YYYY-Qn.md  # 実施記録（四半期 table-top + 半期 実機）
├── oncall/
│   ├── README.md
│   ├── rotation/
│   │   ├── README.md            # PagerDuty Schedule 整備手順
│   │   └── 2026-Q2.yaml         # PagerDuty Schedule export（採用契約後に作成）
│   ├── escalation.md            # SEV1 30 分タイムライン プロセス
│   ├── contacts.md              # 連絡先一覧（Kyverno で読取制限）
│   └── sops-key/                # SOPS AGE 鍵
│       ├── README.md
│       └── age.txt.enc          # OpenBao Transit で暗号化（採用後の運用拡大時 で導入）
├── load/                        # k6 負荷シナリオ
│   ├── README.md
│   ├── k6/
│   │   └── helpers/
│   │       └── common.js        # 共通 helper（環境変数 / endpoint / auth）
│   └── scenarios/
│       ├── state_baseline.js    # State Save/Get/Delete baseline (NFR-B-PERF-001)
│       ├── rate_limit.js        # RateLimit interceptor 検証
│       └── idempotency_replay.js # idempotency_key dedup 検証
├── scripts/
│   ├── bootstrap-cluster.sh     # 新規 k8s クラスタ初期化
│   ├── rotate-tls-certs.sh      # TLS 証明書一括 rotate（cert-manager / SPIRE / Strimzi）
│   ├── harvest-logs.sh          # インシデント対応用ログ収集
│   ├── rollback.sh              # Argo CD + Rollouts undo 1 コマンド rollback (IMP-REL-RB-050)
│   └── lib/
│       └── common.sh
└── supply-chain/                # SBOM + cosign 署名（ADR-SUP-001）
    ├── README.md
    ├── keys/                    # offline cosign 鍵（リリース時点 OSS 検証用、本番は keyless）
    ├── sbom/                    # syft 生成（CycloneDX 1.6 + SPDX）
    └── signatures/              # cosign sign-blob bundle
```

> 本レイアウトは リリース時点 整備実態（2026-05）を反映する正典である。Runbook 命名規則は
> [`docs/04_概要設計/55_運用ライフサイクル方式設計/08_Runbook設計方式.md`](../../../04_概要設計/55_運用ライフサイクル方式設計/08_Runbook設計方式.md) §「Runbook の形式」、
> 目録は [`09_Runbook目録方式.md`](../../../04_概要設計/55_運用ライフサイクル方式設計/09_Runbook目録方式.md) に従う。
> Runbook 内容（必須 8 セクション + YAML frontmatter）は同 08 §「必須 8 セクション」を参照。

## runbooks/ の構造

### incidents/

[`docs/04_概要設計/55_運用ライフサイクル方式設計/08_Runbook設計方式.md`](../../../04_概要設計/55_運用ライフサイクル方式設計/08_Runbook設計方式.md) §「必須 8 セクション」に従い、以下の構成:

1. **前提条件**（権限・ツール・環境）
2. **対象事象**（アラート発火条件 / 観測症状）
3. **初動手順**（5 分以内）
4. **原因特定手順**
5. **復旧手順**（暫定 + 恒久）
6. **検証手順**（復旧後の正常稼働確認）
7. **予防策**（再発防止）
8. **関連 Runbook**

YAML frontmatter で `runbook_id / title / category / severity / owner / automation /
alertmanager_rule / fmea_id / estimated_recovery / last_updated` を必須化。
雛形は [`templates/runbook-template.md`](../../../../ops/runbooks/templates/runbook-template.md) 参照。

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
