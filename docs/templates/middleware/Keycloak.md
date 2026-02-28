# テンプレート仕様 — Keycloak

## 概要

k1s0 CLI ひな形生成のKeycloakテンプレート仕様。サービス単位の Keycloak クライアント設定（clientId、redirectUris、webOrigins、bearerOnly、publicClient 等）を、サービスの `tier` と `kind` に応じて自動生成する。

認証認可設計の全体像は [認証認可設計](../../architecture/auth/認証認可設計.md) を参照。

## 生成対象

| kind       | keycloak-client |
| ---------- | --------------- |
| `server`   | 生成する        |
| `bff`      | 生成する        |
| `client`   | 生成する        |

- **server**: Bearer-only クライアント（トークン検証のみ）
- **bff**: Confidential クライアント（Authorization Code Flow）
- **client**: Public クライアント（Authorization Code Flow + PKCE）

## 配置パス

生成されるリソースファイルは `infra/keycloak/` 配下にサービス名ディレクトリを作成して配置する。

| ファイル           | 配置パス                                                |
| ------------------ | ------------------------------------------------------- |
| Keycloak Client    | `infra/keycloak/{{ service_name }}/keycloak-client.json` |

## テンプレートファイル一覧

テンプレートは `CLI/templates/keycloak/` 配下に配置する。

| テンプレートファイル              | 生成先                                                  | 説明                          |
| --------------------------------- | ------------------------------------------------------- | ----------------------------- |
| `keycloak-client.json.tera`       | `infra/keycloak/{{ service_name }}/keycloak-client.json` | Keycloak クライアント登録設定 |

### ディレクトリ構成

```
CLI/
└── templates/
    └── keycloak/
        └── keycloak-client.json.tera
```

## 使用するテンプレート変数

Keycloak テンプレートで使用する変数を以下に示す。変数の定義と導出ルールの詳細は [テンプレートエンジン仕様](../engine/テンプレートエンジン仕様.md) を参照。

| 変数名                  | 型     | 用途                                       |
| ----------------------- | ------ | ------------------------------------------ |
| `service_name`          | String | clientId、リダイレクト URI のホスト名       |
| `service_name_snake`    | String | クライアントスコープ名                     |
| `service_name_pascal`   | String | クライアント表示名                         |
| `tier`                  | String | Namespace 導出、Realm 選択                 |
| `namespace`             | String | リダイレクト URI の Namespace 部分          |
| `kind`                  | String | クライアントタイプの決定（bearer-only/confidential/public） |

### kind 別クライアントタイプ

| kind     | クライアントタイプ | bearerOnly | publicClient | 認証フロー                        |
| -------- | ------------------ | ---------- | ------------ | --------------------------------- |
| `server` | Bearer-only        | `true`     | `false`      | トークン検証のみ                  |
| `bff`    | Confidential       | `false`    | `false`      | Authorization Code Flow           |
| `client` | Public             | `false`    | `true`       | Authorization Code Flow + PKCE    |

---

## Keycloak Client テンプレート（keycloak-client.json.tera）

サービスの `kind` に応じて適切なクライアントタイプの Keycloak クライアント設定を生成する。

```tera
{
  "clientId": "{{ service_name }}",
  "name": "{{ service_name_pascal }}",
  "enabled": true,
  "protocol": "openid-connect",
{% if kind == "server" %}
  "bearerOnly": true,
  "publicClient": false,
  "directAccessGrantsEnabled": false,
  "standardFlowEnabled": false,
  "serviceAccountsEnabled": true,
{% elif kind == "bff" %}
  "bearerOnly": false,
  "publicClient": false,
  "directAccessGrantsEnabled": false,
  "standardFlowEnabled": true,
  "serviceAccountsEnabled": true,
  "secret": "${KEYCLOAK_CLIENT_SECRET}",
  "redirectUris": [
    "https://{{ service_name }}.{{ namespace }}.svc.cluster.local/*",
    "https://{{ service_name }}.{{ namespace }}/*"
  ],
  "webOrigins": [
    "https://{{ service_name }}.{{ namespace }}.svc.cluster.local",
    "https://{{ service_name }}.{{ namespace }}"
  ],
{% elif kind == "client" %}
  "bearerOnly": false,
  "publicClient": true,
  "directAccessGrantsEnabled": false,
  "standardFlowEnabled": true,
  "serviceAccountsEnabled": false,
  "redirectUris": [
    "https://{{ service_name }}.{{ namespace }}/*",
    "http://localhost:3000/*",
    "http://localhost:5173/*"
  ],
  "webOrigins": [
    "https://{{ service_name }}.{{ namespace }}",
    "http://localhost:3000",
    "http://localhost:5173"
  ],
  "attributes": {
    "pkce.code.challenge.method": "S256"
  },
{% endif %}
  "defaultClientScopes": [
    "openid",
    "profile",
    "email",
    "{{ service_name_snake }}_scope"
  ],
  "optionalClientScopes": [
    "offline_access"
  ]
}
```

### ポイント

- **server（Bearer-only）**: トークン検証のみ行い、リダイレクト URI は不要。`serviceAccountsEnabled: true` でサービス間通信用のクライアント認証情報を取得可能にする
- **bff（Confidential）**: Authorization Code Flow を使用し、クライアントシークレットで認証する。`redirectUris` にクラスタ内のサービス URL を設定する
- **client（Public）**: PKCE（Proof Key for Code Exchange）を使用し、クライアントシークレットなしで安全に認証する。ローカル開発用の `localhost` リダイレクトを含める
- クライアントシークレットは `${KEYCLOAK_CLIENT_SECRET}` プレースホルダーで定義し、デプロイ時に Vault 等から注入する
- `defaultClientScopes` にサービス固有のスコープ（`{{ service_name_snake }}_scope`）を含める

---

## 条件付き生成表

CLI の対話フローで選択されたオプションに応じて、生成されるリソースの内容が変わる。

| 条件            | 選択肢                            | 生成への影響                                              |
| --------------- | --------------------------------- | --------------------------------------------------------- |
| kind (`kind`)   | `server`                          | Bearer-only クライアント（トークン検証のみ）              |
| kind (`kind`)   | `bff`                             | Confidential クライアント（Authorization Code Flow）      |
| kind (`kind`)   | `client`                          | Public クライアント（PKCE）                               |
| kind (`kind`)   | `library` / `database`            | Keycloak クライアントを生成しない                         |
| Tier (`tier`)   | `system` / `business` / `service` | Namespace に応じたリダイレクト URI の生成                  |

---

## 生成例

### system Tier の server の場合

入力:
```json
{
  "service_name": "auth-service",
  "service_name_snake": "auth_service",
  "service_name_pascal": "AuthService",
  "tier": "system",
  "namespace": "k1s0-system",
  "kind": "server"
}
```

生成されるファイル:
- `infra/keycloak/auth-service/keycloak-client.json` -- Bearer-only、serviceAccountsEnabled=true

### service Tier の BFF の場合

入力:
```json
{
  "service_name": "order-bff",
  "service_name_snake": "order_bff",
  "service_name_pascal": "OrderBff",
  "tier": "service",
  "namespace": "k1s0-service",
  "kind": "bff"
}
```

生成されるファイル:
- `infra/keycloak/order-bff/keycloak-client.json` -- Confidential、redirectUris にクラスタ内 URL

### service Tier の client の場合

入力:
```json
{
  "service_name": "order-web",
  "service_name_snake": "order_web",
  "service_name_pascal": "OrderWeb",
  "tier": "service",
  "namespace": "k1s0-service",
  "kind": "client"
}
```

生成されるファイル:
- `infra/keycloak/order-web/keycloak-client.json` -- Public、PKCE 有効、localhost リダイレクト含む

---

## 関連ドキュメント

> 共通参照は [テンプレートエンジン仕様.md](../engine/テンプレートエンジン仕様.md) を参照。

- [認証認可設計](../../architecture/auth/認証認可設計.md) -- 認証・認可の全体設計
- [テンプレート仕様-Kong](Kong.md) -- Kong テンプレート仕様（認証プラグイン連携）
- [テンプレート仕様-ServiceMesh](ServiceMesh.md) -- ServiceMesh テンプレート仕様（AuthorizationPolicy 連携）
