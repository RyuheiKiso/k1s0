# デプロイメント手順

本ドキュメントは、k1s0 における各サービスのデプロイ手順を定義する。

## 1. 前提条件

### 1.1 必要なツール

| ツール | バージョン | 用途 |
|--------|-----------|------|
| kubectl | 1.28+ | Kubernetes クラスタ操作 |
| kustomize | 5.0+ | マニフェスト生成 |
| helm | 3.12+ | Chart デプロイ（必要に応じて） |
| k1s0 CLI | 最新 | デプロイ自動化 |

### 1.2 クラスタアクセス

```bash
# コンテキスト確認
kubectl config current-context

# 環境別コンテキスト
# dev:  k1s0-dev
# stg:  k1s0-stg
# prod: k1s0-prod
```

## 2. デプロイ手順

### 2.1 標準デプロイ（k1s0 CLI）

```bash
# 開発環境へのデプロイ
k1s0 deploy --env dev --service {service_name}

# ステージング環境へのデプロイ
k1s0 deploy --env stg --service {service_name}

# 本番環境へのデプロイ
k1s0 deploy --env prod --service {service_name}
```

### 2.2 Kustomize を使用したデプロイ

```bash
# マニフェスト生成（dry-run）
kustomize build deploy/overlays/{env} | kubectl apply --dry-run=client -f -

# 実際のデプロイ
kustomize build deploy/overlays/{env} | kubectl apply -f -
```

### 2.3 デプロイ前チェックリスト

- [ ] 対象ブランチが正しいことを確認
- [ ] CI/CD パイプラインが成功していることを確認
- [ ] 設定ファイル（`config/{env}.yaml`）の変更がある場合は事前レビュー済み
- [ ] DB マイグレーションが必要な場合は先に実行済み
- [ ] 依存サービスが正常稼働していることを確認

## 3. 環境変数設定

### 3.1 設定の優先順位

1. Kubernetes Secret（機密情報）
2. ConfigMap（`config/{env}.yaml` から生成）
3. デフォルト値（`config/default.yaml`）

### 3.2 機密情報の設定

機密情報は必ず Secret 経由で設定する。

```yaml
# deploy/overlays/{env}/secrets.yaml（暗号化済み）
apiVersion: v1
kind: Secret
metadata:
  name: {service_name}-secrets
type: Opaque
stringData:
  DATABASE_PASSWORD_FILE: /secrets/db-password
  API_KEY_FILE: /secrets/api-key
```

### 3.3 設定の確認

```bash
# ConfigMap の内容確認
kubectl get configmap {service_name}-config -o yaml

# Secret の存在確認（値は表示しない）
kubectl get secret {service_name}-secrets -o jsonpath='{.data}' | jq 'keys'
```

## 4. ヘルスチェック確認

### 4.1 Pod の状態確認

```bash
# Pod 一覧と状態
kubectl get pods -l app={service_name}

# Pod の詳細（イベント含む）
kubectl describe pod -l app={service_name}

# ログ確認（直近 100 行）
kubectl logs -l app={service_name} --tail=100
```

### 4.2 HTTP ヘルスチェック

```bash
# ポートフォワード
kubectl port-forward svc/{service_name} 8080:80

# ヘルスチェック実行
curl http://localhost:8080/healthz
```

期待されるレスポンス:
```json
{
  "status": "healthy",
  "checks": {
    "db": "ok",
    "redis": "ok",
    "config": "ok"
  }
}
```

### 4.3 gRPC ヘルスチェック

```bash
# grpcurl を使用
grpcurl -plaintext localhost:50051 grpc.health.v1.Health/Check
```

期待されるレスポンス:
```json
{
  "status": "SERVING"
}
```

### 4.4 デプロイ後チェックリスト

- [ ] 全 Pod が Running 状態
- [ ] ヘルスチェックエンドポイントが 200 を返す
- [ ] ログにエラーが出ていない
- [ ] メトリクスが Prometheus に送信されている
- [ ] トレースが Collector に送信されている

## 5. ロールバック手順

### 5.1 即座のロールバック（Deployment）

```bash
# 直前のリビジョンへロールバック
kubectl rollout undo deployment/{service_name}

# 特定のリビジョンへロールバック
kubectl rollout history deployment/{service_name}
kubectl rollout undo deployment/{service_name} --to-revision={revision}
```

### 5.2 k1s0 CLI でのロールバック

```bash
# 直前バージョンへロールバック
k1s0 rollback --env {env} --service {service_name}

# 特定バージョンへロールバック
k1s0 rollback --env {env} --service {service_name} --version {version}
```

### 5.3 ロールバック判断基準

以下の場合は即座にロールバックを検討:

| 状況 | アクション |
|------|-----------|
| Pod が CrashLoopBackOff | 即ロールバック |
| ヘルスチェック失敗が 50% 以上 | 即ロールバック |
| エラーレート急増（5分間で 10% 以上） | 即ロールバック |
| レイテンシ急増（p99 が 2 倍以上） | 状況確認後判断 |

### 5.4 ロールバック後チェックリスト

- [ ] 全 Pod が Running 状態に復帰
- [ ] ヘルスチェックが正常
- [ ] エラーレートが正常値に復帰
- [ ] ロールバック理由をインシデントとして記録

## 6. カナリアデプロイ

### 6.1 カナリアリリース手順

```bash
# カナリア Pod のデプロイ（10% トラフィック）
k1s0 deploy --env prod --service {service_name} --canary --weight 10

# メトリクス確認（15分以上観察）
# - エラーレート
# - レイテンシ
# - 成功率

# 問題なければ段階的に増加
k1s0 deploy --env prod --service {service_name} --canary --weight 50
k1s0 deploy --env prod --service {service_name} --canary --weight 100

# カナリア終了（全トラフィック切り替え）
k1s0 deploy --env prod --service {service_name} --promote
```

### 6.2 カナリア中止

```bash
# カナリアを中止し、旧バージョンに戻す
k1s0 deploy --env prod --service {service_name} --abort-canary
```

## 7. デプロイ時の注意事項

### 7.1 本番デプロイ時

- 平日日中（10:00-17:00）に実施を推奨
- 金曜日のデプロイは避ける
- 大規模リリースは事前に関係者へ周知
- デプロイ担当者は Slack #ops チャンネルで開始・終了を報告

### 7.2 緊急デプロイ時

- インシデント対応の場合のみ許可
- 2 名以上での確認を推奨
- デプロイ後は必ずインシデントレポートを作成

## 関連ドキュメント

- [サービス構成規約](../conventions/service-structure.md)
- [設定と秘密情報の規約](../conventions/config-and-secrets.md)
- [トラブルシューティング](troubleshooting.md)
- [インシデント対応](runbooks/incident-response.md)
