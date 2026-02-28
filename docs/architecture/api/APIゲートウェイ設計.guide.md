# API ゲートウェイ ガイド

> **仕様**: テーブル定義・構成管理は [APIゲートウェイ設計.md](./APIゲートウェイ設計.md) を参照。

## 基本方針

- API ゲートウェイは **Kong** を採用し、**DB-backed モード**（PostgreSQL）で運用する
- 管理は **Admin API** 経由で行い、decK で設定を宣言的に管理する
- CI/CD パイプラインから decK を実行し、設定変更をコードレビュー可能にする
- 認証・レート制限・ログ等の横断的関心事は Kong プラグインで一元管理する

## DB-backed モードの利点

- Admin API による動的な設定変更が可能
- 複数 Kong インスタンス間で設定を自動共有
- decK によるバージョン管理・CI/CD 連携が容易

## Admin API による管理

Kong の設定は Admin API を通じて管理する。直接の Admin API 呼び出しは運用時のデバッグに限定し、通常の設定変更は decK 経由で行う。

```bash
# Service の作成例
curl -X POST http://kong-admin:8001/services \
  -d name=order-v1 \
  -d url=http://order-server.k1s0-service.svc.cluster.local:80

# Route の作成例
curl -X POST http://kong-admin:8001/services/order-v1/routes \
  -d name=order-v1-route \
  -d 'paths[]=/api/v1/orders' \
  -d strip_path=false
```

## decK による宣言的設定管理

Kong の設定を YAML ファイルで宣言的に管理し、Git でバージョン管理する。

### kong.yaml の例

```yaml
# infra/kong/kong.yaml
_format_version: "3.0"

services:
  # system Tier
  - name: auth-v1
    url: http://auth-server.k1s0-system.svc.cluster.local:80
    routes:
      - name: auth-v1-route
        paths:
          - /api/v1/auth
        strip_path: false
      - name: auth-v1-login
        paths:
          - /api/v1/auth/login
        strip_path: false
        plugins:
          - name: rate-limiting
            config:
              minute: 30                  # ブルートフォース防止（API設計.md 参照）
              policy: redis
              redis_host: redis.k1s0-system.svc.cluster.local

  - name: saga-v1
    url: http://saga-server.k1s0-system.svc.cluster.local:80
    routes:
      - name: saga-v1-sagas
        paths:
          - /api/v1/sagas
        strip_path: false
        methods: [GET, POST]
      - name: saga-v1-workflows
        paths:
          - /api/v1/workflows
        strip_path: false
        methods: [GET, POST]
    plugins:
      - name: request-transformer
        config:
          add:
            headers:
              - X-Service-Name:saga-server

  - name: dlq-manager-v1
    url: http://dlq-manager.k1s0-system.svc.cluster.local:80
    routes:
      - name: dlq-manager-v1-route
        paths:
          - /api/v1/dlq
        strip_path: false
        methods: [GET, POST, DELETE]
    plugins:
      - name: request-transformer
        config:
          add:
            headers:
              - X-Service-Name:dlq-manager

  # service Tier
  - name: order-v1
    url: http://order-server.k1s0-service.svc.cluster.local:80
    routes:
      - name: order-v1-route
        paths:
          - /api/v1/orders
        strip_path: false

plugins:
  # グローバルプラグイン
  - name: rate-limiting
    config:
      minute: 500
      policy: redis
      redis_host: redis.k1s0-system.svc.cluster.local
      redis_port: 6379
      redis_database: 1
      fault_tolerant: true
      hide_client_headers: false

  - name: jwt
    config:
      key_claim_name: kid
      claims_to_verify:
        - exp

  - name: cors
    config:
      origins:
        - "https://*.k1s0.internal.example.com"
      credentials: true

  - name: prometheus
    config:
      per_consumer: true
      status_code_metrics: true
```

## CI/CD 連携（decK）

```yaml
# .github/workflows/kong-sync.yaml
name: Kong Config Sync

on:
  push:
    branches: [main]
    paths:
      - 'infra/kong/**'

jobs:
  validate:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Install decK
        run: |
          curl -sL https://github.com/Kong/deck/releases/latest/download/deck_linux_amd64.tar.gz | tar xz
          sudo mv deck /usr/local/bin/
      - name: Validate config
        run: deck validate -s infra/kong/kong.yaml

  # NOTE: 各環境の CI/CD ランナーはそれぞれのクラスタ内で動作する。
  # そのため Kong Admin API のサービス名（kong-admin.k1s0-system.svc.cluster.local:8001）は
  # 全環境で同一だが、実際の接続先はランナーが属するクラスタコンテキストによって異なる。
  # dev クラスタのランナー → dev の Kong、staging クラスタのランナー → staging の Kong、
  # prod クラスタのランナー → prod の Kong にそれぞれ接続される。

  diff:
    needs: validate
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Show diff
        run: |
          deck diff -s infra/kong/kong.yaml \
            --kong-addr http://kong-admin.k1s0-system.svc.cluster.local:8001

  sync-dev:
    needs: diff
    runs-on: [self-hosted, dev]
    environment: dev
    steps:
      - uses: actions/checkout@v4
      - name: Sync to dev
        run: |
          # dev クラスタ内のランナーで実行
          deck sync -s infra/kong/kong.yaml \
            --kong-addr http://kong-admin.k1s0-system.svc.cluster.local:8001

  sync-staging:
    needs: sync-dev
    runs-on: [self-hosted, staging]
    environment: staging
    steps:
      - uses: actions/checkout@v4
      - name: Sync to staging
        run: |
          # staging クラスタ内のランナーで実行
          deck sync -s infra/kong/kong.yaml \
            --kong-addr http://kong-admin.k1s0-system.svc.cluster.local:8001

  sync-prod:
    needs: sync-staging
    runs-on: [self-hosted, prod]
    environment:
      name: prod
    steps:
      - uses: actions/checkout@v4
      - name: Sync to prod
        run: |
          # prod クラスタ内のランナーで実行
          deck sync -s infra/kong/kong.yaml \
            --kong-addr http://kong-admin.k1s0-system.svc.cluster.local:8001
```

## PostgreSQL HA 構成詳細

### prod 環境（3ノード構成）

- Primary 1 ノード + Replica 2 ノードの合計 3 ノード構成
- **Bitnami PostgreSQL HA Chart** によるストリーミングレプリケーションとフェイルオーバー管理
  - Primary 障害時に Replica の中から自動的に新しい Primary を選出
  - フェイルオーバー時間目標: 30 秒以内
- **同期レプリケーション**を採用し、データ損失を防止
  - `synchronous_commit = on` により、少なくとも 1 つの Replica への書き込み完了を保証
  - `synchronous_standby_names = 'ANY 1 (*)'` で任意の 1 Replica を同期対象とする
- Kong からの接続は Kubernetes Service 経由でルーティング
- PostgreSQL のデプロイは [terraform設計.md](../../infrastructure/terraform/terraform設計.md) の `modules/database/` で管理する

### staging 環境（2ノード構成）

- Primary 1 ノード + Replica 1 ノードの合計 2 ノード構成
- **非同期レプリケーション**を採用（パフォーマンス優先）
  - `synchronous_commit = off`
- フェイルオーバーのテスト用途を兼ねる

### dev 環境（シングルノード）

- PostgreSQL シングルノード構成
- レプリケーションなし
- 開発・テスト用途のため可用性要件は設けない

## Admin API アクセス制御

環境ごとに異なるアクセス制御を適用し、セキュリティレベルを段階的に強化する。

### dev 環境

- **Basic 認証** + 開発用トークンによるアクセス制御
- 開発者全員がアクセス可能
- 開発用トークンは `kong-admin-dev-token` Secret で管理

### staging 環境

- **IP 制限**: 管理ネットワーク（`10.0.0.0/8` 等の社内ネットワーク）からのアクセスのみ許可
- **mTLS**: クライアント証明書による相互認証を必須とする
  - 運用チーム用のクライアント証明書を発行し、Kong Admin API への接続時に提示
- Kong の `ip-restriction` プラグインと Istio の PeerAuthentication を組み合わせて適用

### prod 環境

- **IP 制限**: 管理ネットワークからのアクセスのみ許可（staging と同様）
- **mTLS**: インフラチームメンバー個人に発行されたクライアント証明書による認証
  - 個人証明書は社内 CA から発行し、有効期限 1 年、失効管理は CRL で実施
  - 証明書の CN にはメンバーの識別子を含め、誰がアクセスしたかを特定可能にする
- **監査ログ記録**: Admin API への全リクエストを監査ログとして記録
  - 記録項目: タイムスタンプ、操作者（証明書 CN）、HTTPメソッド、エンドポイント、リクエストボディ、レスポンスコード
  - ログは Loki に送信し、90 日間保持
  - 設定変更操作（POST / PUT / PATCH / DELETE）は Microsoft Teams の `infra-audit` チャンネルにもリアルタイム通知
