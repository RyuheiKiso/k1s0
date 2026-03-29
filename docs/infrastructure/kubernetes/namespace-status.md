# k8s Namespace 静的検証記録（M-05 監査対応）

## 概要

外部技術監査（M-05）の指摘に基づき、`infra/kubernetes/namespaces/` 配下の
Namespace YAML を静的検証した結果を記録する。

検証日: 2026-03-28

## 検証対象ファイル

| ファイル | Namespace 名 | Tier | PSS enforce |
|---------|------------|------|-------------|
| `cert-manager.yaml` | cert-manager | infra | baseline |
| `harbor.yaml` | harbor | infra | baseline |
| `ingress.yaml` | ingress | infra | baseline |
| `k1s0-business.yaml` | k1s0-business | business | restricted |
| `k1s0-service.yaml` | k1s0-service | service | restricted |
| `k1s0-system.yaml` | k1s0-system | system | restricted |
| `messaging.yaml` | messaging | infra | baseline |
| `observability.yaml` | observability | infra | baseline |
| `service-mesh.yaml` | service-mesh | infra | baseline |

## 検証項目と結果

### 1. 必須ラベルの確認

全 Namespace に以下のラベルが設定されていることを確認した。

| ラベル | 確認結果 |
|--------|---------|
| `tier` | 全 9 ファイルに設定済み |
| `app.kubernetes.io/part-of: k1s0` | 全 9 ファイルに設定済み |
| `app.kubernetes.io/managed-by: helm` | 全 9 ファイルに設定済み |
| `pod-security.kubernetes.io/enforce` | 全 9 ファイルに設定済み |
| `pod-security.kubernetes.io/audit` | 全 9 ファイルに設定済み |
| `pod-security.kubernetes.io/warn` | 全 9 ファイルに設定済み |

**判定: PASS**

### 2. PSS レベルの適切性確認

| Tier | enforce レベル | 理由 |
|------|--------------|------|
| system / service / business | restricted | アプリサービスは特権不要。最小権限の原則に従い restricted を適用 |
| infra | baseline | 各インフラコンポーネントが Capabilities・hostPath を必要とするため（詳細は各 YAML のコメント参照） |

infra tier の baseline 採用理由は各 YAML ファイルのコメントで個別に説明済み（L-14 監査対応）。

**判定: PASS**

### 3. Kustomization との整合性

`kustomization.yaml` に全 Namespace ファイルが列挙されていることを確認した。

```yaml
# infra/kubernetes/namespaces/kustomization.yaml
resources:
  - cert-manager.yaml
  - harbor.yaml
  - ingress.yaml
  - k1s0-business.yaml
  - k1s0-service.yaml
  - k1s0-system.yaml
  - messaging.yaml
  - observability.yaml
  - service-mesh.yaml
```

**判定: PASS**

### 4. audit/warn の設定確認

infra tier（baseline enforce）でも `audit: restricted` / `warn: restricted` を設定しており、
将来の restricted 移行に向けた違反検知が継続できる状態になっている。

**判定: PASS**

### 5. 特記事項

- k1s0-system Namespace には system tier のアプリサービス（auth, config, bff-proxy 等）が
  デプロイされる。restricted PSS により `runAsNonRoot: true`・`allowPrivilegeEscalation: false`・
  `capabilities.drop: [ALL]` が全 Pod に強制される
- verify 環境の postgres.yaml は verify 用 Namespace（k1s0-system）に含まれるが、
  PostgreSQL 公式イメージが root で起動するため restricted PSS 下では動作しない点に注意
  （詳細は `infra/kubernetes/verify/postgres.yaml` のコメント参照）

## 問題なし確認

検証の結果、Namespace YAML の構成に静的エラーは検出されなかった。

## 参考

- [infra/kubernetes/namespaces/](../../../infra/kubernetes/namespaces/)
- [kubernetes設計.md](kubernetes設計.md)
- 外部技術監査報告書 M-05: "k8s Namespace の設定検証と記録を求める"
- 外部技術監査報告書 L-14: "infra PSS baseline の理由追記"

## 更新履歴

| 日付 | 変更内容 | 変更者 |
|------|---------|--------|
| 2026-03-28 | 初版作成（M-05 監査対応） | 監査対応チーム |
