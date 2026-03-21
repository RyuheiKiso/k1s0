# アラート: Pod 再起動頻発

対象アラート: `auth_server_pod_restart`, `config_server_pod_restart`,
および CLI テンプレート生成の `{ServiceName}PodRestart`

## 概要

| 項目 | 内容 |
|------|------|
| **重要度** | critical（auth/config）/ warning（その他） |
| **影響範囲** | 対象サービスの可用性（再起動中はリクエスト失敗） |
| **通知チャネル** | Microsoft Teams #alert-critical / #alert-warning |
| **対応 SLA** | critical: SEV1（15分以内） / warning: SEV2（30分以内） |

## アラート発火条件

- 1時間以内に Pod が 3回以上再起動

## 初動対応（5分以内）

### 1. Pod の状態確認

```bash
# 再起動回数の確認
kubectl get pods -n {namespace} | grep {service-name}
# RESTARTS 列の数値を確認

# 詳細な再起動情報
kubectl describe pod -n {namespace} {pod-name} | grep -A 10 "Last State\|Reason\|Exit Code"
```

### 2. クラッシュループかどうかの判断

```bash
# CrashLoopBackOff 状態かどうか確認
kubectl get pods -n {namespace} -l app={service-name}
# STATUS が "CrashLoopBackOff" → SEV1（即時対応）
# STATUS が "Running" だが RESTARTS が多い → SEV2（詳細調査）
```

### 3. 直近ログの確認

```bash
# 直前のコンテナのログ（再起動前のクラッシュ原因）
kubectl logs -n {namespace} {pod-name} --previous

# 現在のコンテナのログ
kubectl logs -n {namespace} {pod-name} --tail=100
```

## 詳細調査

### よくある原因

1. **OOM Kill（メモリ不足）**: コンテナがメモリ制限を超えてカーネルに Kill される
2. **Liveness Probe 失敗**: ヘルスチェックエンドポイントがタイムアウト
3. **起動時エラー**: 設定ファイル・シークレット・DB 接続の失敗
4. **パニック / 未処理例外**: アプリケーションのバグによるクラッシュ

### 調査コマンド

```bash
# OOM Kill の確認
kubectl describe pod -n {namespace} {pod-name} | grep -i "oom\|killed\|exit code 137"

# Liveness Probe の設定確認
kubectl get pod -n {namespace} {pod-name} -o yaml | grep -A 10 livenessProbe

# イベント確認（再起動の原因）
kubectl get events -n {namespace} --sort-by='.lastTimestamp' | grep {pod-name}

# リソース使用量の推移（Prometheus）
# container_memory_working_set_bytes{namespace="{namespace}", pod=~"{service-name}-.*"}
```

### Prometheus クエリ例

```promql
# Pod 再起動数の推移
increase(kube_pod_container_status_restarts_total{namespace="{namespace}", pod=~"{service-name}-.*"}[1h])

# メモリ使用量と制限の比較
container_memory_working_set_bytes{namespace="{namespace}", pod=~"{service-name}-.*"}
/ container_spec_memory_limit_bytes{namespace="{namespace}", pod=~"{service-name}-.*"}
```

## 復旧手順

### パターン A: OOM Kill の場合

```bash
# 一時的なメモリ制限増加
kubectl patch deployment {service-name} -n {namespace} \
  --patch '{"spec":{"template":{"spec":{"containers":[{"name":"{service-name}","resources":{"limits":{"memory":"512Mi"}}}]}}}}'

# 根本対応: Helm values のメモリ制限を見直してリデプロイ
```

### パターン B: 設定・シークレットの問題

```bash
# シークレットの存在確認
kubectl get secret -n {namespace} | grep {service-name}

# シークレットの値確認（base64 デコード）
kubectl get secret -n {namespace} {secret-name} -o jsonpath='{.data.{key}}' | base64 -d
```

### パターン C: バグによるクラッシュ

```bash
# 直前のリリースにロールバック
kubectl rollout undo deployment/{service-name} -n {namespace}
kubectl rollout status deployment/{service-name} -n {namespace}
```

## エスカレーション基準

以下の条件に該当する場合はエスカレーションする:

- [ ] CrashLoopBackOff 状態で10分以上回復しない
- [ ] ロールバック後も再起動が継続
- [ ] auth-server / config-server が影響を受けており認証基盤全体が不安定
- [ ] 原因が不明で対処方法が分からない

エスカレーション先: [インシデント管理設計](../インシデント管理設計.md)

## 根本原因分析のポイント

- Exit Code による原因分類（137=OOM, 1=アプリエラー, 2=設定エラー）
- デプロイや設定変更との相関
- メモリリークの有無（長期間での使用量推移を Prometheus で確認）

## 関連ドキュメント

- [可観測性設計](../../可観測性設計.md)
- [ログ設計](../../ログ設計.md)
