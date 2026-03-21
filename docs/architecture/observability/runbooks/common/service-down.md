# アラート: サービスダウン

対象アラート: `ServiceDown`（ローカル開発環境用）

## 概要

| 項目 | 内容 |
|------|------|
| **重要度** | warning（ローカル開発環境） |
| **影響範囲** | 監視対象のサービスが Prometheus から到達不能 |
| **通知チャネル** | Microsoft Teams #alert-warning |
| **対応 SLA** | SEV2（30分以内） |

## アラート発火条件

- Prometheus の `up` メトリクスが 0（1分継続）
- 対象: `{job}` に登録されたサービスのヘルスチェックエンドポイントが応答しない

## 初動対応（5分以内）

### 1. サービスの状態確認

```bash
# ローカル開発環境（Docker Compose）
docker compose ps

# Kubernetes 環境
kubectl get pods -n {namespace} -l app={service-name}
```

### 2. ヘルスチェックエンドポイントへの疎通確認

```bash
# ローカル環境
curl -sf http://localhost:{port}/healthz || echo "SERVICE DOWN"

# Kubernetes 環境（Pod 内から確認）
kubectl exec -n {namespace} {pod-name} -- curl -sf http://localhost:8080/healthz
```

### 3. Prometheus の scrape 状態確認

Prometheus UI → Status → Targets で対象サービスの `State` を確認:
- `UP` → 疎通は回復済み（アラートは自動解消待ち）
- `DOWN` → まだ到達不能（下記調査へ）

## 詳細調査

### よくある原因

1. **サービスがクラッシュ**: アプリが起動失敗またはクラッシュ
2. **ネットワーク問題**: ServiceMonitor / Prometheus の設定ミス
3. **ポート設定ミス**: `/metrics` エンドポイントのポートが誤っている
4. **リソース不足**: メモリ・CPU 制限でサービスが応答不能

### 調査コマンド

```bash
# ローカル環境でコンテナを再起動
docker compose restart {service-name}
docker compose logs {service-name} --tail=50

# Kubernetes 環境でのイベント確認
kubectl describe pod -n {namespace} -l app={service-name} | tail -20
kubectl get events -n {namespace} | grep {service-name}
```

## 復旧手順

### ローカル開発環境

```bash
# コンテナの再起動
docker compose restart {service-name}

# ログを確認しながら起動待ち
docker compose logs -f {service-name}
```

### Kubernetes 環境

```bash
# Pod の再起動
kubectl rollout restart deployment/{service-name} -n {namespace}

# 起動状態の確認
kubectl rollout status deployment/{service-name} -n {namespace}
```

## エスカレーション基準

以下の条件に該当する場合はエスカレーションする:

- [ ] 再起動後も `up == 0` が継続する
- [ ] auth-server など基盤サービスがダウンしている
- [ ] 複数サービスが同時にダウン

エスカレーション先: [インシデント管理設計](../インシデント管理設計.md)

## 根本原因分析のポイント

- クラッシュした場合のエラーログ・Exit Code を確認
- デプロイや設定変更との相関を調べる
- Prometheus の ServiceMonitor / scrape 設定が正しいか確認

## 関連ドキュメント

- [可観測性設計](../../可観測性設計.md)
- [新規サービス監視設定ガイド](../新規サービス監視設定ガイド.md)
