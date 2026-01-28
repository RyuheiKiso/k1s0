# サービス再起動手順

本ドキュメントは、k1s0 サービスの再起動手順を定義する。

## 1. 概要

サービス再起動が必要となるケース:

- メモリリークの一時対処
- 設定変更の反映（ConfigMap/Secret 更新後）
- 不安定状態からの復旧
- 接続プール枯渇の解消

## 2. 事前確認

### 2.1 チェックリスト

- [ ] 再起動の理由を明確にする
- [ ] 現在のサービス状態を確認
- [ ] 他チームへの影響を確認
- [ ] 本番環境の場合、承認を得る

### 2.2 現状確認コマンド

```bash
# Pod の状態確認
kubectl get pods -l app={service_name} -o wide

# レプリカ数確認
kubectl get deployment {service_name} -o jsonpath='{.spec.replicas}'

# 現在のトラフィック確認（メトリクス）
curl 'http://prometheus:9090/api/v1/query?query=sum(rate(k1s0_{service}_request_total[1m]))'
```

## 3. 再起動手順

### 3.1 ローリング再起動（推奨）

サービス無停止で全 Pod を順次再起動する。

```bash
# ローリング再起動
kubectl rollout restart deployment/{service_name}

# 進行状況確認
kubectl rollout status deployment/{service_name}
```

**所要時間:** 約 2-5 分（Pod 数とヘルスチェック設定に依存）

### 3.2 単一 Pod の再起動

特定の Pod のみ再起動する場合。

```bash
# Pod 名を確認
kubectl get pods -l app={service_name}

# 特定 Pod を削除（ReplicaSet が自動再作成）
kubectl delete pod {pod_name}

# 新しい Pod の起動確認
kubectl get pods -l app={service_name} -w
```

### 3.3 強制再起動（緊急時のみ）

全 Pod を同時に再起動する。**サービス停止が発生する。**

```bash
# 全 Pod を一度に削除
kubectl delete pods -l app={service_name}

# または replicas を 0 にして戻す
kubectl scale deployment/{service_name} --replicas=0
kubectl scale deployment/{service_name} --replicas={desired_replicas}
```

## 4. 再起動後の確認

### 4.1 確認チェックリスト

- [ ] 全 Pod が Running 状態
- [ ] ヘルスチェックが正常（Ready 状態）
- [ ] ログにエラーがない
- [ ] メトリクスが送信されている
- [ ] トラフィックが正常に処理されている

### 4.2 確認コマンド

```bash
# Pod 状態確認
kubectl get pods -l app={service_name}

# ログ確認（直近 50 行）
kubectl logs -l app={service_name} --tail=50

# ヘルスチェック
kubectl exec -it {pod_name} -- curl http://localhost:8080/healthz

# イベント確認
kubectl get events --field-selector involvedObject.name={deployment_name} --sort-by='.lastTimestamp'
```

### 4.3 メトリクス確認

```bash
# エラーレートの確認
curl 'http://prometheus:9090/api/v1/query?query=sum(rate(k1s0_{service}_request_failures_total[5m]))/sum(rate(k1s0_{service}_request_total[5m]))'

# レイテンシの確認
curl 'http://prometheus:9090/api/v1/query?query=histogram_quantile(0.99,sum(rate(k1s0_{service}_request_duration_seconds_bucket[5m]))by(le))'
```

## 5. トラブルシューティング

### 5.1 再起動後も問題が継続

| 症状 | 対処 |
|------|------|
| Pod が起動しない | ログを確認し、[トラブルシューティング](../troubleshooting.md) を参照 |
| ヘルスチェック失敗 | 依存サービスの状態を確認 |
| 同じエラーが再発 | 根本原因の調査が必要 |

### 5.2 ロールバックが必要な場合

再起動後に問題が発生した場合:

```bash
# 直前のリビジョンへロールバック
kubectl rollout undo deployment/{service_name}

# ロールバック確認
kubectl rollout status deployment/{service_name}
```

## 6. 本番環境での再起動

### 6.1 追加チェックリスト

- [ ] #ops チャンネルで再起動開始を通知
- [ ] 2 名以上で実施（1 名が実行、1 名が確認）
- [ ] ダッシュボードを常時監視
- [ ] ロールバック準備

### 6.2 通知テンプレート

**開始時:**
```
[再起動開始] {service_name}
環境: prod
理由: {理由}
担当: {担当者名}
予定所要時間: 約5分
```

**完了時:**
```
[再起動完了] {service_name}
環境: prod
結果: 正常完了
確認項目: Pod状態OK, ヘルスチェックOK, エラーレート正常
```

## 7. 自動化スクリプト

### 7.1 PowerShell（Windows）

```powershell
# scripts/restart-service.ps1
param(
    [Parameter(Mandatory=$true)]
    [string]$ServiceName,

    [Parameter(Mandatory=$true)]
    [ValidateSet("dev", "stg", "prod")]
    [string]$Env
)

Write-Host "Restarting $ServiceName in $Env..."

# コンテキスト切り替え
kubectl config use-context "k1s0-$Env"

# ローリング再起動
kubectl rollout restart deployment/$ServiceName

# 状態確認
kubectl rollout status deployment/$ServiceName --timeout=300s

if ($LASTEXITCODE -eq 0) {
    Write-Host "Restart completed successfully."
} else {
    Write-Host "Restart failed. Consider rollback."
    exit 1
}
```

### 7.2 k1s0 CLI

```bash
# k1s0 CLI での再起動
k1s0 service restart --env {env} --service {service_name}

# 状態確認
k1s0 service status --env {env} --service {service_name}
```

## 関連ドキュメント

- [トラブルシューティング](../troubleshooting.md)
- [デプロイメント手順](../deployment.md)
- [インシデント対応](incident-response.md)
