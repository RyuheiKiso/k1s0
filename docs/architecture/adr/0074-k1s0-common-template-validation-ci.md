# ADR-0074: k1s0-common Library Chart 変更時の全コンシューマーチャート helm template 検証を CI に追加する

## ステータス

承認済み

## コンテキスト

`infra/helm/charts/k1s0-common/` は全サービスチャートが依存する Helm Library Chart である。  
このチャートには Deployment・Service・HPA・PDB・Istio VirtualService 等の共通テンプレートが集約されており、  
一箇所の変更が全 26+ サービスのマニフェスト出力に影響する。

既存の `helm-lint` ジョブ（H-8 対応）は個別サービスチャートの構文チェックを実施しているが、  
以下の問題を検出できない：

1. **k1s0-common テンプレートの変更がコンシューマーチャートでレンダリングエラーを引き起こす場合**  
   例：必須変数の追加、既存変数名の変更、テンプレート関数のシグネチャ変更
2. **必須フィールド（`required`）の追加により既存 values.yaml が不完全になる場合**  
   コンシューマーチャートの `values.yaml` が更新されずに k1s0-common だけ変更されると、  
   `helm template` が失敗するがリント段階では検出されない

INFRA-003 監査指摘として、Library Chart 変更の影響範囲を CI で自動検証する仕組みが必要との指摘を受けた。

## 決定

`_validate.yaml` に `helm-common-template-validate` ジョブを追加する。

- `infra/helm/services/` 配下の全チャートを走査し、`Chart.yaml` に `k1s0-common` への依存を持つチャートを特定する
- 各対象チャートに対して `helm dependency update` → `helm template` を実行する
- テンプレートエンジンのエラーが発生した場合は `exit 1` で CI を失敗させる（ブロッカー）
- `helm dependency update` が OCI レジストリ未接続等で失敗する場合は `warning` に留める（CI 環境の制約を考慮）

このジョブは既存の `helm-lint` と独立して実行され、`helm-lint` と合わせて二層の検証を提供する。

## 理由

- **Library Chart の影響範囲は広い**：k1s0-common の変更は全サービスに波及するため、個別リントだけでは不十分である。
- **helm template は最も実態に近い検証**：実際の `values.yaml` を使ってテンプレートをレンダリングするため、`helm lint` では検出できない `required` 不足や変数名ミスを検出できる。
- **dependency update の失敗を warning に留める理由**：CI 環境では OCI レジストリ（harbor.internal.example.com）に接続できない場合がある。接続できないだけで CI 全体が失敗すると false negative が多発するため、レジストリ接続失敗はスキップとする。

## 影響

**ポジティブな影響**:

- k1s0-common のテンプレート変更による全サービスへの影響を PR マージ前に自動検出できる
- `required` フィールド追加による values.yaml の更新漏れを早期に検知できる
- ライブラリチャートのリファクタリング時の安全性が向上する

**ネガティブな影響・トレードオフ**:

- CI 実行時間が増加する（全コンシューマーチャートの dependency update + template 実行）
- OCI レジストリに接続できない CI 環境では対象チャートがスキップされ、検証カバレッジが低下する
  - この制約については、将来的に OCI レジストリへの CI アクセス確保か、チャートのローカルキャッシュ化を検討する

## 代替案

| 案 | 概要 | 採用しなかった理由 |
|----|------|-----------------|
| 案 A: k1s0-common 変更時のみ実行（paths-filter）| Library Chart の変更があった PR のみで検証を走らせる | paths-filter は過去に廃止方針（H-8 対応）。Library Chart 以外の変更でも影響が出る場合があり検出漏れのリスクがある |
| 案 B: helm unittest の導入 | helm-unittest プラグインでテストケースを記述する | テストケースの作成・維持コストが高い。まず helm template での構文検証を優先する |
| 案 C: 専用の validation スクリプト（scripts/）| bash スクリプトに切り出して CI から呼び出す | CI ジョブのインライン記述で十分な複雑度。専用スクリプトは過剰投資 |

## 参考

- `infra/helm/charts/k1s0-common/` — Library Chart 本体
- `infra/helm/services/` — コンシューマーチャート一覧
- `.github/workflows/_validate.yaml` — `helm-common-template-validate` ジョブ
- [ADR-0023: Helm OCI Registry](0023-helm-oci-registry.md)

## 更新履歴

| 日付 | 変更内容 | 変更者 |
|------|---------|--------|
| 2026-04-03 | 初版作成（INFRA-003 監査対応） | @kiso |
