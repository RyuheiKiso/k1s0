# ADR-0087: cert-manager 外部 PKI 統合ロードマップ

## ステータス

承認済み（移行計画中）

## コンテキスト

外部技術監査（M-023）において、k1s0 の TLS 証明書管理が自己署名 3 段階チェーン
（selfsigned → k1s0-ca → internal-ca）で構成されていることが指摘された。

現在の構成:

```
selfsigned Issuer
    └── k1s0-ca (ClusterIssuer / 自己署名ルート CA)
            └── internal-ca (ClusterIssuer / 中間 CA)
                    └── サービス証明書
```

自己署名 PKI の課題:
- ブラウザ・OS の信頼ストアに登録されていないため、外部クライアントから証明書エラーが発生する
- 証明書の有効期限・ローテーション管理が手動になりやすい
- コンプライアンス要件（PCI DSS, SOC2 等）で認定 CA が要求される場合がある

現状は内部サービス間通信（mTLS）に限定して使用しており、直接的なセキュリティリスクはない。

## 決定

**現状**: 自己署名 3 段階チェーンは内部通信用として許容する。

**本番環境移行時**: 外部 PKI/CA との統合を計画する。
移行フェーズは以下の通り:

### フェーズ 1（現状・開発/ステージング）
- 自己署名 CA を継続使用
- cert-manager によるサービス証明書の自動ローテーションを維持

### フェーズ 2（本番環境移行・2026-H2 目標）
以下のいずれかの外部 PKI を選択して統合する:

**オプション A: Let's Encrypt（公開サービスの場合）**
```yaml
apiVersion: cert-manager.io/v1
kind: ClusterIssuer
metadata:
  name: letsencrypt-prod
spec:
  acme:
    server: https://acme-v02.api.letsencrypt.org/directory
    email: admin@example.com
    privateKeySecretRef:
      name: letsencrypt-prod
    solvers:
      - http01:
          ingress:
            class: kong
```

**オプション B: HashiCorp Vault PKI（内部サービスの場合）**
```yaml
apiVersion: cert-manager.io/v1
kind: ClusterIssuer
metadata:
  name: vault-issuer
spec:
  vault:
    path: pki/sign/k1s0-role
    server: https://vault.k1s0-system.svc.cluster.local
    auth:
      kubernetes:
        role: cert-manager
        mountPath: /v1/auth/kubernetes
        secretRef:
          name: cert-manager-vault-token
          key: token
```

**オプション C: 企業内 CA（エンタープライズ環境の場合）**
- cert-manager の `ca` Issuer に企業内 CA の証明書と秘密鍵を設定する

## 理由

- 現状の自己署名は開発・ステージング環境で機能しており、直ちに移行が必要な問題はない
- 本番環境では運用者の要件（パブリック/プライベート、コンプライアンス要件）によって最適な選択肢が異なる
- フェーズ分けにより段階的かつ安全に移行できる

## 影響

**ポジティブな影響**:

- 外部クライアントからの証明書エラーがなくなる
- コンプライアンス要件を満たせる
- 証明書の信頼チェーンが確立される

**ネガティブな影響・トレードオフ**:

- 外部 PKI との統合には追加の設定と管理コストが発生する
- Let's Encrypt は公開ドメインが必要（内部専用サービスには不向き）
- Vault PKI は Vault の可用性に依存する

## 代替案

| 案 | 概要 | 採用しなかった理由 |
|----|------|-----------------|
| 自己署名を継続 | 外部 PKI 統合を行わない | 本番環境ではコンプライアンス要件を満たさない可能性がある |
| 手動証明書管理 | cert-manager を使わず手動で証明書を更新 | 更新漏れによるサービス停止リスクがある |

## 参考

- [cert-manager 公式ドキュメント](https://cert-manager.io/docs/)
- [HashiCorp Vault PKI](https://developer.hashicorp.com/vault/docs/secrets/pki)
- `infra/terraform/modules/vault-pki/` — Vault PKI Terraform モジュール

## 更新履歴

| 日付 | 変更内容 | 変更者 |
|------|---------|--------|
| 2026-04-04 | 初版作成（M-023 監査対応） | k1s0 team |
