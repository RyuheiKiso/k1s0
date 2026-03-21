# レートリミット戦略 — 単一ソースオブトゥルース（SoT）

k1s0 プロジェクトでは、Kong API Gateway と Istio サービスメッシュの両方にレートリミット機能が存在する。
本ドキュメントでは、どちらを単一の管理主体（SoT: Source of Truth）とするかを決定し、
その方針・設定箇所・責任境界を明文化する。

---

## 結論: Kong が SoT

**レートリミットの単一ソースオブトゥルースは Kong API Gateway とする。**

Istio VirtualService にはレートリミット設定を持たせない。
Istio のトラフィック管理機能（タイムアウト・リトライ・サーキットブレーカー等）とは責任を分離する。

---

## 役割分担

| 機能 | 担当コンポーネント | 管理ファイル |
|------|-------------------|-------------|
| レートリミット（外部クライアント → API） | **Kong** | `infra/helm/services/system/kong/values-{env}.yaml` |
| タイムアウト制御 | Istio VirtualService | `infra/istio/virtual-service.yaml` |
| リトライ制御 | Istio VirtualService | `infra/istio/virtual-service.yaml` |
| サーキットブレーカー | Istio DestinationRule | `infra/istio/destinationrules/` |
| mTLS・認可ポリシー | Istio PeerAuthentication / AuthorizationPolicy | `infra/istio/peerauthentication.yaml` / `infra/istio/authorizationpolicy.yaml` |

---

## Kong レートリミット設定

### 設定方針

- **Tier 別デフォルト値** を環境ファイルで定義する（`values-prod.yaml` を基準値とする）
- **環境別倍率** を乗算することで dev / staging の緩和を実現する
- **Redis バックエンド** を使用し、Kong のマルチレプリカ間でカウンターを共有する

### Tier 別デフォルト値（production 基準）

| Tier | per minute | per second | 用途 |
|------|-----------|-----------|------|
| system | 3,000 | 100 | 認証・設定など基盤サービス |
| business | 1,000 | 40 | 業務ロジックサービス |
| service | 500 | 20 | 個別機能サービス |

### 環境別倍率

| 環境 | 倍率 | per minute (system 例) |
|------|------|----------------------|
| prod | x1 | 3,000 |
| staging | x2 | 6,000 |
| dev | x10 | 30,000 |

### 設定ファイル

```
infra/helm/services/system/kong/
├── values.yaml           # 基本設定（レートリミット無し、ベースのみ）
├── values-prod.yaml      # SoT: 本番レートリミット設定（multiplier=1）
├── values-staging.yaml   # ステージング設定（multiplier=2）
└── values-dev.yaml       # 開発設定（multiplier=10）
```

---

## Istio VirtualService との関係

Istio VirtualService には **レートリミット設定を追加しない**。

### 理由

1. **重複管理の排除** — Kong と Istio の両方にレートリミットを設定すると、
   どちらの値が有効かが不明確になり、設定変更時の漏れや不整合が生じる。

2. **責任の明確化** — Kong は外部クライアントからのリクエストを受ける API Gateway として
   「誰が・何回リクエストできるか」を制御する。Istio はサービスメッシュ内の通信品質
   （タイムアウト・リトライ・フォールト回復）を制御する。

3. **Kong の機能優位性** — Kong の Rate Limiting プラグインは Redis 連携による
   分散カウンター管理・クライアント識別（Consumer 単位）・超過時のレスポンスヘッダー
   付与などの機能を提供しており、Istio の実装より高機能。

### Istio に残す設定（レートリミット以外）

```yaml
# infra/istio/virtual-service.yaml
# NOTE: レートリミットはKongで管理するため、ここには設定しない。
# Tier別のタイムアウトとリトライのみ管理する（サービスメッシュ設計.md 参照）
spec:
  http:
    - timeout: 5s       # system tier デフォルト
      retries:
        attempts: 3
        perTryTimeout: 2s
```

---

## 設定変更手順

### レートリミット値を変更する場合

1. `infra/helm/services/system/kong/values-prod.yaml` の `rateLimiting.tiers` を更新する
2. `staging` / `dev` の倍率（`multiplier`）は原則変更しない
3. Helm Chart をデプロイしてKong に反映する（Kong は設定を DB に保存するため再起動不要）

### 禁止事項

- Istio VirtualService へのレートリミット関連フィールドの追加
- `values-prod.yaml` の `rateLimiting` セクションを Kong 管理外（Nginx Ingress 等）に移動すること
- Kong プラグイン設定を `values.yaml` の `rateLimiting` セクションと二重管理すること

---

## 関連ドキュメント

- `infra/helm/services/system/kong/values-prod.yaml` — Kong レートリミット SoT ファイル
- `infra/istio/virtual-service.yaml` — Istio タイムアウト・リトライ設定
- [docs/infrastructure/service-mesh/](service-mesh/) — Istio サービスメッシュ設計
- [docs/architecture/adr/](../architecture/adr/) — 関連 ADR
