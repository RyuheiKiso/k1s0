# ADR-DIR-002: infra / deploy / ops の 3 階層分離とルート昇格

- ステータス: Accepted
- 起票日: 2026-04-23
- 決定日: 2026-04-23
- 起票者: kiso ryuhei
- 関係者: システム基盤チーム / tier1 開発チーム / 運用チーム / SRE 担当 / GitOps 担当

## コンテキスト

概要設計の DS-SW-COMP-120（[../../04_概要設計/20_ソフトウェア方式設計/01_コンポーネント方式設計/06_パッケージ構成_Rust_Go.md](../../04_概要設計/20_ソフトウェア方式設計/01_コンポーネント方式設計/06_パッケージ構成_Rust_Go.md) 参照）は、Kubernetes manifests / Dapr Components / Helm charts を `src/tier1/infra/` に配置する方針を確定している。これは tier1 のインフラ関連資産を tier1 配下にまとめる前提である。

しかし実装フェーズで運用領域を俯瞰すると、以下の構造的な問題が浮上した。

- **infra（素構成）と deploy（配信定義）と ops（Runbook / Chaos / DR）は責務が異なる**にもかかわらず、全部を `src/tier1/infra/` に押し込む設計になっている。CNCF のプラットフォームエンジニアリング標準（[CNCF Platform Engineering Maturity Model](https://tag-app-delivery.cncf.io/)）では、クラスタ素構成（Kubernetes マニフェスト・ミドルウェア）と GitOps 配信定義（ArgoCD Application・Kustomize overlays）と運用手順（Runbook・Chaos シナリオ）は別階層で管理することが推奨されている。
- **infra 層は tier 横断の共通基盤**であり tier1 所有物ではない。Kafka、CloudNativePG、Keycloak、OpenBao は tier1 ・ tier2 ・ tier3 すべてに使われる。tier1 配下に置くと tier2 / tier3 のインフラ所有者が tier1 チームの承認を得なければ変更できない運用になる。
- **Istio Ambient / Dapr Control Plane / cert-manager 等のクラスタ単位資源**は tier 単位の概念ではなくクラスタ単位の概念であり、どの tier にも属さない。
- **スパースチェックアウト cone** で `tier1-rust-dev` 役割の開発者は tier1 の Rust コードのみ必要で、Kubernetes YAML や Helm chart は不要。`src/tier1/infra/` に混在していると、tier1-rust-dev cone に infra YAML が必然的に混入する。
- **GitOps 配信定義（ArgoCD ApplicationSet）はクラスタ素構成とも運用スクリプトとも別資産**である。配信定義は「どの ApplicationSet がどの環境にどの Helm chart を撒くか」を宣言する。これを infra と混ぜると、ArgoCD Application の PR と Kafka の設定変更 PR が同じディレクトリ階層で混じる。
- **運用 Runbook / Chaos / DR スクリプト**は手順書・実行可能スクリプトであり、ArgoCD が配信する対象ではない。ここを別ディレクトリに分離することで、GitOps の対象範囲と運用手順の管理範囲を明確に分離できる。
- **ベアメタル OpenTofu プロビジョニング**は Kubernetes の前段にあり、さらに別の時間軸（Day -1 のハードウェア立ち上げ）で動くため、配信定義とも分離して管理する必要がある。

Phase 0 稟議承認前に配置を確定することで、実装開始後の大規模な移動コスト（CI path-filter / CODEOWNERS / ArgoCD 検索パスの全書き換え）を回避する。

## 決定

**`src/tier1/infra/` を廃止し、以下の 3 階層に分離してリポジトリルートに昇格する。**

```
k1s0/
├── infra/                       # ★昇格 1: クラスタ素構成
│   ├── k8s/                     # bootstrap / namespaces / networking / storage
│   ├── mesh/                    # istio-ambient / envoy-gateway
│   ├── dapr/                    # control-plane / components（旧 src/tier1/infra/dapr）
│   ├── data/                    # cloudnativepg / kafka / valkey / minio
│   ├── security/                # keycloak / openbao / spire / cert-manager / kyverno
│   ├── observability/           # LGTM + Pyroscope + OTel Collector
│   ├── feature-management/      # flagd
│   ├── scaling/                 # KEDA
│   └── environments/            # dev / staging / prod の環境差分
├── deploy/                      # ★新設 2: GitOps 配信定義
│   ├── apps/                    # ArgoCD Application / ApplicationSet
│   ├── charts/                  # 共通 Helm charts（tier1 / tier2 / tier3）
│   ├── kustomize/               # base + overlays
│   ├── rollouts/                # Argo Rollouts AnalysisTemplate
│   ├── opentofu/                # ベアメタルプロビジョン
│   └── image-updater/           # Argo CD Image Updater 設定
└── ops/                         # ★新設 3: 運用領域
    ├── runbooks/                # Runbook 実装
    ├── chaos/                   # Litmus シナリオ
    ├── dr/                      # DR スクリプト
    ├── oncall/                  # オンコール運用手順
    ├── load/                    # 負荷試験スクリプト
    └── scripts/                 # 運用スクリプト
```

3 階層の責務分担は以下とする。

- **infra/** : クラスタ素構成。Kubernetes の宣言的リソース（Deployment / StatefulSet / ConfigMap 原本）と OSS ミドルウェアの Helm values。ArgoCD がこれらを指す Application を介して撒く対象。
- **deploy/** : GitOps 配信定義。ArgoCD Application / ApplicationSet / Kustomize overlays / Argo Rollouts AnalysisTemplate / OpenTofu のベアメタルプロビジョン。配信の「レシピ」を置く場所。
- **ops/** : 運用領域。Runbook（障害対応手順）/ Chaos シナリオ / DR スクリプト / オンコール手順 / 負荷試験スクリプト。GitOps の配信対象ではなく、人間または CI の手動トリガーで実行される資産。

CODEOWNERS は `infra/` を SRE + tier 横断基盤担当、`deploy/` を GitOps 担当 + SRE、`ops/` を SRE + 運用担当に割り当てる。スパースチェックアウト役割 `infra-ops` は `infra/` + `deploy/` + `ops/` を含み、`tier1-rust-dev` 等は原則として含まない。

## 検討した選択肢

### 選択肢 A: 3 階層分離 + ルート昇格（採用）

- 概要: 上記の通り `infra/` `deploy/` `ops/` の 3 層をリポジトリルートに配置
- メリット:
  - CNCF Platform Engineering 標準に準拠
  - クラスタ素構成・配信定義・運用手順の責務を階層で明示
  - tier 横断資産が tier 所有物でないことをパスで示せる
  - スパースチェックアウト cone が tier 開発と infra-ops で明確に分離できる
  - CODEOWNERS の path-pattern が自然に切れる
  - ArgoCD の検索パスが `deploy/apps/` に限定でき、誤爆リスクが下がる
- デメリット:
  - 概要設計 DS-SW-COMP-120 の改訂が必要
  - リポジトリルート直下のディレクトリ数が増える（`src/` + `infra/` + `deploy/` + `ops/` + `docs/` + `tools/` + `tests/` + `examples/` + `third_party/` = 9 個）

### 選択肢 B: `src/tier1/infra/` 維持

- 概要: 既存 DS-SW-COMP-120 を変更せず、tier1 配下に infra を置く
- メリット:
  - 概要設計の改訂が不要
  - Phase 1a 時点では tier1 のみ実装されるため、tier1 所有物として扱っても破綻しない
- デメリット:
  - Phase 1b 以降で tier2 / tier3 の infra が加わると、tier1 配下に tier2 / tier3 の Kafka 設定等を置く矛盾が生じる
  - Kafka / CloudNativePG / Keycloak がクラスタ単位資源であり tier1 所有物ではないという論理が表現できない
  - スパースチェックアウト cone で tier1-rust-dev に infra YAML が混入する
  - ArgoCD 配信定義を同じ階層に混ぜるのか別にするのか曖昧なまま

### 選択肢 C: `src/infra/` に昇格（tier1 配下から出すが src 配下は維持）

- 概要: `src/` の中に infra を置き、ソースコードと同じ階層に配置
- メリット:
  - リポジトリルート直下のディレクトリ数を抑制
  - tier1 所有物ではないことは表現できる
- デメリット:
  - `src/` は「ソースコードの一次ディレクトリ」という既定方針（[../../../CLAUDE.md](../../../CLAUDE.md) 参照）と不整合（YAML はソースコードではない）
  - Go / Rust / C# / TS のソースと Kubernetes YAML が同じ `src/` に混ざると、cargo / go / dotnet build ツールの検索対象が拡張される恐れ
  - deploy / ops も `src/` に入れるか、infra だけ `src/` に入れるかで一貫性が崩れる

### 選択肢 D: infra + deploy の 2 階層分離（ops は infra 配下に残す）

- 概要: `infra/` と `deploy/` は分離するが、`ops/` は `infra/ops/` として従属
- メリット:
  - リポジトリルートのディレクトリ数を 1 つ節約
  - Runbook が infra に近い場所にある
- デメリット:
  - Runbook は infra 変更 PR のレビュー対象ではなく運用 PR のレビュー対象。CODEOWNERS を path-pattern で正確に切れない
  - Chaos / DR スクリプトは infra の宣言的リソースではなく実行可能な Go / Shell スクリプトであり、infra 配下の宣言型 YAML 群に混ぜると品質基準（リンタ・フォーマッタ設定）が噛み合わない
  - スパースチェックアウトで運用担当と infra 担当を分離する際の粒度が荒くなる

## 帰結

### ポジティブな帰結

- CNCF Platform Engineering Maturity Model が要求する「素構成 / 配信定義 / 運用手順」の分離を構造レベルで実現
- tier 横断基盤（Kafka / CloudNativePG / Keycloak 等）の所有権が tier1 から独立
- ArgoCD ApplicationSet の検索パス（`deploy/apps/`）とクラスタ素構成（`infra/`）の参照関係が明示される
- Argo CD Image Updater の write-back 先（`deploy/apps/` または `deploy/kustomize/overlays/*/`）が明確
- スパースチェックアウト cone で tier 開発者と infra-ops 担当者のワーキングセットが構造的に分離
- Phase 2 でマルチクラスタ化した際、`infra/environments/` に `prod-east/` `prod-west/` を追加する形で対応できる

### ネガティブな帰結

- DS-SW-COMP-120 の改訂が発生
- 旧 `src/tier1/infra/dapr/components/` の内容は `infra/dapr/components/` に物理移動する（本 ADR 起票時点では実装がないため、配置計画書のパス表現のみ書き換え）
- リポジトリルート直下のディレクトリ数が増え、README.md のガイド記述が充実化を要する
- ArgoCD ApplicationSet の template を書く際、`{{path}}` / `{{repoURL}}` の参照先を新構造に合わせる必要がある

### 移行・対応事項

- [../../04_概要設計/20_ソフトウェア方式設計/01_コンポーネント方式設計/06_パッケージ構成_Rust_Go.md](../../04_概要設計/20_ソフトウェア方式設計/01_コンポーネント方式設計/06_パッケージ構成_Rust_Go.md) の DS-SW-COMP-120 を改訂し、`src/tier1/infra/` を削除、代わりに「infra / deploy / ops はルート直下の別階層で管理（ADR-DIR-002 参照）」と明記
- 実装フェーズのディレクトリ設計書（`docs/05_実装/00_ディレクトリ設計/50_infraレイアウト/`, `60_operationレイアウト/`）で本 ADR を参照
- CODEOWNERS サンプルで `infra/` / `deploy/` / `ops/` の責任分界を定義
- ArgoCD ApplicationSet のマッチパターンを `deploy/apps/*/Application.yaml` 等に設定する雛形を実装ドキュメントに収録
- IMP-DIR-INFRA-\* / IMP-DIR-OPS-\* と DS-SW-COMP-120 との双方向トレースを [../../../docs/05_実装/00_ディレクトリ設計/90_トレーサビリティ/02_DS-SW-COMP_121-135_との対応.md](../../../docs/05_実装/00_ディレクトリ設計/90_トレーサビリティ/02_DS-SW-COMP_121-135_との対応.md) で管理

## 参考資料

- [ADR-DIR-001: contracts 昇格](ADR-DIR-001-contracts-elevation.md)
- [ADR-DIR-003: スパースチェックアウト cone mode 採用](ADR-DIR-003-sparse-checkout-cone-mode.md)
- [ADR-CICD-001: ArgoCD 採用](ADR-CICD-001-argocd.md)
- [ADR-CICD-002: Argo Rollouts 採用](ADR-CICD-002-argo-rollouts.md)
- [DS-SW-COMP-120](../../04_概要設計/20_ソフトウェア方式設計/01_コンポーネント方式設計/06_パッケージ構成_Rust_Go.md)
- CNCF Platform Engineering Maturity Model: [tag-app-delivery.cncf.io](https://tag-app-delivery.cncf.io/)
- GitOps Principles: [opengitops.dev](https://opengitops.dev/)
- Argo CD Application の Best Practices: [argo-cd.readthedocs.io](https://argo-cd.readthedocs.io/en/stable/operator-manual/declarative-setup/)
- Kustomize overlay 設計: Airbnb / Shopify / Netflix のパブリック事例
