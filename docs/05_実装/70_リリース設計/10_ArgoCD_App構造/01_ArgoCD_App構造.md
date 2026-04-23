# 01. ArgoCD App 構造

本ファイルは k1s0 の GitOps 配信経路を担う Argo CD の Application / ApplicationSet 構造を物理配置レベルで確定する。ADR-CICD-001 で選定した Argo CD 2.12+ を前提に、`deploy/apps/` 配下の app-of-apps パターン、tier / 環境別 ApplicationSet 6 本、Helm / Kustomize の二層構成、Argo CD 自体の HA、image-updater の opt-in 運用までを規定する。

![Argo CD App 構造全体像](img/ArgoCD_App構造全体像.svg)

## なぜ app-of-apps + ApplicationSet を併用するのか

Argo CD の配信対象は 6 領域（tier1 / tier2 / tier3 / infra / observability / security）にまたがり、各領域内でさらに 3 環境（dev / staging / prod）の差分と複数コンポーネントの組合せが生じる。Application を手で列挙すると Phase 1a 時点で 100 個を超え、新規コンポーネント追加のたびに Application 追加 PR が発生する。これは 2 名運用で即破綻する。

ルートを 1 つの「app-of-apps」で束ね、その下で領域別 ApplicationSet（Git generator）が `deploy/apps/<tier>/` のディレクトリ構造を自動スキャンする構造にすれば、新規コンポーネントは「ディレクトリを追加して PR を出す」だけで自動検出される。Argo CD の sync 経路・RBAC・監査証跡は一箇所に集約されるため、IMP-REL-POL-001（GitOps 唯一経路）との整合が構造的に担保される。

## `deploy/apps/` のディレクトリ構造

```
deploy/apps/
├── root-app.yaml                 # app-of-apps のルート Application
├── tier1/
│   ├── appset.yaml               # tier1 ApplicationSet（Git generator）
│   ├── t1-state.yaml             # 個別 Application（Helm 参照 + env overlay）
│   ├── t1-secret.yaml
│   ├── t1-workflow.yaml
│   ├── t1-decision.yaml
│   ├── t1-audit.yaml
│   └── t1-pii.yaml
├── tier2/
│   ├── appset.yaml
│   └── <service>.yaml            # tier2 ドメインサービス（Scaffold 生成）
├── tier3/
│   ├── appset.yaml
│   ├── web-portal.yaml
│   ├── bff.yaml
│   └── native-distribution.yaml  # MAUI 配布パイプライン
├── infra/
│   ├── appset.yaml
│   ├── istio-ambient.yaml
│   ├── dapr-control-plane.yaml
│   ├── cloudnativepg.yaml
│   ├── strimzi-kafka.yaml
│   ├── valkey.yaml
│   ├── minio.yaml
│   ├── longhorn.yaml
│   └── metallb.yaml
├── observability/
│   ├── appset.yaml
│   ├── grafana-lgtm.yaml
│   ├── pyroscope.yaml
│   └── otel-collector.yaml
└── security/
    ├── appset.yaml
    ├── keycloak.yaml
    ├── openbao.yaml
    ├── spire.yaml
    ├── cert-manager.yaml
    └── kyverno.yaml
```

`root-app.yaml` は 6 ApplicationSet をまとめて配布する app-of-apps であり、Argo CD が起動直後に読み取る単一の entry point となる。ApplicationSet の追加は root-app を触らずに `root-app.yaml` の `spec.source.directory.recurse=true` により自動検出される。

## 6 ApplicationSet の Git generator 仕様

6 ApplicationSet は同一の骨格（Git generator + Helm / Kustomize 参照）を持つが、`generators.git.files` の pattern と `template.spec.destination.namespace` の導出ルールが異なる。

- **tier1** : `deploy/apps/tier1/*.yaml` を列挙、namespace は `k1s0-tier1`、sync wave 10
- **tier2** : `deploy/apps/tier2/*.yaml` を列挙、namespace は `k1s0-tier2-<service>`（サービスごとに分離）、sync wave 20
- **tier3** : `deploy/apps/tier3/*.yaml`、namespace は `k1s0-tier3`、sync wave 30
- **infra** : `deploy/apps/infra/*.yaml`、namespace は各コンポーネント固有（istio-system / dapr-system 等）、sync wave 0（最先行）
- **observability** : `deploy/apps/observability/*.yaml`、namespace は `k1s0-observability`、sync wave 5
- **security** : `deploy/apps/security/*.yaml`、namespace は各コンポーネント固有（keycloak / openbao / spire-system 等）、sync wave 5

sync wave は「依存する側が後」の原則で設定する。infra が最先行（wave 0）、security / observability が次（wave 5）、tier1 → tier2 → tier3 の順に 10 / 20 / 30 で後続する。同一 wave 内の順序保証はないため、wave 内依存は `dependsOn` annotation または Argo CD の Resource Hook で制御する。

## sync policy：環境別の自動化度

環境ごとに sync policy を変える。これは ADR-CICD-001 で選定済みの Argo CD の Manual（本番）/ Automated（dev/staging）を物理配置に落とし込む部分である。

- **dev** : `automated: {prune: true, selfHeal: true, allowEmpty: false}` + `syncOptions: [CreateNamespace=true, ApplyOutOfSyncOnly=true]`。Git に書いた瞬間に即反映、drift は selfHeal で消す
- **staging** : dev と同構成だが、`automated.selfHeal: false`。drift は検出して通知するが自動消去しない。SRE がレビューして意図的 drift かを判定
- **prod** : `automated: null`（manual sync）+ `syncOptions: [CreateNamespace=true]`。PR merge 後に SRE が Argo CD UI で手動 sync を承認、承認履歴は monitoring 対象

manual sync は「本番で誰が何を反映したか」の監査証跡を強制する意味がある。automated にすると merge 者 = sync 実行者 になり、4-eyes 原則が成立しない。IMP-REL-POL-001 の「唯一経路」は automated / manual の差を貫く規約であり、どちらも `kubectl apply` 直打ちを排除する点は同じである。

## Helm charts と Kustomize overlays の二層構成

コンポーネント本体のマニフェストは Helm chart で記述し、env 別差分は Kustomize overlay で当てる。この二層構成は Argo CD の `source.helm.valueFiles` + `source.plugin` の組合せでは表現しきれない env 差分（例: HorizontalPodAutoscaler の minReplicas の env 別調整、NetworkPolicy の env 別許可 CIDR）を吸収するために採用する。

```
deploy/charts/
├── tier1/
│   ├── Chart.yaml
│   ├── values.yaml               # 既定値
│   ├── values.dev.yaml
│   ├── values.staging.yaml
│   ├── values.prod.yaml
│   └── templates/
│       ├── deployment.yaml
│       ├── rollout.yaml          # Argo Rollouts CRD
│       ├── service.yaml
│       ├── analysis-template-ref.yaml
│       └── ...
└── tier2-service-template/       # tier2 汎用 chart（Scaffold がコピー）

deploy/kustomize/
├── base/
│   └── tier1/                    # Helm 出力を kustomize で base 化
└── overlays/
    ├── dev/
    ├── staging/
    └── prod/
```

Application は `source.helm.chart` ではなく `source.path: deploy/kustomize/overlays/<env>/<tier>` を指定し、overlay 内部で `helmCharts:` を使って chart を展開する構成とする。これにより「chart 本体 → Helm 展開 → Kustomize patch → 最終マニフェスト」という 3 段パイプラインが Argo CD 上で単一 source として扱える。

## Argo CD 自体の HA 構成

Argo CD 自体がダウンすると GitOps 経路全体が停止するため、HA は Phase 0 時点で必須である。次の構成で配置する（`infra/argocd/` 配下、App-of-apps の対象外で bootstrap 時に別途適用）。

- **argocd-server** : 3 replicas、`podAntiAffinity` で node 分散、Ingress 経由で MetalLB 公開
- **argocd-repo-server** : 3 replicas、Git clone キャッシュを共有 PVC（Longhorn）上に保持、大規模リポジトリの clone 時間を短縮
- **argocd-application-controller** : 3 replicas（sharding 有効）、controller の CPU 消費が大きいため shard 分割で水平スケール
- **argocd-dex-server** : 2 replicas、Keycloak（ADR-SEC-001）への OIDC middleman
- **argocd-notifications-controller** : 1 replica（状態保持は etcd 側）
- **backing store** : Redis HA ではなく Valkey Sentinel（ADR-DATA-004）。3 sentinel + 3 replica の標準構成

Argo CD 自体のアップグレードは自己反映を避けるため、`infra/argocd/bootstrap/` 配下に Helm chart を置き、SRE が手動で helm upgrade を実行する（app-of-apps の対象外）。この「Argo CD は Argo CD を管理しない」分離は、アップグレード失敗時の rollback 経路を確保するために必須である。

## image-updater の opt-in 運用

Argo CD Image Updater は「新 image tag が push されたら Git の Application マニフェストを書き換えて sync を誘発する」ツールであり、CI → image push → 自動反映のパイプラインを完成させる。ただし本番での自動反映は「誰が何をいつ反映したか」が人の承認を伴わないため、Phase 0 では dev / staging のみ opt-in とする。

- `deploy/image-updater/config.yaml` : 対象 Application のリスト（dev / staging の tier2 / tier3 のみ）
- tier1 は Phase 1b で opt-in 検討。tier1 は広範囲に影響するため、Phase 0 では手動 tag 更新 PR + manual sync を維持
- image tag 書換は `git-write-back` モードで Application マニフェストの `spec.source.helm.parameters` を更新する
- Renovate（ADR-DEP-001）との棲み分け: Renovate は依存ライブラリ / Dockerfile base image の更新 PR を出す、Image Updater は自プロジェクトの image tag 更新を自動化する

## drift 検知と PagerDuty 連動

IMP-REL-POL-001 の「Git に存在しない差分は self-heal または PagerDuty 通知」の実装は、Argo CD Notifications + PagerDuty Webhook で行う。`deploy/apps/notifications/` 配下に `argocd-notifications-cm.yaml` を配置し、次のトリガを定義する。

- `on-sync-failed` : sync 失敗時に PagerDuty（Sev3）+ Slack 通知
- `on-health-degraded` : health=Degraded 移行時に PagerDuty（Sev2）+ Slack 通知
- `on-out-of-sync` (prod のみ) : drift 検知時に Slack 通知（自動 sync しない prod での drift は常に人手介入を要する）
- `on-deployed` : 正常同期完了時に Slack 通知（#k1s0-deploys チャネル）

## 対応 IMP-REL ID

本ファイルで採番する実装 ID は以下とする。

- `IMP-REL-ARG-010` : `deploy/apps/root-app.yaml` の app-of-apps 構成
- `IMP-REL-ARG-011` : 6 ApplicationSet（tier1 / tier2 / tier3 / infra / observability / security）の配置と sync wave
- `IMP-REL-ARG-012` : 環境別 sync policy（dev automated selfHeal / staging automated no-selfHeal / prod manual）
- `IMP-REL-ARG-013` : Helm charts + Kustomize overlays の二層構成（`deploy/charts/` + `deploy/kustomize/`）
- `IMP-REL-ARG-014` : Argo CD 自体の HA 構成（server x3 / repo-server x3 / controller x3 shard）
- `IMP-REL-ARG-015` : Argo CD の backing store を Valkey Sentinel に統一
- `IMP-REL-ARG-016` : image-updater の opt-in 運用（dev / staging の tier2 / tier3 のみ）
- `IMP-REL-ARG-017` : Argo CD Notifications による drift / health / sync イベントの PagerDuty / Slack 連動

## 対応 ADR / DS-SW-COMP / NFR

- ADR: [ADR-CICD-001](../../../02_構想設計/adr/ADR-CICD-001-argocd.md)（Argo CD）/ [ADR-STOR-001](../../../02_構想設計/adr/ADR-STOR-001-longhorn.md)（Longhorn）/ [ADR-DATA-004](../../../02_構想設計/adr/ADR-DATA-004-valkey.md)（Valkey）/ [ADR-SEC-001](../../../02_構想設計/adr/ADR-SEC-001-keycloak.md)（Keycloak OIDC）
- DS-SW-COMP: DS-SW-COMP-135（配信系）
- NFR: NFR-A-CONT-001（SLA 99%）/ NFR-D-MTH-002（Canary / Blue-Green）

## 関連章との境界

- [`00_方針/01_リリース原則.md`](../00_方針/01_リリース原則.md) の IMP-REL-POL-001（GitOps 唯一経路）の物理配置を本ファイルで固定する
- [`../20_ArgoRollouts_PD/01_ArgoRollouts_PD設計.md`](../20_ArgoRollouts_PD/01_ArgoRollouts_PD設計.md) が本ファイルの Helm chart 内 `rollout.yaml` を参照する
- [`../30_flagd_フィーチャーフラグ/`](../30_flagd_フィーチャーフラグ/) の flagd も本ファイルの infra ApplicationSet 経由で配信される
- [`../../00_ディレクトリ設計/`](../../00_ディレクトリ設計/) の IMP-DIR-* と `deploy/` 配下配置が整合する
