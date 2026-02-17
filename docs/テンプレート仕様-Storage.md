# テンプレート仕様 — Storage

## 概要

本ドキュメントは、k1s0 CLI の「ひな形生成」機能で生成される **Ceph StorageClass** および **PersistentVolumeClaim（PVC）** のテンプレート仕様を定義する。データベースを使用するサービスに対して、`tier` に応じたストレージサイズの PVC と、Ceph RBD ベースの StorageClass を自動生成する。

ストレージ設計の全体像は [インフラ設計](インフラ設計.md) を、Terraform による StorageClass 管理は [terraform設計](terraform設計.md) を参照。

## 生成対象

| kind       | StorageClass | PVC          |
| ---------- | ------------ | ------------ |
| `server`   | 条件付き生成 | 条件付き生成 |
| `bff`      | 生成しない   | 生成しない   |
| `client`   | 生成しない   | 生成しない   |
| `library`  | 生成しない   | 生成しない   |
| `database` | 条件付き生成 | 条件付き生成 |

- **条件**: `has_database == true` の場合にのみ生成する
- server kind でデータベースを利用するサービス、または database kind のサービスが対象

## 配置パス

生成されるリソースファイルは `infra/storage/` 配下にサービス名ディレクトリを作成して配置する。

| ファイル       | 配置パス                                                   |
| -------------- | ---------------------------------------------------------- |
| StorageClass   | `infra/storage/{{ service_name }}/storage-class.yaml`      |
| PVC            | `infra/storage/{{ service_name }}/pvc.yaml`                |

## テンプレートファイル一覧

テンプレートは `CLI/templates/storage/` 配下に配置する。

| テンプレートファイル          | 生成先                                                     | 説明                          |
| ----------------------------- | ---------------------------------------------------------- | ----------------------------- |
| `storage-class.yaml.tera`     | `infra/storage/{{ service_name }}/storage-class.yaml`      | Ceph StorageClass 定義        |
| `pvc.yaml.tera`               | `infra/storage/{{ service_name }}/pvc.yaml`                | PersistentVolumeClaim 定義    |

### ディレクトリ構成

```
CLI/
└── templates/
    └── storage/
        ├── storage-class.yaml.tera
        └── pvc.yaml.tera
```

## 使用するテンプレート変数

Storage テンプレートで使用する変数を以下に示す。変数の定義と導出ルールの詳細は [テンプレートエンジン仕様](テンプレートエンジン仕様.md) を参照。

| 変数名          | 型      | StorageClass | PVC | 用途                                          |
| --------------- | ------- | ------------ | --- | --------------------------------------------- |
| `service_name`  | String  | 用           | 用  | リソース名                                    |
| `tier`          | String  | 用           | 用  | reclaimPolicy、ストレージサイズの決定         |
| `namespace`     | String  | -            | 用  | PVC の配置先 Namespace                        |
| `has_database`  | Boolean | -            | -   | 生成判定（true の場合のみ生成）               |
| `database_type` | String  | -            | 用  | ラベル付与（postgresql / mysql 等）           |

### Tier 別ストレージ設定

| 設定             | system | business | service |
| ---------------- | ------ | -------- | ------- |
| ストレージサイズ | 50Gi   | 20Gi     | 10Gi    |
| reclaimPolicy    | Retain | Retain   | Retain  |

---

## StorageClass テンプレート（storage-class.yaml.tera）

Ceph RBD を使用する StorageClass を定義する。サービス固有の StorageClass を作成し、ストレージプールの分離と管理の柔軟性を確保する。

```tera
apiVersion: storage.k8s.io/v1
kind: StorageClass
metadata:
  name: {{ service_name }}-ceph-block
  labels:
    app.kubernetes.io/name: {{ service_name }}
    tier: {{ tier }}
provisioner: rbd.csi.ceph.com
parameters:
  clusterID: ceph-cluster
  pool: k8s-block
  imageFormat: "2"
  imageFeatures: layering
  csi.storage.k8s.io/provisioner-secret-name: rook-csi-rbd-provisioner
  csi.storage.k8s.io/provisioner-secret-namespace: rook-ceph
  csi.storage.k8s.io/controller-expand-secret-name: rook-csi-rbd-provisioner
  csi.storage.k8s.io/controller-expand-secret-namespace: rook-ceph
  csi.storage.k8s.io/node-stage-secret-name: rook-csi-rbd-node
  csi.storage.k8s.io/node-stage-secret-namespace: rook-ceph
reclaimPolicy: Retain
allowVolumeExpansion: true
mountOptions:
  - discard
```

### ポイント

- provisioner は `rbd.csi.ceph.com`（Ceph RBD CSI ドライバ）を使用する
- `reclaimPolicy: Retain` で全 Tier 共通とし、データの誤削除を防止する（dev 環境での `Delete` への変更は Terraform の環境別 tfvars で制御する）
- `allowVolumeExpansion: true` でボリュームの動的拡張を許可する
- `mountOptions: discard` で TRIM/Unmap をサポートし、ストレージの効率的な利用を促進する
- インフラ設計の StorageClass 定義に準拠する

---

## PVC テンプレート（pvc.yaml.tera）

PersistentVolumeClaim を定義する。Tier に応じたストレージサイズを設定し、`ceph-block` StorageClass を使用する。

```tera
apiVersion: v1
kind: PersistentVolumeClaim
metadata:
  name: {{ service_name }}-data
  namespace: {{ namespace }}
  labels:
    app.kubernetes.io/name: {{ service_name }}
    app.kubernetes.io/component: storage
    tier: {{ tier }}
{% if database_type %}
    database-type: {{ database_type }}
{% endif %}
spec:
  accessModes:
    - ReadWriteOnce
  storageClassName: ceph-block
  resources:
    requests:
{% if tier == "system" %}
      storage: 50Gi
{% elif tier == "business" %}
      storage: 20Gi
{% elif tier == "service" %}
      storage: 10Gi
{% endif %}
```

### ポイント

- `accessModes: ReadWriteOnce` で単一ノードからの読み書きアクセスを許可する（データベース用途に適切）
- `storageClassName: ceph-block` でクラスタ共通の Ceph ブロックストレージを使用する（サービス固有の StorageClass は個別調整が必要な場合に使用する）
- system Tier は50Gi、business Tier は20Gi、service Tier は10Giをデフォルトサイズとする
- `database_type` が指定されている場合、ラベルとして付与し、バックアップジョブ等での識別に利用する
- `allowVolumeExpansion: true`（StorageClass 側）により、運用中にサイズの拡張が可能

---

## 条件付き生成表

CLI の対話フローで選択されたオプションに応じて、生成されるリソースの内容が変わる。

| 条件                         | 選択肢                             | 生成への影響                                        |
| ---------------------------- | ---------------------------------- | --------------------------------------------------- |
| データベース (`has_database`) | `true`                             | Storage リソースを生成する                          |
| データベース (`has_database`) | `false`                            | Storage リソースを生成しない                        |
| Tier (`tier`)                | `system`                           | ストレージサイズ 50Gi                               |
| Tier (`tier`)                | `business`                         | ストレージサイズ 20Gi                               |
| Tier (`tier`)                | `service`                          | ストレージサイズ 10Gi                               |
| DB 種別 (`database_type`)    | `postgresql` / `mysql` 等          | PVC ラベルに database-type を付与                   |
| kind (`kind`)                | `server`（has_database=true）      | Storage リソースを生成する                          |
| kind (`kind`)                | `database`（has_database=true）    | Storage リソースを生成する                          |
| kind (`kind`)                | `bff` / `client` / `library`      | Storage リソースを生成しない                        |

---

## 生成例

### system Tier の PostgreSQL サーバーの場合

入力:
```json
{
  "service_name": "auth-service",
  "tier": "system",
  "namespace": "k1s0-system",
  "has_database": true,
  "database_type": "postgresql"
}
```

生成されるファイル:
- `infra/storage/auth-service/storage-class.yaml` -- rbd.csi.ceph.com、reclaimPolicy=Retain
- `infra/storage/auth-service/pvc.yaml` -- 50Gi、storageClassName=ceph-block、database-type=postgresql

### business Tier の MySQL サーバーの場合

入力:
```json
{
  "service_name": "accounting-api",
  "tier": "business",
  "namespace": "k1s0-business",
  "has_database": true,
  "database_type": "mysql"
}
```

生成されるファイル:
- `infra/storage/accounting-api/storage-class.yaml` -- rbd.csi.ceph.com、reclaimPolicy=Retain
- `infra/storage/accounting-api/pvc.yaml` -- 20Gi、storageClassName=ceph-block、database-type=mysql

### service Tier のデータベース単体の場合

入力:
```json
{
  "service_name": "order-db",
  "tier": "service",
  "namespace": "k1s0-service",
  "has_database": true,
  "database_type": "postgresql"
}
```

生成されるファイル:
- `infra/storage/order-db/storage-class.yaml` -- rbd.csi.ceph.com、reclaimPolicy=Retain
- `infra/storage/order-db/pvc.yaml` -- 10Gi、storageClassName=ceph-block、database-type=postgresql

---

## 関連ドキュメント

- [インフラ設計](インフラ設計.md) -- オンプレミスインフラ構成・Ceph ストレージ設計
- [terraform設計](terraform設計.md) -- Terraform kubernetes-storage モジュール・StorageClass 管理
- [テンプレートエンジン仕様](テンプレートエンジン仕様.md) -- テンプレート変数・条件分岐・フィルタの仕様
- [テンプレート仕様-Helm](テンプレート仕様-Helm.md) -- Helm テンプレート仕様（PVC マウント設定連携）
- [テンプレート仕様-Config](テンプレート仕様-Config.md) -- Config テンプレート仕様（データベース接続設定連携）
- [テンプレート仕様-Observability](テンプレート仕様-Observability.md) -- Observability テンプレート仕様
- [kubernetes設計](kubernetes設計.md) -- Namespace・StorageClass 設計
