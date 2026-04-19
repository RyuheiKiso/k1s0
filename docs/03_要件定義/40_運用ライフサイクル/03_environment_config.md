# OPS-ENV: 環境構成 / 設定管理要件

本ファイルは、k1s0 プラットフォームの**環境種別（dev / stg / prod）**、**設定値の流通方式（GitOps + OpenBao）**、**環境間プロモーション手順** を要件化する。CI/CD のパイプラインは [`01_CICD.md`](./01_CICD.md)、リリース戦略は [`02_release.md`](./02_release.md) を参照。

環境と設定の議論が崩れると、dev で動いたコードが本番で Secret 不足や構成差異で動かない・本番の設定が Git 外で書き換えられて監査証跡が残らない・シークレットが誤って Git に commit される、という事故につながる。本ファイルでは Phase 1a から守るべき最小ルールを確定する。

---

## 前提

- [`01_CICD.md`](./01_CICD.md) — Argo CD GitOps デプロイ
- [`../30_セキュリティ_データ/02_kms_crypto.md`](../30_セキュリティ_データ/02_kms_crypto.md) — OpenBao の暗号鍵管理
- [`../../02_構想設計/03_技術選定/03_周辺OSS/08_シークレット管理.md`](../../02_構想設計/03_技術選定/03_周辺OSS/08_シークレット管理.md) — OpenBao 採用根拠

---

## 要件本体

### OPS-ENV-001: 環境種別 3 種（dev/stg/prod）の責任分界

- 優先度: MUST（prod と stg の混同は本番事故の 8 割の遠因。環境境界を最初に固定しないと全ての設定管理が崩れる）
- Phase: Phase 1a（tier1 開発開始時）
- 関連: OPS-CID-003 / OPS-ENV-004

現状、dev と prod の境界が曖昧で、起案者が dev クラスタの MinIO に本番相当のダミーデータを置いたまま運用している。tier2/3 が参加する前に境界を固定しないと、「本番データが dev に流出」「dev の設定を本番にコピペ」という事故が必然になる。

要件達成後の世界では、k1s0 は **dev / stg / prod** の 3 環境を以下の責任で分離する。(1) **dev** は開発者の自由度最大、PII ダミーデータ、クラスタ名 `k1s0-dev`、OIDC realm は `k1s0-dev`、(2) **stg** は prod と**構成ドリフト 0 を監査**する準本番、匿名化された本番コピーデータ、クラスタ名 `k1s0-stg`、(3) **prod** は本番、PII 本物、変更は GitOps 2 段承認必須、クラスタ名 `k1s0-prod`。ネットワーク的にも Namespace と Istio AuthorizationPolicy で通信を遮断し、dev から prod Secret Manager への到達は ESO / Kyverno で Admission ブロック。

崩れた時、dev のバグが prod データを破壊する、あるいは prod 向けの稟議資料の数字が実は dev 環境の数字だった、という事故が発生する。SOX 監査で「開発 / 本番の分離」は必須論点のため、環境分離未達では稟議が通らない。

**受け入れ基準**

- 3 クラスタ（dev/stg/prod）が独立した kubeadm クラスタとして稼働、Namespace 共有なし
- 各環境の Keycloak Realm は独立し、dev ユーザーの prod ログインは Realm 分離で不可能
- Kyverno ポリシーで cross-env の Secret / ConfigMap 参照を Admission ブロック
- `.env.prod` を dev で開く操作は GitHub Actions の `environment: prod` ガードで Review 必須
- 環境識別ラベル `k1s0.io/env: dev|stg|prod` が全 Pod に必須（欠落時は Admission 拒否）

**検証方法**

- 月次で 3 環境の Istio AuthorizationPolicy を監査、cross-env 許可ルールの混入を検知
- 四半期ごとに dev → prod への到達テスト（意図的な接続試行）を行い、全てブロックされることを確認

---

### OPS-ENV-002: GitOps による宣言的設定（全環境）

- 優先度: MUST（手動変更を許すと環境ドリフトが発生、「stg で動いたが prod で動かない」原因の 7 割がこれ）
- Phase: Phase 1a（tier1）/ Phase 1b で全 tier
- 関連: OPS-CID-003 / OPS-ENV-004

現状、ConfigMap の一部が手動 edit で更新されており、Git リポにない設定が稼働している。次回の Argo CD 同期で上書きされるか、selfHeal 無効で放置されるかが Application ごとに違う。

要件達成後の世界では、全環境の全設定（Deployment / Service / ConfigMap / ApplicationSet / NetworkPolicy）が GitOps リポ `k1s0/gitops` に Kustomize base + env overlay で宣言管理される。`base/` は環境非依存、`overlays/{dev,stg,prod}/` は環境固有パラメータ（replica 数、resource requests、外部エンドポイント）のみ。prod の設定変更は必ず `overlays/prod/` への PR として可視化され、Argo CD の `selfHeal: true` が Git と本番の差分を検知次第自動上書きする。

崩れた時、本番に Git 外の「隠し設定」が蓄積し、DR 訓練で復旧クラスタが本番と同じ構成で立ち上がらない。これは QUA-DR（復旧時間目標 4h）の未達を意味する。

**受け入れ基準**

- 全 Argo CD Application の `syncPolicy.automated.selfHeal: true`（prod も含む）
- Kustomize 構成は base + 3 overlay、それ以外のパッチ（strategic merge の直書き）は CI で検知 fail
- Git と本番の差分（`argocd app diff`）が過去 30 日で 0 件
- 新規環境パラメータ追加時は base のデフォルトと 3 overlay の値が全て揃っていることを CI で検査
- Terraform / Pulumi などの IaC も同一リポに同居、手動 `kubectl apply` は監査で検知

**検証方法**

- Argo CD ダッシュボードで drift 件数を日次監視
- 四半期ごとに prod の全リソースを `argocd app diff` で照合、差分 0 を確認

---

### OPS-ENV-003: Secret は OpenBao 経由、Git に平文禁止

- 優先度: MUST（Git 平文 Secret は 1 度の漏洩で全テナント影響、法務クライシス。Phase 1a で固定必須）
- Phase: Phase 1a
- 関連: SEC-KMS-001 / SEC-IAM-001

現状、起案者の開発環境では一部 Secret が `values.yaml` に平文で書かれており、GitHub に push された経歴がある。git 履歴から削除しても漏洩リスクは消えない。

要件達成後の世界では、全ての Secret（DB 接続文字列、API トークン、署名鍵、PII 暗号鍵）は OpenBao（Vault 互換）に格納され、Pod は External Secrets Operator（ESO）経由で Secret を k8s Secret リソースにマウントする。Git に入るのは `ExternalSecret` カスタムリソース（参照名のみ）と Sealed Secrets（GitOps 内暗号化用、補助的用途）のみ。`git-secrets` + `gitleaks` を pre-commit と GHA に導入し、秘匿情報らしき文字列（AKIA.. / BEGIN RSA PRIVATE KEY.. 等）の commit をブロックする。

崩れた時、GitHub から平文 Secret が漏洩すると、全テナントの再発行 + 監査 + 法務対応で約 200〜500 万円 + 数週間の工数。上場企業としての情報開示義務が発生する。

**受け入れ基準**

- 全 k8s Secret は ESO 経由（ExternalSecret から派生）、直接 `kubectl create secret` は Kyverno audit で検知
- OpenBao は HA 3 レプリカ、unseal 鍵は Shamir 5 of 3、オフライン保管 2 か所（金庫 + 別オフィス）
- `gitleaks` が全 PR で実行、検出時は merge ブロック
- Secret ローテーションは OpenBao の `leaseDuration` で 90 日上限、自動再発行
- OpenBao の audit log は 7 年保管、アクセス全件記録

**検証方法**

- 四半期ごとに `gitleaks --all-history` で全リポ全履歴を走査
- OpenBao audit log を週次で SIEM に転送、異常アクセスパターンを検知

---

### OPS-ENV-004: 環境間プロモーション手順（dev → stg → prod）

- 優先度: MUST（プロモーション手順が無いと「dev で OK だから prod に直接」が常態化し、stg が形骸化する）
- Phase: Phase 1c（stg/prod 稼働時）
- 関連: OPS-REL-001 / OPS-CID-004

現状、環境間のプロモーションは属人的で、dev でテストした起案者が気分次第で stg を飛ばして prod に当てることが技術的に可能。stg の存在意義が薄い。

要件達成後の世界では、プロモーションは**必ず dev → stg → prod の順**で進み、各段階の昇格条件が GitOps リポの `promotion-policy.yaml` で宣言される。dev → stg 昇格は GHA の main merge で自動、**stg → prod 昇格は「stg で 24 時間エラーなし稼働」＋「OPS-CID-004 の 2 段承認」** が条件。昇格の自動化は Argo Events + Argo Rollouts Promotion Step で実装し、tag 書き換え PR が自動生成される。

崩れた時、stg を飛ばした prod デプロイで「stg では再現しなかった本番依存の問題」に気付かず、カナリア段階で初めて発覚してロールバック。この遅延は OPS-REL-002 の MTTR 悪化に直結する。

**受け入れ基準**

- プロモーション CR `promotion-policy.yaml` が全 tier1 / tier2 / tier3 Application で定義
- stg → prod 昇格に「stg 24h エラー 0 件」条件を必須化、Prometheus からの自動取得
- 手動で stg を飛ばした prod 変更 PR は Kyverno Admission で拒否
- プロモーション履歴は 365 日保管（Argo Events log + GitHub Actions Audit）
- 月次のプロモーション失敗率を集計、5% 超は原因分析

**検証方法**

- 四半期ごとに「stg を飛ばす」抜き打ちテストを実施し、ブロックされることを確認
- Argo CD History API で過去 90 日の昇格フロー完全性を監査

---

### OPS-ENV-005: 環境別設定の差分最小化

- 優先度: SHOULD（overlay 差分が肥大化すると stg の検証価値が落ちる）
- Phase: Phase 2
- 関連: OPS-ENV-002 / DEV-DX-003

現状、3 環境の overlay が肥大化しており、prod 固有の差分が 500 行を超える Application もある。stg で動作検証した内容が prod で再現しない確率が高まる。

要件達成後の世界では、overlay に含める差分は**環境固有の接続先と容量パラメータに限定**し、それ以外は base に集約する。Backstage Scorecard で各 Application の overlay 行数を可視化し、100 行を超えた Application は四半期レビューでリファクタリング対象となる。

崩れた時、stg で通過した設定が prod 固有の差分により失敗する確率が上がり、カナリアの自動ロールバック発火頻度が増加する。

**受け入れ基準**

- 全 Application の overlay 行数が 100 行以内（Backstage Scorecard で可視化）
- overlay に含めてよい項目は Runbook `overlay-allowed-fields.md` で明文化
- 100 行超の Application は四半期以内にリファクタリング PR を提出
- 新規 Application は雛形生成 CLI 経由で 30 行以内の overlay でスタート
- 環境間の差分ログ（`kustomize diff`）が weekly CI で自動更新

**検証方法**

- Backstage Scorecard `overlay-size` を月次レビュー
- 新規 Application の overlay 行数初期値を四半期レポート

---

## 章末サマリ

### ID 一覧

| ID | タイトル | 優先度 | Phase |
|---|---|---|---|
| OPS-ENV-001 | 環境種別 3 種の責任分界 | MUST | 1a |
| OPS-ENV-002 | GitOps 宣言管理 | MUST | 1a/1b |
| OPS-ENV-003 | OpenBao 経由の Secret | MUST | 1a |
| OPS-ENV-004 | プロモーション手順 | MUST | 1c |
| OPS-ENV-005 | overlay 差分最小化 | SHOULD | 2 |

### 優先度分布

| 優先度 | 件数 | 代表 ID |
|---|---|---|
| MUST | 4 | OPS-ENV-001, 002, 003, 004 |
| SHOULD | 1 | OPS-ENV-005 |

### Phase 達成度

| Phase | 必達件数 | 未達影響 |
|---|---|---|
| 1a | 3 | dev 着手時の環境 / Secret 分離が未成立、Phase 1a 納品不可 |
| 1c | 4 | stg/prod プロモーションが未確立、本番稼働できない |
| 2 | 5 | 稼働業務数拡大時の overlay 肥大化で stg 価値低下 |
