# ADR-0003: Kustomize + Helm の使い分け方針

- ステータス: Accepted
- 起票日: 2026-04-14
- 決定日: 2026-04-14
- 起票者: kiso ryuhei
- 関係者: インフラチーム / 起案者 / 決裁者

## コンテキスト

Phase 1a (MVP-0) で kubeadm + Dapr を立ち上げ、Phase 1b (MVP-1a) で Argo CD による GitOps に移行する。
それに先立ち、Kubernetes マニフェストの **生成・管理・テンプレート戦略** を確定する必要がある。

k1s0 が Kubernetes 上に展開する対象は性質の異なる 2 系統に分かれる。

1. **自製マニフェスト**: tier1 / tier2 / tier3 の自前サービス。リポジトリ内で管理し、環境ごと (dev / staging / prod) に値を差し替える。
2. **サードパーティ OSS**: Strimzi (Kafka) / Keycloak / Istio / Argo CD / Prometheus 等。多くが上流で **Helm Chart** として配布されており、それ以外の形式での導入は実質的に難しい。

「全部 Helm」「全部 Kustomize」のいずれの単純戦略も、それぞれ無理が生じる。

## 決定

**自製マニフェスト = Kustomize、サードパーティ OSS = Helm Chart** の使い分けを採用する。

| 対象 | ツール | 管理方式 |
|---|---|---|
| 自製サービス (tier1 / tier2 / tier3) | **Kustomize** | `base/` + `overlays/<env>/` 構成。環境差分は patch で表現 |
| サードパーティ OSS (Strimzi / Keycloak / Istio 等) | **Helm Chart** | 上流 Chart を `values.yaml` でカスタマイズしてインストール |

Argo CD はいずれの形式もネイティブにサポートしているため、GitOps 導入後も二刀流の構成を維持する。

## 検討した選択肢

### 選択肢 A: Helm 単独

- 概要: 自製マニフェストも独自 Chart 化して Helm で統一する。
- メリット: ツールが 1 種類で学習コストが小さい。
- デメリット: Go テンプレートが入り混じった YAML は可読性が低く、レビュー・差分確認が困難。値の優先順位 (`values.yaml` / `--set` / 環境別 values) が複雑化する。

### 選択肢 B: Kustomize 単独

- 概要: サードパーティ OSS も生マニフェストに展開して Kustomize で管理する。
- メリット: ツールが 1 種類で kubectl 組み込み。
- デメリット: サードパーティ OSS は Helm 配布が事実上の標準。`helm template` で吸い出して手動編集する運用は、上流バージョン追従が極端に重くなり現実的でない。

### 選択肢 C: Jsonnet / Tanka / Timoni / cdk8s 等

- 概要: より高機能なテンプレート言語/SDK を採用する。
- メリット: プログラマブルで強力。
- デメリット: 学習コストが高く、JTC 内での採用例も限定的。バス係数 2 を実証する MVP-1a のスコープに見合わない。

### 選択肢 D: Kustomize + Helm の使い分け (採用)

- 概要: 自製は Kustomize、サードパーティは Helm。
- メリット: それぞれの強みを活かせる。Argo CD でも自然にサポートされる。
- デメリット: 開発者が 2 ツールを習得する必要がある。

## 決定理由

- **kubectl 組み込みで追加導入不要**: Kustomize は `kubectl` に組み込まれており、ローカル開発で追加インストールが不要。
- **YAML patch で可読性を維持**: 自製マニフェストは Go テンプレートを混入させず、純粋な YAML として管理できる。差分レビューが容易。
- **Argo CD ネイティブサポート**: 両形式とも Argo CD の Application で直接扱える。GitOps への移行で構成を変更する必要がない。
- **サードパーティの上流追従を犠牲にしない**: Helm Chart のまま管理することで、上流のバージョンアップを `values.yaml` 互換性確認のみで取り込める。Renovate での自動 PR 生成と相性が良い。
- **学習コストは限定的**: Kustomize は patch のみ、Helm は `values.yaml` 設定のみという狭いスコープでの利用に閉じれば、学習負荷は許容範囲内。

## 影響

### ポジティブな影響

- 自製マニフェストの可読性が高く、レビュー・差分確認が容易になる。
- サードパーティ OSS の上流追従が `values.yaml` の差分確認だけで完結し、Renovate 自動更新と相性が良い。
- Argo CD への移行 (Phase 1b) で構成変更を必要としない。

### ネガティブな影響 / リスク

- 開発者は Kustomize と Helm の双方を習得する必要がある。
  - 緩和策: 利用範囲を限定 (Kustomize は patch のみ、Helm は values 設定のみ) し、雛形 CLI で標準ディレクトリ構成を生成する。
- Kustomize の `strategicMergePatch` は意図しないマージ挙動をする場合がある。
  - 緩和策: 既定で **JSON 6902 patch** を使用する方針とする。
- Helm Chart の上流が破壊的変更を入れた場合、影響範囲の特定に時間がかかる。
  - 緩和策: Renovate での PR 単位で取り込み、CI でデプロイテストを自動実行する。

### 移行・対応事項

- 雛形生成 CLI に **`deploy/base/` + `deploy/overlays/{dev,staging,prod}/`** の Kustomize 標準構成を生成させる。
- サードパーティ OSS 導入は `infra/helm/<chart-name>/values.yaml` で値を管理する標準ディレクトリを定義する。
- Argo CD 導入時 (Phase 1b) に、両形式を扱う Application 定義のテンプレートを用意する。
- Kustomize の patch 方針 (JSON 6902 を既定とする) を開発ガイドラインに記載する。

## 参考資料

- [`../../01_企画/04_技術選定/12_追加採用OSS_4.md`](../../01_企画/04_技術選定/12_追加採用OSS_4.md) — Kustomize + Helm 採用の根拠
- [`../../01_企画/07_ロードマップと体制/00_フェーズ計画.md`](../../01_企画/07_ロードマップと体制/00_フェーズ計画.md) — Phase 1a 〜 1b でのデプロイ手段の段階導入
- [ADR-0004](./ADR-0004-kubeadm-adoption.md) — kubeadm 採用
