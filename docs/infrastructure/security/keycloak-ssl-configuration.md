# Keycloak SSL 設定ガイド

HIGH-21 監査対応: 開発レルムと本番レルムの `sslRequired` 設定の差異と運用上の注意事項をまとめたドキュメント。

## 概要

Keycloak レルムには `sslRequired` という設定があり、クライアントがどの接続で SSL/TLS を必須とするかを制御する。
k1s0 プロジェクトでは環境ごとに異なるレルムファイルを管理しており、設定値が意図的に異なる。

| 環境 | レルムファイル | sslRequired | 理由 |
|------|------------|-------------|------|
| 開発 | `infra/docker/keycloak/k1s0-realm.json` | `external` | ローカル開発環境では localhost の HTTP 通信を許可する必要があるため |
| 本番 | `infra/keycloak/realm-k1s0.json` | `all` | 全通信（内部通信を含む）に TLS を強制してトークン盗聴を防ぐ |

---

## 開発レルム: `sslRequired: external`

**ファイルパス:** `infra/docker/keycloak/k1s0-realm.json`

**設定値の意味:**
`external` は「外部ネットワーク（ループバック以外）からのアクセスには SSL を要求するが、
`localhost` / `127.0.0.1` からのアクセスには HTTP を許可する」を意味する。

**この設定を使う理由:**
- ローカル開発環境では `docker-compose up` で起動した Keycloak に HTTP（ポート 8080）でアクセスする
- フロントエンド開発サーバー（React: `localhost:3000`, Flutter: エミュレーター）は自己署名証明書を
  ブラウザに信頼させる手間なしに Keycloak と通信できる
- CI パイプラインの統合テストも同様に HTTP で Keycloak にアクセスする

**使用場所:** `docker-compose.yaml` の `keycloak` サービス

---

## 本番レルム: `sslRequired: all`

**ファイルパス:** `infra/keycloak/realm-k1s0.json`

**設定値の意味:**
`all` は「ループバック（localhost）を含む全ての接続に SSL を要求する」を意味する。

**この設定を使う理由:**
- 本番環境ではクラスタ内部通信（Pod 間）も TLS で保護することが求められる
- JWT トークンがネットワーク上で平文送信されるリスクを排除する
- セキュリティ監査（HIGH-21）の要件: 本番 Keycloak は全通信を暗号化すること

**使用場所:** Kubernetes 上の Keycloak デプロイ（`infra/keycloak/`）

---

## デプロイ時の注意事項

### 本番レルムファイルを必ず使用すること

Kubernetes 環境（staging/prod）に Keycloak をデプロイする際は、
**必ず `infra/keycloak/realm-k1s0.json` を使用すること。**
`infra/docker/keycloak/k1s0-realm.json`（開発用）を本番環境に適用してはならない。

```bash
# 本番レルムのインポート（Keycloak Admin API 経由）
kubectl exec -n k1s0-system deploy/keycloak -- \
  /opt/keycloak/bin/kc.sh import \
  --file /opt/keycloak/data/import/realm-k1s0.json
```

### cert-manager との連携

本番環境では `infra/kubernetes/cert-manager/cluster-issuer.yaml` が定義する
ClusterIssuer を使って TLS 証明書を自動更新する。
Keycloak の Ingress リソースには `cert-manager.io/cluster-issuer` アノテーションを付与すること。

---

## SSL 要件の差異がテスト結果に影響する場合

### 症状

CI の統合テストで Keycloak への認証が以下のエラーで失敗する場合がある:

```
HTTP protocol is not allowed [https://auth.example.com/realms/k1s0]
SSL required for all connections
```

### 原因

テスト環境が本番レルム設定（`sslRequired: all`）を誤って読み込んでいる場合に発生する。

### 対処法

1. **環境変数で切り替える:** `docker-compose.yaml` の `keycloak` サービスに
   以下の環境変数を設定し、起動時にインポートするファイルを環境別に選択する

   ```yaml
   environment:
     KC_IMPORT: /opt/keycloak/data/import/k1s0-realm.json   # 開発用
   ```

2. **ローカル開発環境を明示的に指定する:** `docker-compose.dev.yaml` で上書きすることで、
   誤って本番レルムが読み込まれないようにする

3. **CI テスト環境の確認:** GitHub Actions では `services:` コンテナとして起動する
   Keycloak が開発用レルムを使っているかを確認する

   ```yaml
   services:
     keycloak:
       image: quay.io/keycloak/keycloak:26.x
       volumes:
         - ./infra/docker/keycloak/k1s0-realm.json:/opt/keycloak/data/import/k1s0-realm.json:ro
   ```

4. **接続プロトコルの確認:** テストコードが `http://` で Keycloak に接続していないか確認する。
   `sslRequired: external` の場合は localhost 向け HTTP は許可されるが、
   `sslRequired: all` の場合は HTTP は完全に拒否される。

---

## 関連ファイル

| ファイル | 説明 |
|---------|------|
| `infra/docker/keycloak/k1s0-realm.json` | ローカル開発用レルム定義（`sslRequired: external`） |
| `infra/keycloak/realm-k1s0.json` | 本番環境用レルム定義（`sslRequired: all`） |
| `infra/kubernetes/cert-manager/cluster-issuer.yaml` | Let's Encrypt ClusterIssuer 定義 |
| `docs/architecture/adr/0045-vault-per-service-role.md` | Vault per-service ロール分離 ADR |

## 参考資料

- [Keycloak SSL/TLS 設定ドキュメント](https://www.keycloak.org/docs/latest/server_admin/#_ssl_modes)
- [Kubernetes cert-manager](https://cert-manager.io/docs/)
