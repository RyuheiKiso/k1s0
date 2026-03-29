# ADR-0051: Kubernetes マニフェストのプレースホルダー自動検証 CI パイプライン設計

## ステータス

承認済み

## コンテキスト

k1s0 の Kubernetes マニフェスト（`infra/kubernetes/`）には本番デプロイ前に実際の値で置換が必要なプレースホルダー（例: `<REPLACE_WITH_ACTUAL_SECRET>`）が存在する。

外部技術監査（CRIT-5, CRIT-6）にて以下のファイルでプレースホルダーが残存していることが指摘された:

- `infra/kubernetes/security/encryption-config.yaml`: `<REPLACE_WITH_BASE64_ENCODED_32_BYTE_KEY>`
- `infra/kubernetes/ingress/kong-consumer-grafana.yaml`: `<REPLACE_WITH_GRAFANA_API_KEY>`

これらのプレースホルダーが本番クラスタにデプロイされた場合、設定が無効になるか機密情報の欠落によるセキュリティ上の問題が発生する。

## 決定

CI パイプラインに `validate-k8s-placeholders` ジョブを追加し、`scripts/check-placeholder-secrets.sh` が `<REPLACE_WITH_` パターンを検出した場合は CI を失敗させる。

```yaml
# .github/workflows/_validate.yaml
validate-k8s-placeholders:
  name: k8s プレースホルダー検証
  runs-on: ubuntu-latest
  steps:
    - uses: actions/checkout@v4
    - name: Check for unresolved placeholders
      run: bash scripts/check-placeholder-secrets.sh
```

`check-placeholder-secrets.sh` は以下のファイルを検査する:

- `infra/kubernetes/security/encryption-config.yaml`
- `infra/kubernetes/ingress/kong-consumer-grafana.yaml`

## 理由

- プレースホルダーの残存は人的ミスによる可能性が高い。自動検証で確実に防止する
- 既存の `_validate.yaml` ワークフローに組み込むことで、PR マージ前に強制チェックできる
- シェルスクリプトによる実装はプラットフォーム非依存で、CI ランナー環境に追加ツールが不要
- プレースホルダーパターン `<REPLACE_WITH_` は意図的な変数名と区別しやすいため誤検知が少ない

## 影響

**ポジティブな影響**:

- プレースホルダー残存によるデプロイ事故を自動的に検出・防止できる
- 将来新しいプレースホルダーファイルが追加された場合も、スクリプトに追加するだけで検証対象に含められる
- レビュー時の手動確認負担が軽減される

**ネガティブな影響・トレードオフ**:

- 新しいプレースホルダーパターンを使用する場合はスクリプト更新が必要
- verify 環境や開発環境でプレースホルダーを意図的に残す場合（コメントアウト等）は検知の例外処理が必要

## 代替案

| 案 | 概要 | 採用しなかった理由 |
|----|------|-----------------|
| Kustomize + Secret 管理ツール | 本番環境では external-secrets 等でプレースホルダーを排除 | 開発・verify 環境の検証は別途必要。CI での早期検出の補完として両立する |
| git-secrets / detect-secrets | 汎用的なシークレット検出ツール | プレースホルダーパターン `<REPLACE_WITH_` に特化したルールが追加設定なしには難しい |
| PR テンプレートのチェックリスト | レビュー担当者が手動確認 | 人的ミスを防げない。自動化で確実性を高める |

## 参考

- `scripts/check-placeholder-secrets.sh`
- `.github/workflows/_validate.yaml`
- `infra/kubernetes/security/encryption-config.yaml`
- `infra/kubernetes/ingress/kong-consumer-grafana.yaml`
- [CRIT-5, CRIT-6 外部技術監査対応]

## 更新履歴

| 日付 | 変更内容 | 変更者 |
|------|---------|--------|
| 2026-03-29 | 初版作成（CRIT-5/CRIT-6 外部技術監査対応） | system |
