# 設定と秘密情報の規約

本ドキュメントは、k1s0 における設定（config）と秘密情報（secrets）の取り扱い規約を定義する。

## 1. 基本方針

- **環境変数は使用しない**（アプリ実装での `std::env` / `os.Getenv` 等は禁止）
- framework 自身の設定は **YAML ファイル**で制御する
- feature 固有の動的設定は **DB（`fw_m_setting`）** で管理する
- 秘密情報は **ファイル参照**のみを保持し、値そのものを YAML/DB に置かない

## 2. 設定の優先順位

```
1. CLI 引数（--config / --env / --secrets-dir）
       ↓ override
2. YAML（config/{env}.yaml）
       ↓ override
3. DB（fw_m_setting）
```

## 3. CLI 引数

サービスは起動時に以下の引数を明示的に受け取る：

| 引数 | 説明 | 例 |
|------|------|-----|
| `--env` | 環境名 | `--env dev` |
| `--config` | 設定ファイルパス | `--config /etc/k1s0/config/dev.yaml` |
| `--secrets-dir` | 秘密情報ディレクトリ | `--secrets-dir /var/run/secrets/k1s0/` |

**暗黙の環境選択は禁止**（必ず明示する）

## 4. YAML 設定ファイル

### 4.1 配置先

```
{service}/config/
├── default.yaml  # 共通デフォルト
├── dev.yaml      # 開発環境
├── stg.yaml      # ステージング環境
└── prod.yaml     # 本番環境
```

### 4.2 YAML に書いてよいもの

- 非機密の静的設定（ホスト名、ポート、タイムアウト等）
- 秘密情報への **参照**（`*_file` キー）

### 4.3 YAML に書いてはいけないもの

- パスワード、API キー、トークン等の **秘密情報そのもの**

### 4.4 キー例

```yaml
db:
  host: localhost
  port: 5432
  name: k1s0_dev
  user: k1s0_user
  password_file: /var/run/secrets/k1s0/db_password  # 値ではなく参照

auth:
  jwt_private_key_file: /var/run/secrets/k1s0/jwt_private_key.pem
  jwt_public_key_file: /var/run/secrets/k1s0/jwt_public_key.pem

http:
  timeout_ms: 5000
  max_connections: 100
```

## 5. 秘密情報の配布

### 5.1 Kubernetes 環境

```yaml
# Pod spec（抜粋）
volumes:
  - name: config
    configMap:
      name: {service}-config
  - name: secrets
    secret:
      secretName: {service}-secrets

containers:
  - name: {service}
    volumeMounts:
      - name: config
        mountPath: /etc/k1s0/config/
      - name: secrets
        mountPath: /var/run/secrets/k1s0/
    args:
      - --env
      - $(ENV)
      - --config
      - /etc/k1s0/config/$(ENV).yaml
      - --secrets-dir
      - /var/run/secrets/k1s0/
```

### 5.2 ローカル開発

```
{service}/
└── secrets/            # .gitignore に含める
    └── dev/
        ├── db_password
        └── jwt_private_key.pem
```

起動例：
```bash
cargo run -- --env dev --config ./config/dev.yaml --secrets-dir ./secrets/dev/
```

## 6. DB 設定（fw_m_setting）

### 6.1 用途

- feature 固有の動的設定
- 実行時に変更可能な設定

### 6.2 setting_key 命名規則

```
{category}.{name}
```

- 小文字 + 数字 + アンダースコア
- ドット区切り

例：
- `http.timeout_ms`
- `db.pool_size`
- `auth.jwt_ttl_sec`
- `feature.flag_x`

### 6.3 config-service 障害時の挙動

#### 起動時

設定単位で以下から選択（既定は A）：

| 選択肢 | 挙動 |
|--------|------|
| A | キャッシュがあれば起動可（キャッシュなしなら起動不可） |
| B | フェイルオープン（DB 設定なしでも起動、YAML 既定値で動作） |
| C | 起動不可（設定取得が必須な機能に適用） |

#### 稼働中

- 取得失敗時は直前のキャッシュを使用
- 一定時間後にリトライ
- 失敗はメトリクス/ログ/トレースで観測可能

## 7. 禁止事項

| 禁止事項 | 理由 |
|----------|------|
| 環境変数での設定注入 | 監査困難、誤設定リスク |
| `envFrom` / `secretKeyRef` での Secret 注入 | 上記同様 |
| ConfigMap への機密値直書き | Git 等への漏洩リスク |
| 暗黙の環境選択（`--env` 省略） | 意図しない環境での動作リスク |

## 8. 検査（lint）

`k1s0 lint` は以下を検査する：

- `config/{env}.yaml` に機密パターン（password/token/secret/key 等）の値が直書きされていないか
- `*_file` 以外の機密キーがないか

## 関連ドキュメント

- [サービス構成規約](service-structure.md)
- [構想.md](../../work/構想.md): 全体方針（11. 設定と秘密情報）
