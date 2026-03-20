# gRPC セキュリティ方針

## 概要

k1s0 の gRPC 通信は、Istio サービスメッシュの mTLS を一次防御として利用する。
アプリケーションレベルの TLS（tonic `ServerTlsConfig`）は補助的な手段として位置づける。

## トランスポートセキュリティ

### Istio mTLS（一次防御）

| 項目                | 設定値                                   |
| ------------------- | ---------------------------------------- |
| PeerAuthentication  | `STRICT` モード（平文通信を拒否）        |
| 証明書管理          | Istio CA による自動ローテーション        |
| 対象                | メッシュ内の全 Pod 間通信                |

Istio の `PeerAuthentication` ポリシーにより、k1s0 名前空間内の全 gRPC 通信は
自動的に mTLS で暗号化される。証明書のプロビジョニングとローテーションは Istio が管理するため、
アプリケーション側での証明書管理は不要。

### アプリケーションレベル TLS（補助防御）

Istio を使用しない環境や、メッシュ外からの直接 gRPC アクセスが必要な場合は、
tonic の `ServerTlsConfig` を有効化する。

証明書は Vault PKI エンジンから発行し、init-container で取得する:

```yaml
# Kubernetes Pod spec の例
initContainers:
  - name: vault-cert
    image: vault:latest
    command:
      - vault
      - write
      - -format=json
      - pki_int/issue/auth-server  # サーバー固有の PKI ロール
      - common_name=auth-server.k1s0-system.svc.cluster.local
      - ttl=24h
```

### 認可

gRPC メタデータ（ヘッダー）による認可は各サービスの interceptor で実装する。
Kong API Gateway 経由のリクエストには `X-User-Id` / `X-User-Roles` ヘッダーが付与される。

## 環境別設定

| 環境       | トランスポートセキュリティ | 備考                              |
| ---------- | -------------------------- | --------------------------------- |
| production | Istio mTLS (STRICT)        | アプリ TLS は冗長のため無効       |
| staging    | Istio mTLS (STRICT)        | production と同一構成             |
| dev/docker | 平文（TLS なし）           | ローカル開発用、Istio なし        |
