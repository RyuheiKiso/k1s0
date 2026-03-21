# アラート: TLS 証明書期限切れ

対象アラート: `CertificateExpiringSoon`, `CertificateExpiringCritical`,
`CertManagerCertExpiringSoon`, `CertManagerCertExpiringCritical`, `CertManagerCertNotReady`

## 概要

| 項目 | 内容 |
|------|------|
| **重要度** | critical（7日以内） / warning（30日以内） |
| **影響範囲** | HTTPS/TLS 通信全般（期限切れになると通信不能） |
| **通知チャネル** | Microsoft Teams #alert-critical / #alert-warning |
| **対応 SLA** | critical: SEV1（15分以内） / warning: SEV3（翌営業日） |

## アラート発火条件

| アラート名 | 条件 | 重要度 |
|-----------|------|--------|
| CertificateExpiringSoon | Blackbox Exporter 経由: 30日以内に期限切れ | warning |
| CertificateExpiringCritical | Blackbox Exporter 経由: 7日以内に期限切れ | critical |
| CertManagerCertExpiringSoon | cert-manager 管理: 30日（2592000秒）以内 | warning |
| CertManagerCertExpiringCritical | cert-manager 管理: 7日（604800秒）以内 | critical |
| CertManagerCertNotReady | cert-manager 証明書が Ready でない | warning |

## 初動対応（5分以内）

### 1. 証明書の状態確認

```bash
# cert-manager 管理の証明書一覧
kubectl get certificate -A

# 期限切れが近い証明書の詳細
kubectl describe certificate {cert-name} -n {namespace}

# 証明書の有効期限確認
kubectl get certificate {cert-name} -n {namespace} \
  -o jsonpath='{.status.notAfter}'
```

### 2. 証明書の種別を確認

- [ ] cert-manager 管理 (`CertManagerCert*`) → 自動更新が機能していない可能性
- [ ] 外部証明書 (`Certificate*`) → 手動更新が必要な可能性

### 3. 緊急度の判断

- [ ] 7日以内に期限切れ → SEV1（即時更新が必要）
- [ ] 30日以内に期限切れかつ自動更新設定あり → SEV3（自動更新を確認）

## 詳細調査

### cert-manager の自動更新確認

```bash
# cert-manager の Pod が正常に動作しているか確認
kubectl get pods -n cert-manager

# CertificateRequest の状態確認（更新処理中かどうか）
kubectl get certificaterequest -n {namespace}

# cert-manager のログでエラー確認
kubectl logs -n cert-manager deploy/cert-manager --tail=100 | grep -i "error\|{cert-name}"

# Certificate のイベント確認
kubectl describe certificate {cert-name} -n {namespace} | grep -A 20 Events
```

### よくある原因

1. **Let's Encrypt のレート制限**: 短期間での更新リクエストが多すぎる
2. **DNS 検証の失敗**: ACME challenge の DNS レコード設定に問題
3. **cert-manager のバグ/クラッシュ**: cert-manager 自体が正常動作していない
4. **Issuer の設定ミス**: ClusterIssuer / Issuer の ACME 設定が誤っている

## 復旧手順

### パターン A: cert-manager の自動更新が機能していない場合

```bash
# cert-manager を再起動
kubectl rollout restart deployment/cert-manager -n cert-manager
kubectl rollout restart deployment/cert-manager-webhook -n cert-manager

# 証明書の手動更新トリガー（cert-manager v1.x）
kubectl annotate certificate {cert-name} -n {namespace} \
  cert-manager.io/issuer-kind=ClusterIssuer --overwrite

# cmctl ツールで手動更新（cmctl がインストール済みの場合）
cmctl renew {cert-name} -n {namespace}
```

### パターン B: 手動証明書の期限切れ（外部 CA の場合）

1. 証明書発行機関から新しい証明書を取得
2. Kubernetes Secret を更新:

```bash
kubectl create secret tls {secret-name} -n {namespace} \
  --cert=new-cert.pem --key=new-key.pem \
  --dry-run=client -o yaml | kubectl apply -f -
```

3. 証明書を使用しているリソース（Ingress / Gateway）を確認して再適用

### パターン C: CertManagerCertNotReady の場合

```bash
# Certificate の状態と Conditions を確認
kubectl get certificate {cert-name} -n {namespace} -o yaml | grep -A 20 status

# ACME challenge の状態確認
kubectl get challenge -n {namespace}
kubectl describe challenge -n {namespace} | tail -20
```

## エスカレーション基準

以下の条件に該当する場合はエスカレーションする:

- [ ] 証明書が既に期限切れで通信不能
- [ ] cert-manager の再起動後も更新が失敗し続ける
- [ ] 外部 CA への証明書発行依頼が必要（権限が必要な場合）

エスカレーション先: [インシデント管理設計](../インシデント管理設計.md)

## 根本原因分析のポイント

- cert-manager の自動更新サイクル（デフォルト: 期限の30日前）が正しく設定されているか
- Let's Encrypt レート制限への抵触履歴
- 証明書の有効期限を定期的にモニタリングするアラートが機能しているか確認

## 関連ドキュメント

- [可観測性設計](../../可観測性設計.md)
- [インフラアラート設計](../../監視アラート設計.md)
