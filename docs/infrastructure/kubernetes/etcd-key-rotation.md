# etcd 暗号化キーローテーション手順書

## 概要

本手順書は、etcd に保存された Kubernetes リソース（Secrets / ConfigMaps）を暗号化する
AES-GCM キーを安全にローテーションするための手順を定義する。

### なぜ定期的なローテーションが必要か

etcd 暗号化キーは kube-apiserver が Secrets および ConfigMaps を etcd に書き込む際に使用する。
同一キーを長期間使用し続けると以下のリスクが増大する:

1. **鍵の露出リスク**: 期間が長いほど、CI/CD ログ・監査ログ・メモリダンプ等からの
   キー漏洩機会が増える。
2. **暗号文の蓄積**: 同一キーで暗号化された暗号文が増えるほど、統計的解析攻撃の
   成功可能性がわずかに上昇する（AES-GCM の nonce 空間枯渇対策としても有効）。
3. **コンプライアンス要件**: PCI-DSS・SOC2・ISO27001 等の規格では暗号化キーの
   定期ローテーションを要求することが多い。

推奨ローテーション周期: **90日ごと**（または重大なセキュリティインシデント発生時）。

---

## 前提条件

- `kubectl` で kube-apiserver が稼働するクラスタへのアクセス権限があること
- Vault へのアクセス権限があること（キー生成・保存用）
- `infra/kubernetes/security/encryption-config.yaml` が CI/CD 経由でデプロイされる
  環境であること（詳細は同ファイルのコメントを参照）
- kube-apiserver の静的 Pod 設定を変更できる権限（control plane ノードへの SSH または
  Kubernetes Cluster API 経由のアクセス）があること

---

## ローテーション手順

### ステップ 1: 新しいキーを生成し Vault に保存する

```bash
# 新しい 32 バイトランダムキーを生成する（AES-256-GCM 用）
NEW_SECRETS_KEY=$(head -c 32 /dev/urandom | base64)
NEW_CONFIGMAPS_KEY=$(head -c 32 /dev/urandom | base64)

# Vault に新しいキーを保存する
# 既存の古いキーは old_secrets_key / old_configmaps_key として保管しておく
vault kv patch secret/k1s0/infra/etcd-encryption-key \
  secrets_key="${NEW_SECRETS_KEY}" \
  configmaps_key="${NEW_CONFIGMAPS_KEY}"
```

### ステップ 2: encryption-config.yaml の先頭に新しいキーを追加する

`infra/kubernetes/security/encryption-config.yaml` を以下のように更新する。

**重要**: 古いキーは必ず末尾に残すこと。古いキーを削除する前に既存データを
新しいキーで再暗号化しないと、既存の Secrets が読めなくなりクラスタが停止する。

```yaml
# ローテーション中の状態（新キー先頭・旧キー末尾）
providers:
  - aesgcm:
      keys:
        # 新しいプライマリキー（新規書き込みはこちらで暗号化される）
        - name: secrets-key2
          secret: <NEW_KEY_FROM_VAULT>
        # 古いキー（既存データの復号に必要。再暗号化完了後に削除する）
        - name: secrets-key1
          secret: <OLD_KEY_FROM_VAULT>
  - identity: {}
```

### ステップ 3: kube-apiserver に新しい encryption-config.yaml を適用する

CI/CD パイプライン（GitHub Actions または ArgoCD）経由でデプロイする:

```bash
# CI/CD パイプラインが Vault からキーを取得して encryption-config.yaml を生成し、
# control plane ノードに配置する（cluster-etcd-encryption.yaml ワークフロー参照）

# 手動適用が必要な場合（緊急時のみ）:
# 1. 生成した encryption-config.yaml を各 control plane ノードにコピーする
#    scp encryption-config.yaml <control-plane-node>:/etc/kubernetes/
# 2. kube-apiserver の静的 Pod が自動的に再起動されることを確認する
#    kubectl get pods -n kube-system -l component=kube-apiserver -w
```

kube-apiserver が正常に再起動したことを確認する:

```bash
# kube-apiserver が Running 状態であることを確認する
kubectl get pods -n kube-system -l component=kube-apiserver

# API サーバーが正常に応答することを確認する
kubectl cluster-info
```

### ステップ 4: 既存のリソースを新しいキーで再暗号化する

kube-apiserver が新しい設定で起動した後、既存の Secrets と ConfigMaps を
新しいキーで再暗号化する。これにより、すべてのデータが新しいプライマリキーで
暗号化された状態になる。

```bash
# すべての Namespace の Secrets を再暗号化する
# kubectl replace はリソースを読み込んで書き直すため、
# kube-apiserver が新しいプライマリキーで再暗号化する
kubectl get secrets --all-namespaces -o json | kubectl replace -f -

# すべての Namespace の ConfigMaps を再暗号化する
kubectl get configmaps --all-namespaces -o json | kubectl replace -f -
```

再暗号化が完了したことを確認する:

```bash
# etcd から直接データを取得し、新しいキー名のプレフィックスで暗号化されているか確認する
# （ETCDCTL_API=3 が設定されていること）
ETCDCTL_API=3 etcdctl get /registry/secrets/default/sample-secret \
  --endpoints=<etcd-endpoint> \
  --cert=/etc/kubernetes/pki/etcd/server.crt \
  --key=/etc/kubernetes/pki/etcd/server.key \
  --cacert=/etc/kubernetes/pki/etcd/ca.crt \
  | head -c 20

# 出力が "k8s:enc:aesgcm:v1:secrets-key2:" で始まれば再暗号化済み
```

### ステップ 5: 古いキーを encryption-config.yaml から削除する

再暗号化が完了したことを確認したら、古いキーを削除して設定を更新する。

```yaml
# ローテーション完了後の状態（新キーのみ）
providers:
  - aesgcm:
      keys:
        # 新しいプライマリキーのみを残す
        - name: secrets-key2
          secret: <NEW_KEY_FROM_VAULT>
  - identity: {}
```

CI/CD パイプライン経由で再度デプロイし、kube-apiserver の再起動を確認する。

---

## 注意事項

### 作業前の確認事項

- [ ] クラスタの etcd バックアップが最新であることを確認する
  （`infra/kubernetes/backup/etcd-backup-cronjob.yaml` の最終実行を確認）
- [ ] kube-apiserver の冗長性を確認する（マルチマスター構成であること）
- [ ] 作業中はデプロイ・スケールアウトを控える（kube-apiserver 再起動中は API が不安定になる）

### 古いキーを削除するタイミング

古いキーは **再暗号化が完全に完了した後にのみ** 削除すること。
削除が早すぎると古いキーで暗号化された Secrets が復号できなくなり、
クラスタが正常に動作しなくなる。

### キー名の命名規則

キーをローテーションするたびにキー名の番号を増加させる:
`secrets-key1` → `secrets-key2` → `secrets-key3` ...

これにより etcd 内のデータがどのキー世代で暗号化されたかを追跡しやすくなる。

### ロールバック手順

kube-apiserver が新しい設定で起動しない場合:

1. control plane ノードの `/etc/kubernetes/encryption-config.yaml` を古い設定に戻す
2. kube-apiserver が自動再起動するのを待つ
3. `kubectl get pods -n kube-system -l component=kube-apiserver` で正常化を確認する

---

## 関連ドキュメント・ファイル

- `infra/kubernetes/security/encryption-config.yaml` — etcd 暗号化設定ファイル
- `infra/kubernetes/security/encryption-config.template.yaml` — CI/CD テンプレート
- `infra/kubernetes/backup/etcd-backup-cronjob.yaml` — etcd バックアップ CronJob
- `docs/architecture/adr/0031-etcd-encryption-at-rest.md` — etcd 暗号化の ADR
- `docs/infrastructure/kubernetes/kubernetes設計.md` — Kubernetes 全体設計書
- [Kubernetes 公式: Secrets の暗号化](https://kubernetes.io/docs/tasks/administer-cluster/encrypt-data/)
- [Kubernetes 公式: 暗号化プロバイダーの設定](https://kubernetes.io/docs/reference/config-api/apiserver-config.v1/)
