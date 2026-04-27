# infra/environments — 環境別 Kustomize overlay

`infra/` 配下の prod 基準 base 構成に対し、dev / staging / prod の 3 環境差分を Kustomize overlay として適用する。

## 設計正典

- [`docs/05_実装/00_ディレクトリ設計/50_infraレイアウト/08_環境別パッチ配置.md`](../../docs/05_実装/00_ディレクトリ設計/50_infraレイアウト/08_環境別パッチ配置.md) — 配置仕様（IMP-DIR-INFRA-078）
- [`docs/04_概要設計/55_運用ライフサイクル方式設計/03_環境構成管理方式.md`](../../docs/04_概要設計/55_運用ライフサイクル方式設計/03_環境構成管理方式.md) — DS-OPS-ENV-007〜011（3 環境構成と本番同等性）
- [`docs/02_構想設計/adr/ADR-CICD-001-argocd.md`](../../docs/02_構想設計/adr/ADR-CICD-001-argocd.md) — GitOps 採用根拠

## 設計原則

- **prod を base、dev / staging が overlay**: `infra/` 直下の `*-cluster.yaml` / `*-policies.yaml` 等は prod 基準で記述し、リソース規模を縮小したい dev / staging のみが Kustomize patch で上書きする。
- **同じにすべきもの**: OSS バージョン / Helm Chart バージョン / Operator バージョン / Network Policy / RBAC / Istio Ambient / Dapr Component / 監視設定 / ログ設定（DS-OPS-ENV-007）
- **違っていて良いもの**: replica 数 / リソース制限値 / テナント数 / endpoint URL / 外部連携先（DS-OPS-ENV-007）
- **ドリフト検出**: Argo CD DriftDetection で staging と prod の差分が想定外に拡大していないかを継続監視

## ディレクトリ構成

```text
infra/environments/
├── README.md                        # 本ファイル
├── dev/
│   ├── kustomization.yaml           # base 参照 + 環境 patch
│   ├── patches/                     # CRD instance / namespace への env 別パッチ
│   ├── values/                      # Helm chart の values 上書き（Helmfile / Argo CD で参照）
│   ├── secrets/                     # SOPS 暗号化済み secret（リリース時点 は .gitkeep のみ）
│   └── README.md
├── staging/                         # 同上、prod 同等構成だがスケール 1/3
│   └── …
└── prod/                            # base がそのまま prod を表現、patches/ は空
    └── …
```

## 各環境の特徴

| 観点 | dev | staging | prod |
|---|---|---|---|
| 規模 | 単一ノード kind / k3d で起動可能（CNPG 1 / Kafka 1 / Valkey standalone） | prod 同構成で replica を 1/3 に縮小 | フル HA（CNPG 3 / Kafka 3 / Valkey replication） |
| TLS | Let's Encrypt staging（rate limit 緩和） | Let's Encrypt staging | Let's Encrypt prod |
| 外部連携 | ダミー IdP / モック S3 | ステージング版の外部サービス | 本番外部サービス |
| 保持期間 | ログ 7 日 / トレース 3 日 | prod と同（30 日 / 7 日） | ログ 30 日 / トレース 7 日 / メトリクス 13 ヶ月 |
| データ | 合成データのみ | 擬似プロダクション（実顧客データなし） | 本番テナント |
| デプロイ | `kubectl apply -k infra/environments/dev/` | Argo CD ApplicationSet（手動承認 staging→prod） | Argo CD ApplicationSet（手動承認） |

## 利用方法

### dev（ローカル kind）

`tools/local-stack/up.sh` が以下を実行する想定:

```bash
kind create cluster --config tools/local-stack/kind-cluster.yaml
kubectl apply -k infra/environments/dev/
```

### staging / prod（GitOps）

[`deploy/apps/application-sets/`](../../deploy/apps/application-sets/) の Argo CD ApplicationSet が `infra/environments/<env>/` を target として宣言的に同期する。staging で 24 時間連続稼働検証を通過しない変更は prod に届かない構造ガードとする（DS-OPS-ENV-007）。

## SOPS 暗号化と secrets/

各環境の `secrets/` に置かれる Secret は SOPS + AGE で暗号化される。AGE 鍵は `ops/oncall/sops-key/` 配下にある（リポジトリ外、運用チームのみアクセス可）。GitOps（Argo CD）は SOPS plugin で復号してから apply する。リリース時点 では `secrets/.gitkeep` のみを配置する。

## dapr-components-overlay

`infra/dapr/components/` が配置され次第、`infra/environments/<env>/dapr-components-overlay/kustomization.yaml` を追加して env 別の Component patch を適用する（state-store 接続先 / pubsub Kafka broker URI 等）。リリース時点 では同ディレクトリのみ用意し、kustomization.yaml は base 配置時に追記する。
