# Terraform 運用手順書

Terraform の日常運用と障害時の手順を定義する。

関連設計: [terraform設計.md](./terraform設計.md)

---

## 前提: 環境別アクセス権限

| 環境 | アクセス権 | トークン取得元 |
|-----|-----------|--------------|
| dev | 開発チーム全員（read/write） | GitHub Secrets: `CONSUL_HTTP_TOKEN_DEV` |
| staging | インフラチーム + リード（read/write） | GitHub Secrets: `CONSUL_HTTP_TOKEN_STAGING` |
| prod | インフラチームのみ（read/write） | GitHub Secrets: `CONSUL_HTTP_TOKEN_PROD` |

---

## 1. 通常の apply 手順

```bash
# 1. 環境ディレクトリへ移動する
cd infra/terraform/environments/<env>  # dev / staging / prod

# 2. Consul に接続するための環境変数を設定する
export CONSUL_HTTP_ADDR="https://consul.internal.example.com:8500"
export CONSUL_HTTP_TOKEN="<token>"  # 上記の権限表を参照

# 3. 初期化（プロバイダーとモジュールのダウンロード）
terraform init

# 4. 差分を確認する（必ず apply 前に確認すること）
terraform plan

# 5. 適用する（prod は必ずチームレビュー後に実行すること）
terraform apply
```

---

## 2. State ロックの確認と解除

CI ジョブが中断した場合などに State がロックされたままになることがある。

### 2-1. ロック状態の確認

```bash
# Consul KV でロック状態を確認する
export CONSUL_HTTP_ADDR="https://consul.internal.example.com:8500"
export CONSUL_HTTP_TOKEN="<token>"

consul kv get terraform/k1s0/<env>/.lock
# ロックされている場合: JSON でロック保持者・タイムスタンプが表示される
# ロックされていない場合: No key exists at terraform/k1s0/<env>/.lock

# terraform コマンドでも確認できる（エラーメッセージにロック ID が含まれる）
cd infra/terraform/environments/<env>
terraform plan
# Error: Error acquiring the state lock
# ID: <LOCK_ID>  ← この ID をメモする
```

### 2-2. 安全なロック解除の判断基準

**以下をすべて確認してから解除すること:**

1. ロック保持者が GitHub Actions の CI/CD ジョブである
2. そのジョブが **失敗またはキャンセル** になっている（GitHub Actions の UI で確認）
3. 現在、別の terraform apply が実行中でない

```bash
# 安全を確認したらロックを解除する
terraform force-unlock -force <LOCK_ID>

# 解除後、State の整合性を確認する
terraform plan
# 差分が意図した範囲内であることを確認する
```

**注意**: 稼働中の apply を中断してロックを解除すると State が破損する可能性がある。必ず上記の確認をすること。

---

## 3. Consul ACL トークンの取得

### 3-1. 通常の開発・運用時

GitHub Secrets に保存されているトークンを使用する（直接 apply する場合は承認されたメンバーのみ）。

```bash
# self-hosted ランナー上では自動的に設定される
# ローカルから実行する場合は以下のいずれかを使用する:
# - dev: 開発用トークン（チーム共有）
# - staging/prod: インフラチームメンバーの個人トークン
```

### 3-2. トークンが失効した場合

```bash
# Consul サーバーで新規トークンを発行する（インフラチームのみ）
consul acl token create \
  --description "terraform-<env>-<date>" \
  --policy-name "terraform-<env>" \
  --token "<bootstrap_token>"
```

---

## 4. State バックアップからの復旧

Consul 自体が障害を起こした場合は [バックアップリストア手順書](../kubernetes/バックアップリストア手順書.md) の「6. Consul リストア手順」を参照。

### 4-1. State の手動エクスポート

万一のための State ローカルバックアップ手順:

```bash
cd infra/terraform/environments/<env>
terraform state pull > /tmp/terraform-state-<env>-<date>.json

# 内容確認
jq '.resources | length' /tmp/terraform-state-<env>-<date>.json
```

### 4-2. State の整合性確認

plan で差分が大量に出る場合は State の不整合を疑う:

```bash
# 特定リソースの State を確認する
terraform state show <resource_type>.<resource_name>

# State に含まれるリソース一覧
terraform state list

# 実環境と State を照合する（差分が予想外に多い場合は確認する）
terraform plan -target=<resource_type>.<resource_name>
```

---

## 5. 環境別の注意事項

| 環境 | 注意点 |
|-----|--------|
| **dev** | apply は自由に実施可。ただし他の開発者の作業を中断させないよう、apply 前に Slack 等で通知する |
| **staging** | インフラチームの1名以上のレビュー後に apply。週次メンテナンスウィンドウ（毎週水曜 10:00〜12:00 JST）を活用する |
| **prod** | インフラチームのリード承認必須。変更内容を GitHub PR に記録してから apply。**平日昼間（10:00〜17:00 JST）以外は原則禁止** |

### prod 環境の必須チェック

```bash
# prod apply 前に必ず確認する
# 1. バックアップが正常に完了していること
kubectl get cronjobs -n k1s0-system
kubectl get jobs -n k1s0-system | grep backup

# 2. example.com プレースホルダーが残っていないこと（variables.tf にバリデーションあり）
terraform validate

# 3. plan の変更範囲が承認済みであること（破壊的変更 = destroy がないことを確認）
terraform plan | grep -E "destroy|replace"
```

---

## 6. よくある問題と対処

| 問題 | 原因 | 対処 |
|-----|------|------|
| `Error acquiring the state lock` | 前の apply が中断した | 「2. State ロックの確認と解除」を参照 |
| `Backend initialization required` | backend.tf が変更された | `terraform init -reconfigure` を実行 |
| `Provider version constraints` | terraform.lock.hcl との不一致 | `terraform init -upgrade` を実行 |
| `Error: timeout` | Consul/K8s への接続タイムアウト | ネットワーク疎通を確認し再試行 |
| `Resource already exists` | State 外でリソースが手動作成された | `terraform import <resource> <id>` で State に取り込む |

---

## 関連ドキュメント

- [terraform設計.md](./terraform設計.md)
- [バックアップリストア手順書](../kubernetes/バックアップリストア手順書.md)
- [災害復旧計画](../overview/災害復旧計画.md)
