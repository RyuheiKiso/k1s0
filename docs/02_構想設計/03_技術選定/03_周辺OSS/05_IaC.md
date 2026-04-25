# IaC (Infrastructure as Code) — OpenTofu

## 目的

k1s0 の infra 層の「さらに下」にある物理・仮想基盤 (VM / ネットワーク / ストレージ) を宣言的に管理する手段として、OpenTofu を採用する。本資料はその採用根拠・管理スコープ・段階導入計画を整理する。

## 採用決定の位置付け

- 採用対象: **OpenTofu** (`opentofu/opentofu`, MPL 2.0)
- 採用区分: k1s0 の正式採用技術
- 判断時期: 2026-04-12

---

## 1. 解決したい課題

k1s0 の企画書は k8s クラスタ上のコンポーネント配置を詳細に定義しているが、**k8s クラスタ自体の構築方法**は「セルフマネージド (kubeadm メイン / k3s サブプラン)」の一言で終わっている。

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
| 採用側組織の情シス部門が扱える | **適合**。HCL は宣言的で学習曲線が緩やか。Terraform の既存学習資産がそのまま使える |

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

### 4.1 候補一覧と定量比較

IaC ツール候補 10 種を「ライセンス OSI 適合 / オンプレ vSphere 対応 / 宣言性 / 学習コスト / エコシステム / コミュニティ規模」の 6 観点で同列評価する。OpenTofu 採用の核心は「Terraform エコシステムの完全互換」と「LF/CNCF 中立ガバナンス」の同時成立であり、両方を満たす候補は他に存在しない。

| 候補 | ライセンス | OSI 承認 | オンプレ vSphere | 宣言/命令 | 主言語/DSL | 学習コスト | Provider/モジュール数 | コミュニティ規模 | k1s0 適合 | 採否 |
|---|---|---|---|---|---|---|---|---|---|---|
| **OpenTofu** | MPL 2.0 | ◯ | ◎ vSphere Provider 利用可 | 宣言的 | HCL | 低 (Terraform 学習資産がそのまま使える) | Terraform Registry 互換、3,800+ Provider | GitHub 25k stars、CNCF Sandbox、LF 傘下、18 FTE | ◎ 全前提条件を満たす唯一の候補 | **採用** |
| Terraform | **BSL 1.1** | **✕** | ◎ | 宣言的 | HCL | 低 | Terraform Registry 公式、4,000+ Provider | GitHub 44k stars、HashiCorp/IBM 主導 | ✕ **OSI 非承認、選定方針違反** | 却下 |
| Pulumi | Apache 2.0 | ◯ | ◎ vSphere Provider あり | 命令的 (汎用言語) | TypeScript / Python / Go / C# / Java | 中〜高 (各言語の Pulumi SDK + 状態管理を学ぶ必要) | Pulumi Registry、~150 Provider (Terraform Bridge 経由で互換層あり) | GitHub 22k stars、Pulumi Corp 主導 | △ HCL より柔軟だが、採用側組織の情シスの宣言的志向と相性悪い | 次点 |
| CDK for Terraform (CDKTF) | Apache 2.0 | ◯ | ◎ Terraform Provider をそのまま使用 | 命令的 | TypeScript / Python / Go / Java / C# | 中 | Terraform Provider をブリッジ | HashiCorp 主導、3.5k stars | △ HashiCorp 主導 = ライセンス変更の連鎖リスク | 却下 |
| Crossplane | Apache 2.0 | ◯ | △ 一部 Provider あるが薄い | 宣言的 (k8s CRD) | YAML | 中 (k8s CRD と Composition の理解必須) | Crossplane Configuration Marketplace | CNCF Incubating、9k stars | ✕ **k8s 自体の構築 (k8s より下) には使えない** | 対象外 (補完用途) |
| Ansible | GPL 2.0/3.0 | ◯ | ○ vmware.vmware_rest コレクション | 命令的 (手続き的) | YAML + Jinja2 | 低〜中 | Ansible Galaxy、30,000+ コレクション | GitHub 63k stars、RedHat 主導 | △ 構成管理向け、IaC の冪等性は弱い | 次点 (補完用途で残す可能性) |
| Chef | Apache 2.0 | ◯ | ○ knife-vsphere | 命令的 | Ruby DSL | 中〜高 (Ruby + Chef Server 運用) | Chef Supermarket | Progress 主導、7.5k stars、活動鈍化 | ✕ Ruby スキル要員、採用側組織ではマイナー | 却下 |
| Puppet | Apache 2.0 | ◯ | ○ vsphere モジュール | 宣言的 | Puppet DSL | 中 | Puppet Forge、7,000+ モジュール | Puppet (Perforce) 主導、7k stars、活動鈍化 | ✕ 独自 DSL、エージェント常駐モデル、採用側組織での採用例少 | 却下 |
| Bicep | MIT | ◯ | **✕ Azure 限定** | 宣言的 | Bicep DSL | 低 | Azure Resource Provider のみ | Microsoft 主導 | ✕ オンプレ vSphere 不可 | 却下 |
| AWS CloudFormation | 商用 (AWS) | ✕ | **✕ AWS 限定** | 宣言的 | YAML/JSON | 中 | AWS リソースのみ | AWS 限定 | ✕ オンプレ vSphere 不可 | 却下 |
| vSphere PowerCLI | (商用 / Broadcom) | ✕ | ◎ ネイティブ | **命令的** (PowerShell スクリプト) | PowerShell | 中 | vSphere SDK のみ | Broadcom 主導 | ✕ 宣言性なし、状態管理なし、ベンダーロックイン | 却下 |
| 自前で Bash + vSphere API スクリプト | — | — | ○ | 命令的 | Bash | 高 (再現性 / 冪等性 / 状態管理を全部書く) | ゼロ | — | ✕ 見積 5〜8 人月、車輪の再発明 | 却下 |

### 4.2 観点別の落選理由

各候補が k1s0 の前提条件のどこで脱落したかを散文で展開する。

#### Terraform (HashiCorp / IBM)

機能・エコシステム・実績では業界トップだが、2023-08 に **MPL 2.0 → BSL 1.1** に変更され OSI 承認外となった。BSL は「商用 SaaS 化を制限する」条項を含み、k1s0 が将来 SaaS 提供する場合の法的解釈が複雑になる。k1s0 の選定方針「OSI 承認 OSS のみ採用」に正面から違反するため、技術的優位性以前に対象外。OpenTofu はこのライセンス変更を契機に発足したフォークであり、HCL / State / Provider が完全互換という点で**Terraform を選ばない以上は OpenTofu が必然の選択肢**となる。

#### Pulumi

最大の魅力は「汎用言語 (TS/Python/Go/C#) でインフラを記述できる」ことだが、これは IaC の核心である **宣言性** を犠牲にする。IaC の本質は「あるべき状態を宣言し、差分を機械が解決する」ことであり、ループ・条件分岐・関数呼び出しを多用する命令的記述は、採用側組織の情シスのコードレビュー文化 (差分の意図を読みやすくする) と相性が悪い。また Pulumi Corp 単独主導であり、ライセンス変更リスクは Terraform と同質。次点ではあるが、HCL 既存学習資産を活かせる OpenTofu に軍配。

#### CDK for Terraform (CDKTF)

Pulumi と同じく命令的アプローチ。さらに HashiCorp 主導であり、Terraform 本体のライセンス変更が連鎖するリスクを抱える。同じ命令的選択肢なら、独立した Pulumi のほうがライセンスリスクが低い。

#### Crossplane

「IaC を k8s CRD で宣言する」という思想は魅力的だが、**k1s0 が必要としているのは「k8s より下のレイヤー (VM / ネットワーク / k8s クラスタ自体)」のプロビジョニング**である。Crossplane は k8s クラスタが既に存在することを前提とするため、ニワトリと卵の問題で k8s 自体の構築には使えない。将来的に「k8s 上から外部 SaaS リソースを管理する」用途では補完的に検討する余地はあるが、IaC 主軸ツールとしては失格。

#### Ansible

YAML + Jinja2 でオンプレ運用に強く、採用側組織のでも採用実績が多い。ただし Ansible の本質は **構成管理 (Configuration Management)** であり、IaC (Infrastructure Provisioning) とは設計思想が異なる。具体的には:

- 状態管理 (State) を持たないため、「差分のみ適用」を冪等化するのに Playbook 側で工夫が必要
- リソースの依存関係解決が IaC ツールより弱い
- 「現在の構成」と「あるべき構成」の差分計算が苦手

ただし Ansible は「OS レベルの構成管理 / アプリ配信ポータル端末への設定配布」では IaC ツールが手薄なので、**OpenTofu と Ansible を併用** する余地がある (OpenTofu で VM プロビジョン、Ansible で OS 内設定)。次点として補完用途で残す。

#### Chef / Puppet

両者とも IaC 黎明期の代表格だが、近年は活動鈍化 (両者 GitHub stars ~7.5k で頭打ち)。Chef は Ruby、Puppet は独自 DSL で、採用側組織の情シスのスキルセットと整合しない。Puppet は agent 常駐モデルで運用負荷も高い。OpenTofu / Ansible が確立した現在、新規採用する積極的理由がない。

#### Bicep / CloudFormation

両者ともクラウドベンダー専用 (Azure / AWS) であり、**採用側組織のオンプレ vSphere の前提に根本的に非適合**。

#### vSphere PowerCLI

vSphere ネイティブで操作可能だが、PowerShell スクリプトであり**宣言性 / 状態管理 / 差分検知** といった IaC の核心機能を持たない。Broadcom 商用ライセンスでベンダーロックイン懸念もあり、却下。

#### 自前で Bash + vSphere API スクリプト

技術的には可能だが、状態管理・冪等性・依存解決・ロック・差分検知などを全部自作する必要があり、見積 **5〜8 人月**。OpenTofu (vSphere Provider) が無償で提供している価値の再発明であり、OSS リリース時点での採用範囲縮小方針に反する。

### 4.3 結論

「OSI 承認 OSS」「オンプレ vSphere 対応」「宣言的記述」「Terraform エコシステム互換 (Provider 3,800+)」「LF/CNCF 中立ガバナンス」の 5 条件を同時に満たす候補は OpenTofu のみ。Terraform は OSI 違反、Pulumi/CDKTF は命令的、Crossplane は k8s より上のみ、Bicep/CloudFormation はクラウド限定、Ansible は構成管理寄り、Chef/Puppet/PowerCLI は時代遅れまたはロックイン。**Valkey と同じく「ベンダーがライセンス変更したら LF 傘下フォークへ」という k1s0 の一貫した方針** ([3.2 節](#32-valkey--redis-と同じ構図)) に従う限り、OpenTofu 採用は必然である。

---

## 5. 管理スコープの境界

### OpenTofu が管理するもの (k8s より下)

| リソース | 内容 |
|---|---|
| VM インスタンス | vSphere / Proxmox / ベアメタル上の仮想マシン |
| ネットワーク | VLAN / サブネット / ファイアウォールルール |
| ストレージ | VM ディスク / NFS / iSCSI |
| DNS レコード | 内部 DNS (k1s0 関連) |
| k8s クラスタ bootstrap | cloud-init + kubespray 経由で kubeadm を起動 (サブプラン: k3s) |
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

## 6. 採用側組織のオンプレ環境での活用

採用側組織のオンプレ環境は VMware vSphere が主流。OpenTofu の vSphere Provider で以下を自動化する。

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

| 段階 | 範囲 | 成果物 |
|---|---|---|
| **リリース時点** | k8s ノード用 VM 作成 + k8s bootstrap の最小 HCL | VM 3 台 + k8s クラスタが `tofu apply` で立ち上がる |
| **採用後の運用拡大時** | ネットワーク / ストレージ / LB の HCL 化 + State バックエンド (MinIO) 導入 | 環境の完全な再現が可能 |
| **採用側のマルチクラスタ移行時** | dev / staging / prod のマルチ環境対応 (`tfvars` 分離) | 環境差異が `tfvars` に集約 |
| **採用側の全社ロールアウト** | マルチクラスタ対応 | 複数クラスタを宣言的に管理 |

### リリース時点 に含める理由

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
| vSphere Provider のメンテナンス | 採用側組織の主要環境で不具合 | vSphere Provider は Broadcom 公式メンテ。OpenTofu / Terraform 共通 |
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

- [`00_選定方針.md`](../01_俯瞰/00_選定方針.md) — OSI 承認ライセンスの前提条件
- [`04_選定一覧.md`](../01_俯瞰/04_選定一覧.md) — 採用 OSS 一覧
- [`../../01_アーキテクチャ/01_基礎/03_配置形態.md`](../../01_アーキテクチャ/01_基礎/03_配置形態.md) — k8s クラスタの配置 (OpenTofu が構築する対象)
- [`../../../01_企画/企画書.md`](../../../01_企画/企画書.md) — リリース時点 に OpenTofu を含める根拠
