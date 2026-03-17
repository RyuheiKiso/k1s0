# Secrets 管理設計

F-012: シークレット管理の運用設計。HashiCorp Vault を用いたシークレットのライフサイクル管理、ローテーションポリシー、緊急対応手順を定義する。

元ドキュメント: [Vault設計.md](./Vault設計.md)

---

## 基本方針

- すべてのシークレットは **HashiCorp Vault** で一元管理する（環境変数への直接埋め込み禁止）
- Kubernetes 環境では **Vault Agent Sidecar** パターンでシークレットを Pod に注入する
- シークレットローテーションは自動化を原則とし、手動ローテーションは緊急時のみとする
- 環境ごとにシークレットパスを分離し、クロス環境アクセスを防止する

---

## HashiCorp Vault 統合概要

### アーキテクチャ

```
┌──────────────────┐    ┌──────────────────┐    ┌──────────────────┐
│   Application    │    │   Vault Agent    │    │   Vault Server   │
│   (Pod)          │◄───│   (Sidecar)      │◄───│   (HA Cluster)   │
│                  │    │                  │    │                  │
│  /vault/secrets/ │    │  Token 自動更新  │    │  Seal: Shamir    │
│  にファイル読取  │    │  Secret キャッシュ│    │  Storage: Raft   │
└──────────────────┘    └──────────────────┘    └──────────────────┘
```

### 使用するシークレットエンジン

| エンジン | マウントパス | 用途 | ローテーション |
| --- | --- | --- | --- |
| KV v2 | `secret/` | 静的シークレット（API キー、設定値等） | 手動 + ポリシーベース |
| Database | `database/` | DB クレデンシャルの動的生成 | Vault 自動（TTL ベース） |
| PKI | `pki_int/` | 内部 TLS 証明書の発行 | Vault 自動（有効期限ベース） |

詳細なパス体系・ポリシーは [Vault設計.md](./Vault設計.md) を参照。

---

## シークレットローテーションポリシー

### ローテーション間隔

| シークレット種別 | ローテーション間隔 | 方式 | 責任者 |
| --- | --- | --- | --- |
| DB パスワード（動的） | 24 時間 | Vault Database エンジン自動 | Vault |
| DB パスワード（静的） | 90 日 | Terraform + Vault KV v2 | インフラチーム |
| API キー | 90 日 | Vault KV v2 + CI/CD パイプライン | インフラチーム |
| TLS 証明書 | 90 日 | Vault PKI エンジン自動 | Vault |
| JWT 署名鍵 | 90 日 | Keycloak 鍵ローテーション | 認証チーム |
| Kafka SASL クレデンシャル | 180 日 | Vault KV v2 + 手動更新 | インフラチーム |
| Redis AUTH パスワード | 90 日 | Vault KV v2 + 手動更新 | インフラチーム |

### ローテーション通知

ローテーション期限の **14 日前** に Alertmanager 経由で Microsoft Teams へ通知する。

```yaml
# Prometheus アラートルール例
- alert: SecretRotationDue
  expr: vault_secret_last_rotation_days > 76
  for: 1h
  labels:
    severity: warning
  annotations:
    summary: "シークレットのローテーション期限が近づいています"
    description: "{{ $labels.secret_path }} のローテーションが {{ $value }} 日前に実施されました（閾値: 90 日）"
```

---

## Kubernetes Vault Agent Sidecar パターン

### 動作フロー

1. Pod 起動時に Vault Agent Init Container が Kubernetes Auth で Vault に認証する
2. Vault Agent Sidecar が指定パスのシークレットを取得し、共有ボリュームにファイルとして書き込む
3. アプリケーションコンテナはファイルシステム経由でシークレットを読み取る
4. Vault Agent がバックグラウンドで TTL を監視し、期限前にシークレットを自動更新する

### Deployment アノテーション

```yaml
spec:
  template:
    metadata:
      annotations:
        # Vault Agent Injector の有効化
        vault.hashicorp.com/agent-inject: "true"
        # Kubernetes Auth ロール名（サービス名と一致させる）
        vault.hashicorp.com/role: "auth-server"
        # Init Container で起動前にシークレットを注入する
        vault.hashicorp.com/agent-pre-populate-only: "false"
        # シークレットのキャッシュを有効化（Vault 障害時の耐性向上）
        vault.hashicorp.com/agent-cache-enable: "true"
        # 静的シークレット（KV v2）
        vault.hashicorp.com/agent-inject-secret-api-key: "secret/data/k1s0/system/auth-server/api-key"
        # 動的シークレット（Database）
        vault.hashicorp.com/agent-inject-secret-db-creds: "database/creds/auth-server-rw"
        vault.hashicorp.com/agent-inject-template-db-creds: |
          {{- with secret "database/creds/auth-server-rw" -}}
          host=postgres.k1s0-system.svc.cluster.local
          port=5432
          dbname=auth_db
          user={{ .Data.username }}
          password={{ .Data.password }}
          {{- end -}}
```

### ファイルマウントパス規則

Pod にシークレットをファイルとして注入する際のマウントパスは以下に従う。

| マウントパス | 用途 |
| --- | --- |
| `/vault/secrets/db-password` | DB パスワード |
| `/vault/secrets/db-creds` | DB 動的クレデンシャル |
| `/vault/secrets/api-key` | API キー |
| `/vault/secrets/redis-password` | Redis AUTH パスワード |
| `/vault/secrets/oidc` | OIDC Client Secret |
| `/vault/secrets/kafka-sasl` | Kafka SASL クレデンシャル |

### アプリケーション側の読み取りパターン

```rust
// Rust: シークレットファイルの読み取り
use std::fs;

/// Vault Agent が注入したシークレットファイルを読み取る
fn read_vault_secret(secret_name: &str) -> Result<String, std::io::Error> {
    let path = format!("/vault/secrets/{}", secret_name);
    fs::read_to_string(&path).map(|s| s.trim().to_string())
}
```

```go
// Go: シークレットファイルの読み取り
// VaultAgent が注入したシークレットファイルを読み取る
func readVaultSecret(secretName string) (string, error) {
    path := filepath.Join("/vault/secrets", secretName)
    data, err := os.ReadFile(path)
    if err != nil {
        return "", fmt.Errorf("vault secret %s の読み取りに失敗: %w", secretName, err)
    }
    return strings.TrimSpace(string(data)), nil
}
```

---

## 環境別シークレットパス

環境ごとにシークレットパスを分離し、クロス環境アクセスを防止する。

### パス構造

```
secret/data/k1s0/{environment}/{tier}/{service}/{secret-type}
```

| 環境 | パスプレフィックス | Vault クラスタ | 用途 |
| --- | --- | --- | --- |
| ローカル開発 | `secret/data/k1s0/dev/` | ローカル Vault（dev モード） | 開発者ローカル環境 |
| staging | `secret/data/k1s0/staging/` | 共有 Vault クラスタ | 統合テスト・リリース検証 |
| production | `secret/data/k1s0/prod/` | 本番 Vault クラスタ（HA） | 本番運用 |

### ローカル開発環境

ローカル開発環境では Vault を dev モードで起動し、テスト用シークレットを自動シードする。

```bash
# docker-compose で Vault dev モードを起動
docker compose --profile infra up -d vault

# テスト用シークレットのシード（初回のみ）
vault kv put secret/data/k1s0/dev/system/auth-server/database password=dev-password
vault kv put secret/data/k1s0/dev/system/kafka/sasl username=dev-user password=dev-password
```

### 環境間のアクセス制御

- 各環境の Vault クラスタは独立したポリシーを持つ
- staging → production への昇格時にシークレットを手動で同期しない（Terraform で管理）
- CI/CD パイプラインは AppRole 認証で環境に対応した Vault にアクセスする

---

## 緊急シークレットローテーション手順

シークレットの漏洩が疑われる場合の緊急対応手順を定義する。

### 緊急度レベル

| レベル | 状況 | 対応時間目標 |
| --- | --- | --- |
| Critical | 本番 DB クレデンシャル・JWT 署名鍵の漏洩 | 30 分以内 |
| High | API キー・SASL クレデンシャルの漏洩 | 2 時間以内 |
| Medium | 開発環境シークレットの漏洩 | 24 時間以内 |

### 緊急ローテーション手順

#### 1. DB クレデンシャル漏洩時

```bash
# 1. 動的クレデンシャルの即時無効化（Vault のリースを全て revoke）
vault lease revoke -prefix database/creds/

# 2. 静的パスワードの即時変更
vault kv put secret/data/k1s0/prod/system/{service}/database password=$(openssl rand -base64 32)

# 3. 影響を受ける Pod の再起動（Vault Agent が新しいクレデンシャルを取得）
kubectl rollout restart deployment/{service} -n k1s0-system
```

#### 2. API キー漏洩時

```bash
# 1. 現在のキーを無効化し、新しいキーを発行
vault kv put secret/data/k1s0/prod/system/{service}/api-key key=$(openssl rand -hex 32)

# 2. 影響を受ける Pod の再起動
kubectl rollout restart deployment/{service} -n k1s0-system

# 3. 外部連携先に新しいキーを通知（該当する場合）
```

#### 3. TLS 証明書漏洩時

```bash
# 1. 現在の証明書を CRL に追加
vault write pki_int/revoke serial_number=<serial>

# 2. 新しい証明書を発行
vault write pki_int/issue/system common_name="{service}.k1s0-system.svc.cluster.local"

# 3. Istio Sidecar の再起動で新しい証明書を適用
kubectl rollout restart deployment/{service} -n k1s0-system
```

### 事後対応

1. **監査ログの確認**: Vault 監査ログから漏洩経路を特定する
2. **影響範囲の調査**: 漏洩したシークレットでアクセス可能だったリソースを洗い出す
3. **インシデントレポート**: 漏洩の原因・影響・対応をドキュメント化する
4. **再発防止策**: ポリシーの見直し・アクセス権限の最小化を実施する

---

## 監査とコンプライアンス

### Vault 監査ログ

すべてのシークレットアクセスは Vault 監査ログに記録される。

```hcl
# 監査ログ設定（audit.tf）
resource "vault_audit" "file" {
  type = "file"
  options = {
    file_path = "/vault/logs/audit.log"
    log_raw   = false   # シークレット値をマスク
  }
}
```

### 監査対象

| 操作 | 記録内容 |
| --- | --- |
| 認証試行 | 成功・失敗の両方、認証方式、クライアント IP |
| シークレット読み取り | パス、リクエスト元の identity、タイムスタンプ |
| ポリシー変更 | 変更前後の diff、実行者 |
| シークレット書き込み | パス、バージョン番号（値はマスク） |

### 定期監査

- **月次**: シークレットアクセスパターンの異常検知レポートを確認する
- **四半期**: 不要なシークレットパス・ポリシーの棚卸しを実施する
- **年次**: 全シークレットの棚卸しと Vault ポリシーの全面レビューを実施する

---

## 関連ドキュメント

- [Vault設計.md](./Vault設計.md) -- Vault の認証方式・シークレットエンジン・パス体系・ポリシー
- [認証認可設計.md](../../architecture/auth/認証認可設計.md) -- 基本方針・技術スタック
- [helm設計.md](../kubernetes/helm設計.md) -- Helm Chart・Vault Agent Injector
- [terraform設計.md](../terraform/terraform設計.md) -- Terraform モジュール
- [CI-CD設計.md](../cicd/CI-CD設計.md) -- CI/CD パイプラインとシークレットの連携
