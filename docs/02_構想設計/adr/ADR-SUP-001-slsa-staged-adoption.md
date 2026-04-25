# ADR-SUP-001: SLSA v1.1 Level 2 をリリース時点で達成、Level 3 を運用蓄積後の到達目標とする

- ステータス: Accepted
- 起票日: 2026-04-24
- 決定日: 2026-04-24
- 起票者: kiso ryuhei
- 関係者: システム基盤チーム / セキュリティチーム / 法務部 / SRE / 契約レビュー担当 / CI/CD 担当

## コンテキスト

k1s0 は起案者が個人で開発する OSS であるが、採用する 採用側組織の監査部門・情報システム部門からは「ビルド由来の改ざん耐性をどう証明するか」を採用検討の中で問われる。SolarWinds SUNBURST（2020）、Codecov bash uploader（2021）、3CX DesktopApp（2023）、xz-utils backdoor（2024）と続くビルドサプライチェーン事件の系譜を受け、採用側組織の情報セキュリティ指針は「ビルド成果物が改ざんされていないことを第三者検証可能にする」ことを CVE 対応と同等に重視する方向に傾いている。

OpenSSF の SLSA（Supply-chain Levels for Software Artifacts）は 2023 年に v1.0、2024 年に v1.1 が策定され、ビルドトラック（Build Track）を Level 1〜3 で段階化している。要点は以下の通り。

- **Level 1**: ビルドが自動化されている、プロビナンス（誰が・何を・いつ・どう作ったか）が文書化されている
- **Level 2**: ビルドがホストされたビルドシステム（GitHub Actions 等）で実行され、プロビナンスが署名されている。改ざん耐性が確率的に確保される
- **Level 3**: ビルドがハーメティック（外部依存から隔離）かつ再現可能で、プロビナンスが強い非否認性（改ざん時に即検出可能）を持つ

k1s0 は個人開発 OSS としてリリースされるため、Level 3 のハーメティックビルド要求（ビルド実行時に外部ネットワークアクセスを遮断し、全依存を事前解決）を満たすための GitHub Actions カスタムランナー整備をリリース時点から完備するのは、運用負荷の観点から現実的でない。一方、Level 1 のみでは 採用側組織の監査要件を満たさず、採用検討で通らない。

同時に、「どの SLSA Level にいつ到達するか」を明示しないまま採用検討を通すと、運用蓄積後に「どのレベルにいるのか分からない」状態になり、監査対応で不利になる。本 ADR はリリース時点で Level 2、運用蓄積後に Level 3 へ到達する到達計画を確定し、署名基盤（sigstore/cosign keyless）・SBOM（CycloneDX）・Forensics Runbook（image hash から tier1 公開 11 API への影響経路逆引き）までを固定する。

## 決定

**SLSA v1.1 Build Track の Level 2 を k1s0 リリース時点で達成、Level 3 を運用蓄積後（Hermetic Build 移行段階）で到達する。**

### リリース時点の到達状態（Level 2）

- 全ビルドを GitHub Actions の hosted runner で実行する。self-hosted runner はビルド対象外に限定
- プロビナンスは SLSA v1.1 provenance schema に準拠した JSON を `slsa-github-generator` で生成
- 署名は **sigstore/cosign の keyless（GitHub Actions OIDC トークン由来）** を用いる。長期鍵の管理コストを排除
- 生成された provenance は、OCI イメージと並んで `.sig` / `.intoto.jsonl` として同一 OCI リポジトリに格納
- SBOM は **CycloneDX 1.6** 形式で生成（`syft` 利用）、`cosign attest --type cyclonedx` で attestation 化
- 本番 namespace では Kyverno ImageVerify（[ADR-CICD-003](ADR-CICD-003-kyverno.md)）で署名 + provenance attestation + SBOM attestation の 3 点を必須検証

### Hermetic Build 移行段階の到達状態（Level 3）

- GitHub Actions hosted runner ベースではハーメティック要件（ビルド中のネットワーク隔離）を完全には満たせないため、以下の 2 経路のいずれかに移行
  - **経路 α**: GitHub Actions + `actions/dependency-review-action` + `bazelbuild/rules_oci` の hermetic build setup
  - **経路 β**: 自前の isolated runner（Kubernetes 内で kaniko / buildkit を rootless + egress 遮断で駆動）
- リリース後の運用期間中に両経路の PoC を並行実施し、Hermetic Build 移行段階の開始時点で採用経路を別 ADR（ADR-SUP-002）で確定
- provenance の非否認性強化として、Rekor（Sigstore 公開透明性ログ）への登録を必須化。自社内のイミュータブル台帳（S3 object lock / MinIO versioned bucket）にもコピー保管
- SBOM は CycloneDX + SPDX 双方を生成して冗長化、SBOM diff を PR レビュー必須項目に昇格

### Level 2 → Level 3 移行戦略

リリース後の運用期間中に以下の準備を並行で進め、Hermetic Build 移行段階の開始時点で移行判定を行う。

- hermetic ビルド経路 α / β の PoC を `99_壁打ち/` 配下で評価、ビルド時間・再現性・運用負荷の 3 軸で定量比較
- リリース時点の Level 2 構成で蓄積した provenance attestation の再現性（同一 git ref / 同一依存で再ビルドした際の digest 一致率）を週次サンプリングで計測
- 不一致が発生した場合の原因（非決定的タイムスタンプ・ビルド環境差・依存バージョン漂流）を分類し、Level 3 移行前に Runbook `RB-SUP-001: ビルド再現性逸脱対応` を整備

Hermetic Build 移行段階の開始時点で Level 3 到達判定会議を Product Council 経由で実施し、PoC 結果に基づく採用経路の最終確定を別 ADR（ADR-SUP-002）で記録する。判定が遅延した場合でも Level 2 構成は継続して成立し、監査証跡が途切れないことを保証する。

### Forensics Runbook（image hash 逆引き）

インシデント発生時に「この image hash は tier1 のどの公開 11 API に影響するか」を数分で逆引きできる索引を用意する。索引の実装要件は以下。

- **入力**: OCI image digest（`sha256:...`）
- **出力**: 影響する tier1 公開 API のリスト（state / pubsub / serviceinvoke / secrets / binding / workflow / log / telemetry / decision / audit / feature のうち該当するもの）、ビルド日時、ソース git commit、SBOM 差分、provenance 検証結果
- **データ源**: cosign triangle（OCI registry に格納された signature / attestation / SBOM）を起点に、SBOM 内の package 依存をたどって k1s0 の tier1 コードとの関係を再構成
- **実装**: `ops/scripts/forensics-lookup.sh` を整備し、`cosign tree` / `cosign verify-attestation` / `syft diff` を組み合わせる
- **Runbook**: `RB-SEC-005: Image Hash 逆引き Forensics` を Runbook 目録（`docs/04_概要設計/55_運用ライフサイクル方式設計/09_Runbook目録方式.md`）に追加

### cosign keyless の前提

- sigstore 公開 Fulcio / Rekor に依存するため、公共 sigstore のダウン時は署名発行が停止する
- BCP 対応として、Hermetic Build 移行段階で sigstore の OSS コンポーネント（fulcio / rekor / trillian）の self-host PoC を実施し、その後 self-host 自社運用に切り替える経路を別 ADR で管理する
- 公共 sigstore 運用中も署名対象のメタデータ（image digest / signer identity / timestamp）は自社側にミラー保管し、Rekor がダウンしても検証経路を喪失しない構成を取る

## 検討した選択肢

### 選択肢 A: リリース時点で Level 2、Hermetic Build 移行段階で Level 3（採用）

- 概要: k1s0 リリース時点で Level 2 を達成、運用蓄積後の Hermetic Build 移行段階で Level 3 へ到達
- メリット:
  - 採用検討の「改ざん耐性を示せるか」に Level 2 到達で即答可能
  - ハーメティックビルドの整備をリリース後の運用期間中に PoC で詰め、運用蓄積後に Level 3 に昇格するため無理がない
  - cosign keyless により初期の鍵管理コスト・鍵ローテーション Runbook 整備を後送りにできる
  - Rekor による公開透明性ログで第三者検証性が自動的に得られる
- デメリット:
  - Level 2 → Level 3 昇格の運用断面で、Kyverno ポリシー強化・Runbook 追加の移行コストが発生
  - 公共 sigstore への依存が Hermetic Build 移行段階まで残る（BCP 計画を別途整備）

### 選択肢 B: リリース時点で Level 3 を完備

- 概要: リリース時点でハーメティックビルド・再現可能ビルド・強い非否認性を完備
- メリット: 段階移行のオーバヘッドがない、監査対応が最強
- デメリット:
  - 個人開発 OSS のリリース時点で hermetic runner 整備と tier 別ビルド再現性確保は運用負荷過大
  - 不確実性の高い取り組みをリリースのクリティカルパスに置くリスクが大きい
  - 採用検討の応答を遅延させる原因になりうる

### 選択肢 C: Level 1 のみ到達、将来検討

- 概要: ビルド自動化 + プロビナンス文書化のみで採用検討の通過を狙う
- メリット: 即時対応が可能、追加投資ほぼ不要
- デメリット:
  - 採用側組織の情報セキュリティ監査（J-SOX IT 統制含む）で通らない可能性が高い
  - 採用側監査部門から「Level 1 では SolarWinds 級事件を防げない」と指摘された際に反論できない
  - 業界標準から外れることで事業継続上のレピュテーションリスク

### 選択肢 D: SLSA 不採用、独自メトリクスで代替

- 概要: in-toto 準拠の独自 provenance 形式、独自の検証ツールを作る
- メリット: 自社要件に完全フィット
- デメリット:
  - 第三者検証可能性を失う（監査対応で不利）
  - 運用・ツール保守コストが恒常的に残る
  - 業界標準から外れ、10 年保守で負債化

## 帰結

### ポジティブな帰結

- 採用検討に対し「SLSA Level 2 到達済み、Level 3 到達計画を保有」という明示的な回答が可能
- cosign keyless + CycloneDX + Kyverno ImageVerify の 3 点セットにより、本番 namespace での改ざん検知が構造的に担保される
- Rekor 登録で第三者検証可能性が自動的に得られ、採用側監査・FedRAMP 類似審査への耐性が向上
- image hash 逆引き Forensics により、インシデント時の「影響範囲特定」が分単位で行える
- 10 年保守の途中でサプライチェーン事件が発生しても、事前整備により RTO を最小化できる

### ネガティブな帰結

- Hermetic Build 移行段階での Level 3 昇格時に hermetic runner の運用負荷（ビルド時間増・依存事前解決の整備）が発生
- cosign keyless は公共 sigstore 依存であり、ダウン時の BCP を別途整備する必要がある（self-host 移行を将来計画）
- SBOM 差分レビューが PR レビュー工程に追加され、レビュー時間が 5〜10 分程度増える
- sigstore ecosystem 自体の脆弱性が発見された場合の影響評価が必要（依存対象として Renovate 監視に載せる）

### 移行・対応事項

- `tools/ci/slsa-provenance.yml` ワークフローをリリース時点で配置、`slsa-github-generator` の呼び出しを tier 別に定義
- `tools/ci/sbom-cyclonedx.yml` ワークフローで `syft` による SBOM 生成と `cosign attest` の流し込みを実装
- Kyverno ClusterPolicy `verify-image-signature` / `verify-provenance` / `verify-sbom` の 3 本を本番 namespace 向けに enforce 適用
- `ops/scripts/forensics-lookup.sh` および `RB-SEC-005: Image Hash 逆引き Forensics` Runbook を整備
- リリース後の運用期間中に hermetic runner の 2 経路 PoC（経路 α / 経路 β）を `99_壁打ち/` で評価、採用経路を ADR-SUP-002 で確定
- Rekor public instance のバックアップミラーを MinIO versioned bucket（[ADR-DATA-003](ADR-DATA-003-minio.md)）に保管
- `docs/05_実装/80_サプライチェーン設計/` 配下に SLSA Level 2/3 到達の実装指針を展開
- BC-SC-001〜007 系の要件定義との双方向トレースを `docs/04_概要設計/80_トレーサビリティ/02_要件から設計へのマトリクス.md` に追加
- Renovate（[ADR-DEP-001](ADR-DEP-001-renovate-central.md)）監視対象に cosign / syft / slsa-github-generator を含める

## 参考資料

- [ADR-CICD-003: Kyverno 採用](ADR-CICD-003-kyverno.md)
- [ADR-DEP-001: 依存更新中枢に Renovate を採用](ADR-DEP-001-renovate-central.md)
- [ADR-DATA-003: オブジェクトストレージに MinIO を採用](ADR-DATA-003-minio.md)
- [CLAUDE.md](../../../CLAUDE.md)
- SLSA v1.1 仕様: [slsa.dev/spec/v1.1](https://slsa.dev/spec/v1.1)
- Sigstore 公式: [sigstore.dev](https://sigstore.dev)
- cosign keyless signing: [docs.sigstore.dev/cosign/signing/overview](https://docs.sigstore.dev/cosign/signing/overview)
- Rekor 透明性ログ仕様: [docs.sigstore.dev/rekor](https://docs.sigstore.dev/rekor)
- CycloneDX v1.6 仕様: [cyclonedx.org/specification](https://cyclonedx.org/specification)
- SPDX v2.3 仕様: [spdx.github.io/spdx-spec](https://spdx.github.io/spdx-spec)
- slsa-github-generator: [github.com/slsa-framework/slsa-github-generator](https://github.com/slsa-framework/slsa-github-generator)
- syft SBOM generator: [github.com/anchore/syft](https://github.com/anchore/syft)
- OpenSSF "Supply-chain Attacks" 事例レビュー
- NIST SSDF (Secure Software Development Framework) SP 800-218
