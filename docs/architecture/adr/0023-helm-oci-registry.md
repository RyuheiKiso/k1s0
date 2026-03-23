# ADR-0023: Helm Chart の依存関係参照を OCI レジストリに移行

## ステータス

承認済み

## コンテキスト

`infra/helm/services/` 配下の全サービス（33件）の `Chart.yaml` において、共通チャート `k1s0-common` への依存関係が `file://` 参照で定義されていた。

```yaml
# 変更前の状態（問題のある参照）
dependencies:
  - name: k1s0-common
    version: "0.1.0"
    repository: "file://../../../charts/k1s0-common"
```

この `file://` 参照は以下の問題を引き起こしていた:

1. **CI/CD パイプラインでのビルド失敗**: GitHub Actions や ArgoCD のような外部環境では、ローカルファイルシステムのパスが解決できない。
2. **Kubernetes クラスタへのデプロイ不可**: `helm install` / `helm upgrade` を実行する環境でローカルチャートディレクトリが存在しない場合、`helm dependency update` が失敗する。
3. **外部監査 C-02 での指摘**: 本番環境での運用可能性を担保するため、OCI レジストリ参照への移行が必要と判断された。

当初から `NOTE(H-007)` コメントとして「本番環境では OCI レジストリ参照に変更すること」が記載されており、OCI 移行は計画済みだったが実施されていなかった。

## 決定

全33件の `Chart.yaml` および `Chart.lock` における `k1s0-common` チャートの依存関係参照を、`file://` 参照から OCI レジストリ参照に変更する。

```yaml
# 変更後の状態（OCI レジストリ参照）
dependencies:
  - name: k1s0-common
    version: "0.1.0"
    repository: "oci://harbor.k1s0.io/helm-charts"
```

対象ファイル:
- `infra/helm/services/system/` 配下 29 チャート
- `infra/helm/services/business/` 配下 1 チャート
- `infra/helm/services/service/` 配下 3 チャート

## 理由

### OCI レジストリを選択した理由

1. **Helm 3.8+ での公式サポート**: OCI レジストリは Helm 3.8 以降で安定版として提供されており、`oci://` プロトコルが標準的な配布方法となっている。
2. **環境非依存**: レジストリ URL が固定されるため、開発・ステージング・本番すべての環境で同一の参照が使用できる。
3. **Harbor との統合**: プロジェクトでは `harbor.k1s0.io` を内部コンテナ/チャートレジストリとして採用しており、OCI チャートの保管も同一インフラで完結する。
4. **バージョン管理の明確化**: OCI レジストリにプッシュされたチャートはイミュータブルなバージョン管理が可能であり、`Chart.lock` のダイジェスト検証と組み合わせて再現性が保証される。

### ローカル開発への配慮

ローカル開発時の `helm dependency update` は OCI レジストリへのアクセスを必要とするが、以下の手順で対応可能:

1. `harbor.k1s0.io` に VPN 経由または内部ネットワーク経由でアクセスする。
2. または `charts/` ディレクトリに手動で k1s0-common チャートを配置し、一時的に `file://` 参照に戻す（コミット前に必ず OCI 参照に戻すこと）。

## 影響

**ポジティブな影響**:

- CI/CD パイプライン（GitHub Actions、ArgoCD）での `helm dependency update` が正常に動作するようになる
- 本番 Kubernetes クラスタへのデプロイが可能になる
- チャートのバージョン管理がレジストリ側で一元管理される
- 外部監査 C-02 の指摘事項が解消される

**ネガティブな影響・トレードオフ**:

- ローカル開発時に `harbor.k1s0.io` へのネットワークアクセスが必要になる（VPN 接続が必須）
- `k1s0-common` チャートを OCI レジストリに事前プッシュしておく必要がある
- ネットワーク疎通がない完全オフライン環境では `helm dependency update` が失敗する

## 代替案

| 案 | 概要 | 採用しなかった理由 |
|----|------|-----------------|
| 案 A: file:// 参照を維持 | ローカルパスへの依存を継続 | CI/CD・本番環境でのビルド失敗が解消されない。外部監査 C-02 の要件を満たさない |
| 案 B: チャートを各サービスディレクトリにコピー | k1s0-common を各サービスの `charts/` にベンダリング | チャートの重複管理が発生し、共通チャートの更新時に全サービスの更新が必要になる |
| 案 C: HTTP/HTTPS レポジトリ（index.yaml 方式） | 従来の Helm チャートリポジトリ形式 | OCI 方式と比較してインフラが複雑になり、Harbor の OCI サポートを活用できない |
| 案 D: Helm Subchart としてインライン定義 | 共通チャートを廃止して各サービスに直接定義 | DRY 原則に違反し、共通設定の変更が全サービスに波及するリスクが高まる |

## 参考

- [Helm OCI ドキュメント](https://helm.sh/docs/topics/registries/)
- [Harbor Helm チャートリポジトリ](https://goharbor.io/docs/main/working-with-projects/working-with-images/managing-helm-charts/)
- 外部監査報告書 C-02: Helm Chart の file:// 参照問題
- [ADR-0019: Vault ドメイン別シークレット分離](0019-vault-domain-secret-isolation.md)
