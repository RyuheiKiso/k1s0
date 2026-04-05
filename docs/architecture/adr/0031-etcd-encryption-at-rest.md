# ADR-0031: etcd encryption-at-rest（保存時暗号化）の採用

## ステータス

採用済み（2026-03-25）

## コンテキスト

Kubernetes クラスターの etcd には、Secrets・ConfigMap・ServiceAccount トークンなどの機密データが平文で保存される。デフォルト設定ではディスクアクセスが可能な攻撃者や内部関係者が etcd のデータファイルを直接読み取ることで機密情報を入手できる。

以下の要件から、etcd の保存時暗号化（encryption-at-rest）の導入を検討することになった。

- **セキュリティ規制対応**: PCI DSS・SOC 2 Type II に準拠するため、保存データの暗号化が必須要件となっている
- **FIPS 140-2 準拠要件**: 一部の顧客環境では米国政府のFIPSセキュリティ基準への準拠が求められている
- **最小権限の原則の徹底**: etcd へのアクセス権を持つオペレーターであっても、Secrets の平文内容を閲覧できないようにする必要がある
- **監査対応**: 外部セキュリティ監査（2026-03-22）において、etcd の Secrets 暗号化が未対応であることが指摘された

Kubernetes は `EncryptionConfiguration` リソースにより複数の暗号化プロバイダーを選択的に利用できる。主な候補として `aescbc`、`secretbox`、`aesgcm` がある。

## 決定

**aescbc 方式による etcd encryption-at-rest を採用する。**

`/etc/kubernetes/encryption-config.yaml` に以下の設定を適用し、kube-apiserver の起動オプション `--encryption-provider-config` で参照する。

対象リソース:
- `secrets`（最優先）
- `configmaps`（機密値が混在するリスクへの対応）

暗号化設定の構成:
```yaml
apiVersion: apiserver.config.k8s.io/v1
kind: EncryptionConfiguration
resources:
  - resources:
      - secrets
      - configmaps
    providers:
      - aescbc:
          keys:
            - name: key1
              secret: <base64エンコードされた32バイト鍵>
      - identity: {}
```

鍵は Kubernetes Secret または Vault の Transit Secrets Engine で管理し、定期的なローテーション手順（90日ごと）を確立する。

## 理由

### aescbc を選択した理由

**FIPS 140-2 準拠**:
- `aescbc`（AES-256-CBC）は FIPS 140-2 認定暗号アルゴリズムである
- `secretbox`（XSalsa20 + Poly1305）は FIPS 140-2 非認定であり、政府系・金融系顧客の要件を満たせない

**既存ツールとの互換性**:
- `aescbc` は OpenSSL ベースのツールチェーンと親和性が高く、鍵のオフライン検証・バックアップ復号が標準ツールで実施できる
- `secretbox` は NaCl/libsodium 依存であり、既存の運用ツールセットとの統合コストが高い

**Kubernetes 公式推奨**:
- Kubernetes 公式ドキュメントおよび CIS Kubernetes Benchmark v1.8 では `aescbc` を推奨暗号化プロバイダーとして記載している

**運用成熟度**:
- `aescbc` は Kubernetes 1.7 より提供されており、本番環境での実績が豊富
- `aesgcm`（GCM モード）は認証付き暗号であるが、Kubernetes の実装では nonce の使い回しリスクがあり推奨されていない

## 影響

**ポジティブな影響**:

- etcd へのディスクレベルアクセスが可能な攻撃者から Secrets の内容を保護できる
- PCI DSS 要件 3.4（保存データの保護）、SOC 2 CC6.1（論理アクセス）に対応できる
- FIPS 140-2 準拠が必要な顧客環境にも対応可能になる
- 外部監査指摘事項が解消される

**ネガティブな影響・トレードオフ**:

- kube-apiserver の Secret 読み取り時に復号処理が発生し、わずかなレイテンシ増加（数ms程度）が生じる
- 暗号化鍵のローテーション時に全 Secrets の再暗号化が必要となる（`kubectl get secrets --all-namespaces -o json | kubectl replace -f -` による一括更新）
- 鍵を紛失した場合、暗号化済みデータが復元不能になるリスクがある（鍵のバックアップ管理が必須）
- 暗号化前に書き込まれた Secrets は `identity` プロバイダーで読み取れるが、既存データの再暗号化が必要（移行作業が発生）

## 代替案

検討したが採用しなかった案を記載する。

| 案 | 概要 | 採用しなかった理由 |
|----|------|-----------------|
| `secretbox` | XSalsa20 + Poly1305 による認証付き暗号。CPU 効率が高い | FIPS 140-2 非認定のため、規制要件を満たせない |
| `aesgcm` | AES-256-GCM による認証付き暗号 | Kubernetes 実装での nonce 使い回しリスクが指摘されており、公式に非推奨 |
| `kms` (KMS v2) | クラウドプロバイダーの KMS（AWS KMS・GCP Cloud KMS）との統合 | 外部依存が増加し、マルチクラウド環境でのポータビリティが低下する。将来的な移行候補として検討継続 |
| 暗号化なし（現状維持） | etcd デフォルト設定（平文保存） | 外部監査指摘事項であり、規制要件を満たせないため却下 |

## 参考

- [Kubernetes 公式ドキュメント: Encrypt Secret Data at Rest](https://kubernetes.io/docs/tasks/administer-cluster/encrypt-data/)
- [CIS Kubernetes Benchmark v1.8 - 1.2.31 Ensure that encryption providers are appropriately configured](https://www.cisecurity.org/benchmark/kubernetes)
- [NIST SP 800-111: Guide to Storage Encryption Technologies](https://csrc.nist.gov/publications/detail/sp/800-111/final)
- [ADR-0019: Vault ポリシーのドメイン単位シークレット分離](./0019-vault-domain-secret-isolation.md)
- 外部監査対応 2026-03-22

## 更新履歴

| 日付 | 変更内容 | 変更者 |
|------|---------|--------|
| 2026-03-25 | 初版作成（外部監査対応・etcd encryption-at-rest 採用決定） | @k1s0 |
