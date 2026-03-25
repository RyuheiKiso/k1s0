# ADR-0035: Dockerfile テンプレート戦略

## ステータス

承認済み

## コンテキスト

`regions/system/server/rust/` 配下の 27 個の Rust サービス Dockerfile が 99% 同一の内容を持つ。
差異はパッケージ名（`-p k1s0-{service}-server`）の 1 か所のみである。

この状況は以下の問題を引き起こしている。

- ベースイメージのバージョン・ダイジェスト更新時に 27 ファイルを手動修正する必要がある
- `cargo-chef` のバージョンや `apt-get` パッケージ追加時も全ファイルへの変更が必要
- 差分レビューが煩雑になり、意図しない個別差異が混入するリスクがある

## 決定

現時点では個別 Dockerfile を維持する。将来的に `docker buildx bake` による HCL テンプレートへの移行を検討する。

共通変更は `scripts/update-dockerfiles.sh` 等のスクリプト一括適用で対処する。

## 理由

1. **デバッグ容易性**: 個別 Dockerfile はビルドエラー発生時にサービス単位で原因調査しやすい。テンプレート化すると問題の切り分けが困難になる
2. **cargo chef との将来的な統合可能性**: `cargo chef` マルチステージビルドは `ARG PACKAGE_NAME` でパッケージ名を受け取れるため、将来的に 1 ファイルへの統合が技術的に可能である。現時点では統合コストと恩恵のバランスが課題
3. **共通変更の機械的適用**: ダイジェスト固定の自動更新（`scripts/pin-docker-digests.sh` と `.github/workflows/pin-docker-digests.yaml`）で最も頻度の高い変更は既に自動化されている。その他の共通変更もスクリプトで一括適用できる
4. **移行リスクの回避**: `docker buildx bake`（HCL）は学習コストが高く、CI/CD の変更範囲が広い。現時点ではリスクに対する恩恵が小さい

## 影響

**ポジティブな影響**:

- 各サービスの Dockerfile が独立しているため、サービスごとの例外的な設定変更が容易
- ビルドエラーのデバッグが容易（どのサービスのどのステップで失敗したか明確）
- ダイジェスト固定の自動化により、最も頻繁な変更（ベースイメージ更新）は既に解決済み

**ネガティブな影響・トレードオフ**:

- 共通変更の適用漏れリスクがある（スクリプトで緩和）
- ファイル数が多くリポジトリのサイズが増加する
- コードレビュー時に差分が多くなる

## 代替案

| 案 | 概要 | 採用しなかった理由 |
|----|------|-----------------|
| `docker buildx bake`（HCL テンプレート） | `docker-bake.hcl` に全サービスを定義し、パッケージ名を変数化する | 学習コストが高い。CI/CD の `docker build` コマンドを全て `docker buildx bake` に変更する必要がある |
| `Dockerfile.template` + `envsubst` | テンプレートファイルからシェルスクリプトで Dockerfile を生成する | 生成された Dockerfile をリポジトリに含めるか否かで運用が複雑になる。GitHub のコード差分追跡と相性が悪い |
| ARG でパッケージ名を統一した 1 Dockerfile | `ARG PACKAGE_NAME` を使用して全サービスを 1 ファイルでビルド | `cargo chef prepare` ステージがパッケージ名に依存するため、ステージ間で ARG の受け渡しが必要になりビルドが複雑化する |

## 参考

- [Dockerイメージ戦略](../../infrastructure/docker/Dockerイメージ戦略.md) — ビルドコンテキスト最適化・ダイジェスト固定方針
- [pin-docker-digests.yaml](../../../../.github/workflows/pin-docker-digests.yaml) — ダイジェスト自動更新ワークフロー
- `scripts/pin-docker-digests.sh` — ダイジェスト固定スクリプト
- [docker buildx bake ドキュメント](https://docs.docker.com/build/bake/)

## 更新履歴

| 日付 | 変更内容 | 変更者 |
|------|---------|--------|
| 2026-03-25 | 初版作成（MED-01 監査対応） | — |
