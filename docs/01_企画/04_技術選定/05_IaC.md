# IaC (Infrastructure as Code) — OpenTofu

## 目的

k1s0 の infra 層の「さらに下」にある物理・仮想基盤 (VM / ネットワーク / ストレージ) を宣言的に管理する手段として、OpenTofu を採用する。本資料はその採用根拠・管理スコープ・段階導入計画を整理する。

## 採用決定の位置付け

- 採用対象: **OpenTofu** (`opentofu/opentofu`, MPL 2.0)
- 採用区分: k1s0 の正式採用技術
- 判断時期: 2026-04-12

---

## 1. 解決したい課題

k1s0 の企画書は k8s クラスタ上のコンポーネント配置を詳細に定義しているが、**k8s クラスタ自体の構築方法**は「セルフマネージド (kubeadm / k3s / RKE2)」の一言で終わっている。

| 課題 | 影響 |
|---|---|
| VM の作成手順が属人的 | vSphere GUI で手作業。構築した人が退職すると再構築不能 |
| 環境の再現性がない | dev / staging / prod を同一構成で複製できない |
| 構成ドリフトを検知できない | 手で変更した差分が追跡不能 |
| DR (災害復旧) に手順書が必要 | 復旧に数日かかり、差異が混入する |
| バス係数 = 1 のリスク増大 | infra 構築の暗黙知が起案者 1 人に集中 |

これらは k8s **上**の GitOps (Argo CD) では解決できない。k8s **下**のレイヤーに IaC が必要。

---

## 2. OpenTofu とは

| 項目 | 内容 |
|---|---|
| 種別 | Infrastructure as Code (IaC) ツール |
| 起源 | Terraform (HashiCorp) の fork。2023年8月の Terraform BSL 化を受けて発足 |
| ガバナンス | **Linux Foundation + CNCF Sandbox** (2025年4月加入) |
| ライセンス | **MPL 2.0** (OSI承認) |
| 現行バージョン | v1.10+ |
| リリースサイクル | 四半期ごと |
| GitHub Stars | 25,000+ |
| コントリビューター | 70+ (アクティブ)、160+ (累計) |
| 支援企業 | Scalr / Spacelift / Harness / Gruntwork 等 (合計 18 FTE を拠出) |
| エンタープライズ実績 | Fidelity が 50,000+ State ファイルを移行済み |
| Terraform 互換性 | HCL 構文 / State ファイル / 主要 Provider が完全互換 |

---

## 3. 採用理由

### 3.1 k1s0 選定方針への適合

| 選定前提条件 | 評価 |
|---|---|
| オンプレ / VM で動作する | **適合**。CLI ツールのため環境を問わない |
| ベンダーロックインを回避できる | **適合**。LF / CNCF 傘下。特定企業に支配されない |
| OSI 承認 OSS ライセンスである | **適合**。MPL 2.0 |
| コミュニティが活発で長期運用に耐える | **適合**。CNCF Sandbox、25k+ stars |
| JTC 情シス部門が扱える | **適合**。HCL は宣言的で学習曲線が緩やか。Terraform の既存学習資産がそのまま使える |

### 3.2 Valkey / Redis と同じ構図

| 構図 | Redis → Valkey | Terraform → OpenTofu |
|---|---|---|
| ベンダー | Redis Ltd. | HashiCorp (IBM) |
| ライセンス変更 | OSS → RSALv2/SSPL | MPL 2.0 → BSL 1.1 |
| fork 先 | Valkey (LF) | OpenTofu (LF / CNCF) |
| 互換性 | wire protocol 完全互換 | HCL / Provider / State 完全互換 |
| k1s0 の判断 | **Valkey を採用** | **OpenTofu を採用** |

k1s0 は一貫して「ベンダーがライセンスを変更した場合は LF 傘下の fork に移行する」方針を取る。

### 3.3 Terraform にない独自優位性

- **State のデフォルト暗号化**: セキュリティ監査 (ISMS / J-SOX) で加点
- **terraform ブロック内での変数・locals の動的参照**: マルチ環境構成が簡潔に書ける

---

## 4. 候補比較

| 候補 | 採否 | ライセンス | 評価 |
|---|---|---|---|
| **OpenTofu** | 採用 | MPL 2.0 | HCL 互換、CNCF Sandbox、エンタープライズ実績あり |
| Terraform | 却下 | BSL 1.1 | **OSI 非承認。k1s0 選定方針に違反** |
| Pulumi | 次点 | Apache 2.0 | コード (Go/TS/Python) で定義。宣言的ではなく命令的。HCL より学習コスト高 |
| Crossplane | 対象外 | Apache 2.0 | k8s CRD ベース。k8s 自体の構築には使えない |
| Ansible | 次点 | GPL 2.0 | 構成管理は得意だが、インフラプロビジョニングの宣言性と冪等性が IaC 専用ツールより弱い |

---

## 5. 管理スコープの境界

### OpenTofu が管理するもの (k8s より下)

| リソース | 内容 |
|---|---|
| VM インスタンス | vSphere / Proxmox / ベアメタル上の仮想マシン |
| ネットワーク | VLAN / サブネット / ファイアウォールルール |
| ストレージ | VM ディスク / NFS / iSCSI |
| DNS レコード | 内部 DNS (k1s0 関連) |
| k8s クラスタ bootstrap | cloud-init 経由で kubeadm / k3s / RKE2 を起動 |
| ロードバランサ | k8s API サーバーの前段 LB |

### Argo CD が管理するもの (k8s より上)

| リソース | 内容 |
|---|---|
| Helm release / k8s manifest | Istio / Kafka / Dapr / Keycloak / Backstage 等の全 OSS |
| tier1 / tier2 / tier3 サービス | Deployment / Service / ConfigMap 等 |
| Dapr Component | tier1 チームが管理する Component YAML |

### 境界の原則

> **OpenTofu は「箱 (VM / ネットワーク)」を作る。Argo CD は「箱の中身 (k8s リソース)」を管理する。両者は重複しない。**

---

## 6. JTC オンプレ環境での活用

JTC のオンプレ環境は VMware vSphere が主流。OpenTofu の vSphere Provider で以下を自動化する。

| 操作 | 従来の手作業 | OpenTofu |
|---|---|---|
| k8s ノード用 VM 作成 | vSphere Client で GUI 操作 | `tofu apply` |
| CPU / メモリ変更 | 変更申請書 → GUI 操作 | HCL 修正 → PR → `tofu apply` |
| ノード追加 | 手順書を再実行 | `count` を変更して apply |
| 環境複製 (dev → staging) | 全手順を再実行 | `tfvars` の差し替えで apply |

---

## 7. State 管理方針

| 項目 | 方針 |
|---|---|
| State バックエンド | S3 互換ストレージ (MinIO) または PostgreSQL |
| State 暗号化 | OpenTofu のデフォルト暗号化を有効化 |
| State ロック | バックエンドのロック機能を利用 |
| State の Git 管理 | **禁止** (機密情報を含むため)。HCL のみ Git 管理 |
| バックアップ | State バックエンドの定期バックアップを運用手順に含める |

---

## 8. 段階導入計画

| フェーズ | 範囲 | 成果物 |
|---|---|---|
| **Phase 1 (MVP)** | k8s ノード用 VM 作成 + k8s bootstrap の最小 HCL | VM 3 台 + k8s クラスタが `tofu apply` で立ち上がる |
| **Phase 2** | ネットワーク / ストレージ / LB の HCL 化 + State バックエンド (MinIO) 導入 | 環境の完全な再現が可能 |
| **Phase 3** | dev / staging / prod のマルチ環境対応 (`tfvars` 分離) | 環境差異が `tfvars` に集約 |
| **Phase 5** | マルチクラスタ対応 | 複数クラスタを宣言的に管理 |

### MVP に含める理由

- 「試行運用環境の確保」が再現可能になる
- 手作業で構築した環境は壊したら再構築に数日かかるが、HCL があれば数時間で復旧
- バス係数 = 1 の緩和: HCL を読めば起案者以外でも環境を再構築できる

---

## 9. リスクと対処

| リスク | 影響 | 対処 |
|---|---|---|
| HashiCorp の cease-and-desist (2024年4月) | 法的リスク | OpenTofu 側は否認。実害未発生。企業法務に確認を推奨 |
| Terraform との長期的な機能乖離 | Provider 互換性の低下 | 主要 Provider (vSphere / AWS / Azure) は両方をサポート。乖離は限定的 |
| CNCF Sandbox (Graduated ではない) | プロジェクト中止リスク | LF 傘下 + 支援企業 18 FTE。Sandbox → Graduated の昇格は時間の問題 |
| vSphere Provider のメンテナンス | JTC の主要環境で不具合 | vSphere Provider は Broadcom 公式メンテ。OpenTofu / Terraform 共通 |
| State にインフラの機密情報が入る | 漏洩リスク | デフォルト暗号化 + Git 管理禁止 + バックエンド ACL |

---

## 10. ディレクトリ構成 (想定)

| パス | 内容 |
|---|---|
| `infra/tofu/modules/k8s-node/` | VM 作成 + cloud-init モジュール |
| `infra/tofu/modules/network/` | VLAN / サブネット / FW モジュール |
| `infra/tofu/modules/storage/` | NFS / iSCSI モジュール |
| `infra/tofu/modules/loadbalancer/` | k8s API LB モジュール |
| `infra/tofu/envs/dev/` | 開発環境の `main.tf` / `variables.tf` / `terraform.tfvars` |
| `infra/tofu/envs/staging/` | ステージング環境 |
| `infra/tofu/envs/prod/` | 本番環境 |
| `infra/tofu/backend.tf` | State バックエンド設定 |

---

## 関連ドキュメント

- [`00_選定方針.md`](./00_選定方針.md) — OSI 承認ライセンスの前提条件
- [`04_選定一覧.md`](./04_選定一覧.md) — 採用 OSS 一覧
- [`../02_アーキテクチャ/03_配置形態.md`](../02_アーキテクチャ/03_配置形態.md) — k8s クラスタの配置 (OpenTofu が構築する対象)
- [`../07_ロードマップと体制/01_MVPスコープ.md`](../07_ロードマップと体制/01_MVPスコープ.md) — MVP に OpenTofu を含める根拠
