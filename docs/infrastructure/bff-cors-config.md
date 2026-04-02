# BFF Proxy CORS 設定管理手順

## 概要

BFF Proxy（`regions/system/server/go/bff-proxy`）の CORS 設定は、フロントエンドクライアントと BFF サーバー間の
クロスオリジンリクエストを制御する重要なセキュリティ設定です。本ドキュメントでは、環境別の設定方法と
本番環境への安全な適用手順を説明します。

---

## 1. 開発環境の現在の設定

開発環境の CORS 設定は `config/config.yaml` に記載されています。

```yaml
# CORS設定（H-1対応）: 明示的な Origin ホワイトリストで異オリジンリクエストを制御する
cors:
  enabled: true
  allow_origins:
    # localhost:3000 - React (create-react-app)
    # localhost:5173 - Vite (React/Vue/Svelte)
    # localhost:4200 - Angular
    # localhost:8080 - BFF Proxy ローカル直接アクセス
    # localhost:3001 - Flutter Web (flutter run --web-port 3001)
    - "http://localhost:3000"
    - "http://localhost:5173"
    - "http://localhost:4200"
    - "http://localhost:3001"
  credentials_paths:
    - "/auth/"
    - "/api/"
  max_age_secs: 600
```

`allow_origins` には開発時に使用する一般的な localhost ポートが列挙されています。

### Docker 環境

Docker Compose 環境では `config/config.docker.yaml` が使用されます。CORS の `allow_origins` は
`config.yaml` の値が引き継がれます（差分のみ記載する構成のため）。

### ステージング環境

`config/config.staging.yaml` では本番相当のドメインのみを許可しています。

```yaml
cors:
  enabled: true
  allow_origins:
    - "https://app.staging.k1s0.internal"
  credentials_paths:
    - "/auth/"
    - "/api/"
  max_age_secs: 600
```

---

## 2. 本番環境での CORS ホワイトリスト更新手順

### 手順概要

1. **許可するオリジンの確定**: 本番フロントエンドのドメインを確認する
2. **Helm values ファイルの更新**: `infra/helm/bff-proxy/values-production.yaml` の ConfigMap を編集する
3. **差分レビュー**: セキュリティチームによるレビューを実施する
4. **デプロイ**: `helm upgrade` でロールアウトする
5. **動作確認**: ブラウザの DevTools でプリフライトリクエストが通ることを確認する

### Helm ConfigMap による設定注入

本番環境の CORS 設定は Kubernetes ConfigMap に保存し、BFF Proxy に注入します。

```yaml
# infra/helm/bff-proxy/values-production.yaml
config:
  data:
    config.yaml: |
      cors:
        enabled: true
        allow_origins:
          - "https://app.k1s0.example.com"
          - "https://admin.k1s0.example.com"
        credentials_paths:
          - "/auth/"
          - "/api/"
        max_age_secs: 600
```

### 設定変更のデプロイコマンド

```bash
# 本番環境への Helm デプロイ
helm upgrade bff-proxy infra/helm/bff-proxy \
  --namespace k1s0-system \
  --values infra/helm/bff-proxy/values-production.yaml \
  --wait

# ConfigMap の内容確認
kubectl get configmap bff-proxy-config -n k1s0-system -o yaml
```

---

## 3. 環境変数または ConfigMap での設定注入方法

### 方法A: Kubernetes ConfigMap（推奨）

BFF Proxy は起動時に `/app/config/config.yaml` を読み込みます。
ConfigMap をボリュームマウントすることで、環境別の設定を注入できます。

```yaml
# Kubernetes Deployment 抜粋
volumeMounts:
  - name: config
    mountPath: /app/config
    readOnly: true
volumes:
  - name: config
    configMap:
      name: bff-proxy-config
```

ConfigMap の `data.config.yaml` キーに YAML 設定を記載します。

```yaml
# bff-proxy-config ConfigMap
apiVersion: v1
kind: ConfigMap
metadata:
  name: bff-proxy-config
  namespace: k1s0-system
data:
  config.yaml: |
    cors:
      enabled: true
      allow_origins:
        - "https://app.k1s0.example.com"
      credentials_paths:
        - "/auth/"
        - "/api/"
      max_age_secs: 600
```

### 方法B: 環境変数によるオーバーライド

BFF Proxy は Viper を使用しているため、環境変数でも設定を上書きできます。
環境変数名は設定キーを大文字化し `.` を `_` に置換した形式です。

```bash
# 環境変数でのオーバーライド例（Kubernetes Secret / Deployment env に設定）
CORS_ALLOW_ORIGINS="https://app.k1s0.example.com,https://admin.k1s0.example.com"
```

ただし、`allow_origins` はリスト型のため ConfigMap による注入（方法A）を推奨します。

---

## 4. セキュリティ上の注意事項

### ワイルドカード禁止

```yaml
# NG: ワイルドカードは絶対に使用しない
cors:
  allow_origins:
    - "*"  # 禁止: credentials=true と組み合わせると CSRF 脆弱性が発生する

# OK: 明示的なドメインのみ許可する
cors:
  allow_origins:
    - "https://app.k1s0.example.com"
```

`Access-Control-Allow-Credentials: true` を返すエンドポイント（`/auth/`・`/api/`）では、
ワイルドカードを使用するとブラウザがリクエストを拒否するだけでなく、
正しく実装されていない場合はセッションクッキーが第三者に送信されるリスクがあります。

### localhost を本番に含めない

```yaml
# NG: 本番設定に localhost を混入しない
allow_origins:
  - "https://app.k1s0.example.com"
  - "http://localhost:3000"  # 禁止: 本番環境には不要

# OK: 本番ドメインのみ
allow_origins:
  - "https://app.k1s0.example.com"
```

### HTTPS のみ許可する

本番環境では必ず `https://` スキームのオリジンのみを許可します。
`http://` オリジンを許可すると中間者攻撃（MITM）のリスクがあります。

### credentials_paths の最小化

```yaml
# credentials_paths には認証が必要なパスのみを列挙する
credentials_paths:
  - "/auth/"   # 認証フロー（ログイン・コールバック・ログアウト）
  - "/api/"    # 認証済みユーザー向け API プロキシ
# /healthz や /metrics には credentials を付与しない
```

### max_age_secs の上限

`max_age_secs`（プリフライトキャッシュ時間）は 600 秒（10 分）を推奨します。
長くしすぎると、許可リスト変更後もブラウザが古いキャッシュを使用し続けるリスクがあります。

---

## 5. 設定変更チェックリスト

本番環境の CORS 設定を変更する際は以下を確認してください。

- [ ] `allow_origins` にワイルドカード（`*`）が含まれていないこと
- [ ] `allow_origins` に `http://localhost:*` が含まれていないこと
- [ ] すべてのオリジンが `https://` スキームであること
- [ ] 許可するオリジンは業務上必要な最小限であること
- [ ] `credentials_paths` が `/auth/` と `/api/` のみであること
- [ ] セキュリティチームのレビューを受けていること
- [ ] ステージング環境で動作確認済みであること

---

## 参考

- `regions/system/server/go/bff-proxy/config/config.yaml` — 開発環境デフォルト設定
- `regions/system/server/go/bff-proxy/config/config.staging.yaml` — ステージング環境設定
- `regions/system/server/go/bff-proxy/config/config.docker.yaml` — Docker Compose 環境設定
- `docs/infrastructure/security/` — セキュリティ全般ポリシー
