# Terraform: 基本

- 対象読者: クラウドサービスの基本概念（VM、ネットワーク、ストレージ等）を理解している開発者・インフラエンジニア
- 学習目標: Terraform の仕組みを理解し、HCL でインフラを定義して作成・変更・削除できるようになる
- 所要時間: 約 40 分
- 対象バージョン: Terraform 1.14.x
- 最終更新日: 2026-04-14

## 1. このドキュメントで学べること

- Terraform が解決する課題と Infrastructure as Code（IaC）の意義を説明できる
- HCL（HashiCorp Configuration Language）の基本構文を読み書きできる
- terraform init / plan / apply / destroy の基本ワークフローを実行できる
- State（状態管理）の役割と重要性を理解できる

## 2. 前提知識

- いずれかのクラウドサービス（AWS / Azure / GCP 等）の基本操作経験
- コマンドラインの基本操作
- テキストエディタの操作

## 3. 概要

Terraform は HashiCorp 社が開発したオープンソースの Infrastructure as Code（IaC）ツールである。インフラの構成を HCL という宣言的な設定言語で記述し、コマンド一つでクラウドリソースの作成・変更・削除を自動化する。

従来のインフラ構築では、管理コンソールを手動操作してリソースを作成していた。この方法には「誰が・いつ・何を変更したか追跡できない」「同じ環境を再現できない」「手順ミスが発生する」という課題がある。Terraform はインフラの望ましい状態をコードで定義し、Git で管理することで、これらの課題を解決する。

Terraform の最大の特徴はマルチクラウド対応である。Provider（プロバイダ）というプラグイン機構により、AWS・Azure・GCP・Kubernetes など 1000 以上のサービスを同一のワークフローで管理できる。

## 4. 用語の整理

| 用語 | 説明 |
|------|------|
| IaC（Infrastructure as Code） | インフラの構成をコードとして管理する手法 |
| HCL（HashiCorp Configuration Language） | Terraform の設定ファイルに使われる宣言的言語 |
| Provider | クラウドサービスの API を抽象化するプラグイン（例: aws, azurerm, google） |
| Resource | Terraform が管理するインフラの構成要素（例: VM, ネットワーク, データベース） |
| State | Terraform が管理するリソースの現在の状態を記録したファイル（.tfstate） |
| Module | 再利用可能な Terraform 設定のパッケージ |
| Plan | 設定と現在の状態を比較し、実行予定の変更内容を表示する操作 |
| Backend | State ファイルの保存先（ローカルファイルまたはリモートストレージ） |

## 5. 仕組み・アーキテクチャ

Terraform はクライアントサイドのツールであり、サーバーは不要である。開発者が CLI でコマンドを実行すると、Terraform Core が HCL 設定ファイルを読み込み、Provider プラグインを介してクラウドの API を呼び出す。

![Terraform アーキテクチャ](./img/terraform_basics_architecture.svg)

コアワークフローは **Write → Plan → Apply** の 3 ステップで構成される。まず HCL で望ましい状態を記述し、Plan で差分を確認し、Apply で実インフラに適用する。

![Terraform ワークフロー](./img/terraform_basics_workflow.svg)

## 6. 環境構築

### 6.1 必要なもの

- Terraform CLI
- テキストエディタ（VS Code + HashiCorp Terraform 拡張を推奨）
- クラウドプロバイダのアカウントと認証情報

### 6.2 セットアップ手順

```bash
# macOS の場合: Homebrew でインストールする
brew tap hashicorp/tap
brew install hashicorp/tap/terraform

# Windows の場合: Chocolatey でインストールする
choco install terraform

# Linux の場合: 公式リポジトリからインストールする
sudo apt-get update && sudo apt-get install -y terraform
```

### 6.3 動作確認

```bash
# バージョンを確認する
terraform version
```

`Terraform v1.14.x` のように表示されればセットアップ完了である。

## 7. 基本の使い方

以下は AWS に EC2 インスタンスを 1 台作成する最小構成の例である。

```hcl
# AWS に EC2 インスタンスを作成する Terraform 設定ファイル

# Terraform の基本設定を定義する
terraform {
  # AWS Provider のバージョンを指定する
  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = "~> 5.0"
    }
  }
}

# AWS Provider の設定を定義する
provider "aws" {
  # 使用するリージョンを指定する
  region = "ap-northeast-1"
}

# EC2 インスタンスのリソースを定義する
resource "aws_instance" "web" {
  # Amazon Machine Image の ID を指定する
  ami           = "ami-0abcdef1234567890"
  # インスタンスタイプを指定する
  instance_type = "t3.micro"
  # タグを付与する
  tags = {
    Name = "my-first-instance"
  }
}
```

### 解説

- `terraform` ブロック: 使用する Provider とバージョンを宣言する
- `provider` ブロック: Provider の接続設定（リージョン、認証等）を指定する
- `resource` ブロック: 作成したいリソースの種類と設定を定義する。`"aws_instance"` がリソース型、`"web"` が Terraform 内での識別名である

```bash
# Provider プラグインをダウンロードする
terraform init

# 実行計画を確認する（実インフラには変更を加えない）
terraform plan

# 計画を実行し、リソースを作成する（確認プロンプトが表示される）
terraform apply

# 作成したリソースを全て削除する
terraform destroy
```

## 8. ステップアップ

### 8.1 変数による設定の外部化

値をハードコードせず、変数として外部化することで再利用性を高める。

```hcl
# 変数定義ファイル（variables.tf）

# インスタンスタイプの変数を定義する
variable "instance_type" {
  # 変数の説明を記載する
  description = "EC2 インスタンスのタイプ"
  # デフォルト値を設定する
  default     = "t3.micro"
  # 型を指定する
  type        = string
}
```

### 8.2 出力値の定義

作成したリソースの属性を出力する。

```hcl
# 出力定義ファイル（outputs.tf）

# インスタンスのパブリック IP を出力する
output "instance_ip" {
  # 出力する値を指定する
  value = aws_instance.web.public_ip
}
```

## 9. よくある落とし穴

- **State ファイルの直接編集**: .tfstate ファイルは Terraform が自動管理する。手動で編集するとリソースの追跡が壊れる。修正が必要な場合は `terraform state` コマンドを使用する
- **チームでのローカル State**: 複数人で作業する場合、ローカルの State ファイルでは競合が発生する。S3 + DynamoDB などのリモート Backend を使用する
- **plan なしの apply**: `terraform apply` を直接実行すると意図しない変更が適用される可能性がある。必ず `terraform plan` で差分を確認してから適用する
- **Provider バージョンの未固定**: バージョンを指定しないと、Provider の更新で予期しない変更が発生する。`required_providers` でバージョンを固定する

## 10. ベストプラクティス

- State ファイルはリモート Backend（S3、GCS、Azure Blob 等）に保存する
- 環境（dev / stg / prod）ごとにディレクトリまたはワークスペースを分離する
- `terraform fmt` でコードフォーマットを統一する
- `terraform validate` で構文エラーを事前に検出する
- 機密情報（パスワード、APIキー）は変数ファイルに書かず、環境変数や Vault を使用する
- Module を活用し、共通パターンを再利用可能にする

## 11. 演習問題

1. ローカルファイルに文字列を書き込む `local_file` リソースを使い、Terraform の init → plan → apply → destroy のワークフローを一通り実行せよ
2. `variable` と `output` を使い、ファイル名と内容を外部から指定できるように書き換えよ
3. `terraform plan` の出力を読み、追加（+）・変更（~）・削除（-）の表記を確認せよ

## 12. さらに学ぶには

- 公式ドキュメント: <https://developer.hashicorp.com/terraform>
- Get Started チュートリアル: <https://developer.hashicorp.com/terraform/tutorials>
- HCL 言語仕様: <https://developer.hashicorp.com/terraform/language>
- Terraform Registry（Provider / Module 検索）: <https://registry.terraform.io/>

## 13. 参考資料

- What is Terraform?: <https://developer.hashicorp.com/terraform/intro>
- Terraform CLI Documentation: <https://developer.hashicorp.com/terraform/cli>
- Terraform Language Documentation: <https://developer.hashicorp.com/terraform/language>
