# E2E テスト戦略

F-010: E2E（End-to-End）テストの範囲、テスト環境構成、テストデータ管理、CI/CD パイプライン統合を定義する。

---

## 基本方針

- E2E テストは **重要なユーザーフロー** に限定し、テストピラミッドの頂点として最小限に保つ
- テスト環境は本番と同一構成の Kubernetes クラスタ上に構築する
- テストデータは各テスト実行前にシードし、実行後にクリーンアップする
- E2E テストは main マージ後に実行し、PR 時には実行しない（実行コスト考慮）

---

## E2E テスト範囲と対象

### テスト対象の選定基準

E2E テストは以下の基準に基づいて対象を選定する。

| 基準 | 説明 |
| --- | --- |
| ビジネスクリティカル | 障害発生時にビジネスへの影響が大きいフロー |
| クロスサービス | 複数サービスを横断するフロー（単体・統合テストで担保しきれない部分） |
| ユーザー認証フロー | OAuth 2.0 / OIDC を含む認証・認可フロー |
| データ整合性 | 複数サービスの DB にまたがるデータの一貫性を検証するフロー |

### テスト対象フロー一覧

| フロー | 関連サービス | Tier | 優先度 |
| --- | --- | --- | --- |
| ユーザー認証（ログイン → トークン取得 → API 呼び出し） | auth-server, bff-proxy, Keycloak | system | P0 |
| 設定値の取得と配信 | config-server, 各サービス | system | P0 |
| Saga による分散トランザクション | saga-server, 各ドメインサービス | system → service | P0 |
| DLQ メッセージの再処理 | dlq-manager, Kafka | system | P1 |
| API ゲートウェイ経由のリクエストルーティング | bff-proxy, Kong, 各サービス | system → service | P1 |
| アプリ配布（アップロード → レジストリ登録 → ダウンロード） | app-registry, Ceph RGW | system | P2 |

### テスト対象外

| 対象外 | 理由 |
| --- | --- |
| UI の見た目・レイアウト | ビジュアルリグレッションテストは別途検討する |
| 個別サービスのビジネスロジック | 単体テスト・統合テストで担保する |
| インフラの可用性（ノード障害等） | カオスエンジニアリングの範疇とする |

---

## テスト環境構成

### 環境構成

E2E テスト専用の Kubernetes 環境を用意する。本番構成と同一の Namespace 構成・NetworkPolicy を適用する。

```
┌─────────────────────────────────────────────────┐
│              E2E テスト環境 (Kubernetes)          │
│                                                  │
│  ┌──────────────┐  ┌──────────────┐             │
│  │ k1s0-system  │  │ k1s0-service │             │
│  │              │  │              │             │
│  │ auth-server  │  │ order        │             │
│  │ config-server│  │ inventory    │             │
│  │ saga-server  │  │ payment      │             │
│  │ dlq-manager  │  │              │             │
│  │ bff-proxy    │  │              │             │
│  └──────┬───────┘  └──────┬───────┘             │
│         │                  │                     │
│  ┌──────┴──────────────────┴───────┐             │
│  │          共通インフラ            │             │
│  │ PostgreSQL / Kafka / Redis      │             │
│  │ Keycloak / Vault (dev mode)     │             │
│  └─────────────────────────────────┘             │
└─────────────────────────────────────────────────┘
         ▲
         │ テストランナーからリクエスト
┌────────┴────────┐
│   E2E テスト    │
│   ランナー      │
│   (GitHub Actions)│
└─────────────────┘
```

### 環境の差異

| 項目 | 本番環境 | E2E テスト環境 |
| --- | --- | --- |
| Vault モード | HA クラスタ | dev モード（自動 Unseal） |
| DB | マネージド PostgreSQL | Pod 内 PostgreSQL |
| Kafka | 3 ブローカークラスタ | 1 ブローカー（KRaft） |
| レプリカ数 | HPA 制御 | 各サービス 1 レプリカ |
| TLS | Vault PKI + Istio mTLS | 無効（テスト簡略化） |

### 環境の起動と破棄

E2E テスト環境はテスト実行ごとに作成・破棄する（エフェメラル環境）。

```bash
# 環境の作成（Helm Chart を使用）
helm upgrade --install e2e-env infra/helm/environments/e2e \
  --namespace e2e-test \
  --create-namespace \
  --wait --timeout 10m

# テスト実行後の環境破棄
helm uninstall e2e-env --namespace e2e-test
kubectl delete namespace e2e-test
```

---

## テストデータ管理

### テストデータのライフサイクル

```
テスト開始
    │
    ├─► 1. DB マイグレーション実行（スキーマ作成）
    ├─► 2. シードデータ投入（テスト用初期データ）
    ├─► 3. Keycloak テストユーザー作成
    │
    ├─► テストケース実行
    │
    └─► 4. 環境破棄（Namespace 削除で全データクリーンアップ）
```

### シードデータ

E2E テスト用のシードデータは SQL ファイルとしてリポジトリ内に管理する。

```
tests/e2e/
├── fixtures/
│   ├── seed-auth.sql          # 認証テスト用シードデータ
│   ├── seed-config.sql        # 設定テスト用シードデータ
│   ├── seed-saga.sql          # Saga テスト用シードデータ
│   └── keycloak-setup.json    # Keycloak テストユーザー・クライアント
├── scenarios/
│   ├── auth-flow.rs           # 認証フローテスト
│   ├── config-distribution.rs # 設定配信テスト
│   └── saga-transaction.rs    # Saga 分散トランザクションテスト
└── helpers/
    ├── api_client.rs          # テスト用 API クライアント
    ├── kafka_helper.rs        # Kafka メッセージ検証ヘルパー
    └── db_helper.rs           # DB 状態検証ヘルパー
```

### テストデータの原則

1. **冪等性**: シードスクリプトは何度実行しても同じ結果になる（`INSERT ... ON CONFLICT DO NOTHING`）
2. **独立性**: テストケース間でデータを共有しない（各テストが必要なデータを自身でセットアップする）
3. **トレーサビリティ**: テストデータには `e2e-test-` プレフィックスを付与し、本番データと区別する

---

## テストシナリオ構成

### テストシナリオの記述

E2E テストは Rust で記述し、HTTP / gRPC クライアントを使用してサービスにリクエストを送信する。

```rust
// tests/e2e/scenarios/auth_flow.rs

/// 認証フロー E2E テスト: ログインからAPI呼び出しまでの一連のフローを検証する
#[tokio::test]
async fn test_auth_flow_login_to_api_call() {
    // 1. Keycloak でトークンを取得する
    let token = keycloak_client
        .get_token("e2e-test-user", "e2e-test-password")
        .await
        .expect("トークン取得に失敗");

    // 2. 取得したトークンで BFF 経由の API を呼び出す
    let response = api_client
        .get("/api/v1/config")
        .bearer_auth(&token.access_token)
        .send()
        .await
        .expect("API 呼び出しに失敗");

    // 3. レスポンスを検証する
    assert_eq!(response.status(), 200);
    let body: ConfigResponse = response.json().await.unwrap();
    assert!(!body.items.is_empty(), "設定項目が空であってはならない");
}
```

### リトライとタイムアウト

E2E テストでは非同期処理の完了を待つためのリトライロジックを使用する。

```rust
/// 条件が満たされるまでリトライする（最大 30 秒、3 秒間隔）
async fn wait_for_condition<F, Fut>(description: &str, check: F) -> bool
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = bool>,
{
    let max_attempts = 10;
    let interval = std::time::Duration::from_secs(3);

    for attempt in 1..=max_attempts {
        if check().await {
            return true;
        }
        tracing::info!("{}: attempt {}/{}", description, attempt, max_attempts);
        tokio::time::sleep(interval).await;
    }
    false
}
```

---

## CI/CD パイプライン統合

### 実行タイミング

| トリガー | 実行内容 | 目的 |
| --- | --- | --- |
| main マージ後 | 全 E2E テストシナリオ | リリース候補の品質ゲート |
| 日次スケジュール（深夜） | 全 E2E テストシナリオ | 環境劣化の早期検知 |
| 手動トリガー | 指定シナリオのみ | デバッグ・特定フローの検証 |

### GitHub Actions ワークフロー

```yaml
# .github/workflows/e2e-test.yaml
name: E2E Test

on:
  push:
    branches: [main]
  schedule:
    - cron: '0 18 * * *'   # JST 03:00（毎日深夜）
  workflow_dispatch:
    inputs:
      scenario:
        description: '実行するシナリオ（空の場合は全シナリオ）'
        required: false

jobs:
  e2e-test:
    runs-on: ubuntu-latest
    timeout-minutes: 60
    steps:
      - uses: actions/checkout@v4

      # E2E テスト環境のセットアップ
      - name: Setup E2E environment
        run: |
          helm upgrade --install e2e-env infra/helm/environments/e2e \
            --namespace e2e-test \
            --create-namespace \
            --wait --timeout 10m

      # シードデータの投入
      - name: Seed test data
        run: |
          kubectl exec -n e2e-test deploy/postgres -- \
            psql -U postgres -f /seed/seed-auth.sql
          kubectl exec -n e2e-test deploy/postgres -- \
            psql -U postgres -f /seed/seed-config.sql

      # E2E テストの実行
      - name: Run E2E tests
        run: |
          cargo test --manifest-path tests/e2e/Cargo.toml \
            -- ${{ github.event.inputs.scenario }}

      # テスト結果のアーティファクト保存
      - name: Upload test results
        if: always()
        uses: actions/upload-artifact@v4
        with:
          name: e2e-test-results
          path: tests/e2e/results/

      # 環境のクリーンアップ
      - name: Cleanup
        if: always()
        run: |
          helm uninstall e2e-env --namespace e2e-test || true
          kubectl delete namespace e2e-test || true
```

### テスト結果の通知

| 結果 | 通知先 | 通知方法 |
| --- | --- | --- |
| 全テスト成功 | --- | 通知なし（正常） |
| テスト失敗 | Microsoft Teams | Alertmanager 経由 |
| 環境セットアップ失敗 | Microsoft Teams + インフラチーム | Alertmanager 経由 |

### 失敗時のデバッグ

E2E テスト失敗時は以下の情報をアーティファクトとして保存する。

- テストランナーのログ（標準出力・標準エラー）
- 各サービスの Pod ログ（`kubectl logs`）
- Kafka トピックの未消費メッセージ数
- DB のテストデータ状態（スナップショット）

---

## Playwright バージョン管理ポリシー

`tests/e2e/package.json` の `@playwright/test` はキャレット（`^`）を使用しない固定バージョンを指定する。

### 理由

Playwright はブラウザバイナリとセットでバージョン管理されており、マイナーアップデートでも API の破壊的変更や非推奨化が発生することがある。E2E テストは CI の品質ゲートとして機能するため、予期しないバージョン変動による偽陽性・偽陰性を防止する必要がある。

### ポリシー

| 項目 | 方針 |
| --- | --- |
| バージョン指定 | 固定バージョン（`1.49.0` 形式。`^1.49.0` は使用しない） |
| アップグレード | 手動で PR を作成し、変更内容を確認してからマージする |
| package-lock.json | `npm ci` で再現性を確保。lock ファイルは必ずコミットする |

### アップグレード手順

```bash
# バージョンを更新する（package.json を手動編集後）
cd tests/e2e
npm install
# package-lock.json が更新されることを確認する
git add package.json package-lock.json
```

## 関連ドキュメント

- [テスト戦略](./test-strategy.md) -- テストピラミッド・言語別フレームワーク・カバレッジ目標
- [パフォーマンステスト戦略](./performance-strategy.md) -- 負荷テスト・ベンチマーク
- [CI-CD設計.md](../../infrastructure/cicd/CI-CD設計.md) -- CI/CD パイプライン設計
- [kubernetes設計.md](../../infrastructure/kubernetes/kubernetes設計.md) -- Namespace・NetworkPolicy 設計
- [カオスエンジニアリング設計.md](../chaos-engineering/カオスエンジニアリング設計.md) -- 耐障害性テスト
