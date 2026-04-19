# OPS-CID: CI/CD 要件

本ファイルは、k1s0 の全リポジトリ（tier1 自作領域・tier2 サンプル・tier3 アプリ配信ポータル・IaC）で適用される **CI/CD パイプラインのゲート通過条件と承認フロー** を要件化する。リリース時のトラフィック切替やロールバックは [`02_release.md`](./02_release.md) に分離する。

構想設計 [`../../02_構想設計/04_CICDと配信/00_CICDパイプライン.md`](../../02_構想設計/04_CICDと配信/00_CICDパイプライン.md) では「GHA + Harbor + Trivy + Argo CD」の組み合わせを既に選定済み。本ファイルはその選定の上で、**どの検査が PR に必須か / どのブランチにどの保護を掛けるか / 承認は何名か** を ID 単位で固定する。

---

## 前提

- [`../../02_構想設計/04_CICDと配信/00_CICDパイプライン.md`](../../02_構想設計/04_CICDと配信/00_CICDパイプライン.md) — パイプライン構成の選定根拠
- [`../30_セキュリティ_データ/08_artifact_integrity.md`](../30_セキュリティ_データ/08_artifact_integrity.md) — イメージ署名 / SBOM 要件
- [`../30_セキュリティ_データ/00_security.md`](../30_セキュリティ_データ/00_security.md) — CVE 対応の SLA
- [`../50_開発者体験/00_test.md`](../50_開発者体験/00_test.md) — テストカバレッジ要件

---

## 要件本体

### OPS-CID-001: 全 PR で 6 種検査必達（Lint/Test/SBOM/CVE/ライセンス/コミット規約）

- 優先度: MUST（OSS 採用が前提の k1s0 でライセンス検査を欠くと、GPL 汚染が tier1 に混入し配布不可になる）
- Phase: Phase 1a（最小構成でも Lint/Test は必須）/ Phase 1b で SBOM/CVE/ライセンス/コミット規約を追加
- 関連: SEC-ART-001（SBOM）/ SEC-CVE-001（CVE 対応 SLA）/ BIZ-LIC-001（ライセンス適合）

現状、PR レビューは人間の目視のみで、Lint エラー・テスト失敗・CVE 混入・GPL 依存の追加・意味不明なコミットメッセージが main ブランチに入り込む余地がある。tier1 自作領域が 4 言語（Go / Rust / TypeScript / C#）にまたがるため、属人レビューでは品質が均一にならない。

要件達成後の世界では、全 PR に対して GHA ワークフロー `pr-check.yml` が 6 種の検査を並列実行する。(1) 言語別 Lint（rustfmt/clippy, gofmt/golangci-lint, eslint, dotnet format）、(2) ユニットテスト + カバレッジ 70% ガード、(3) Syft による SBOM（SPDX 形式）生成、(4) Trivy FS による CVE スキャン（CRITICAL で fail）、(5) `license-finder` によるライセンス分類（GPL/AGPL で fail、MPL/LGPL は警告）、(6) Conventional Commits 準拠（`commitlint`）。いずれかが fail すれば merge ボタンが GitHub UI 上でグレーアウトする。

崩れた時、たとえば GPL 依存の OSS が tier1 Rust クレートに merge されると、k1s0 全体の配布条件が変わり、法務対応だけで 3 か月の稟議再審査が発生する。CVE 混入は「CVE Critical 48h 以内対応」という SLA 違反となり、監査指摘で Phase ゲートが閉じる。コミット規約違反は `git-cliff` の自動チェンジログ生成を壊し、OPS-REL-003（リリースノート自動化）が動作しない。

**受け入れ基準**

- 全リポジトリの `.github/workflows/pr-check.yml` に 6 種検査が定義され、雛形生成 CLI で自動配置
- 6 種検査のパイプライン実行時間 P95 が 30 分以内（PR 体験を阻害しない上限）
- CRITICAL CVE / GPL・AGPL ライセンス / 70% 未満カバレッジは PR merge をブロック
- コミット規約違反は PR タイトル＋全 commit に対して検査、squash merge 時のコミットメッセージも対象
- 検査結果は GitHub Checks API と Backstage Scorecard の両方に 10 秒以内で反映

**検証方法**

- 月次で「検査 fail のまま main に入った PR 数」を集計、0 件を維持
- 四半期ごとに意図的に GPL 依存を PR に混ぜる抜き打ちテストを実施し、ブロックされることを確認
- パイプライン P95 を Grafana `gha-pr-check-latency` で週次レビュー

---

### OPS-CID-002: main ブランチ保護（2 reviewers + 署名必須 + 管理者例外禁止）

- 優先度: MUST（監査ログの改ざん耐性は「誰も 1 人では main を変更できない」構造で担保される）
- Phase: Phase 1a（tier1 自作領域のみ）/ Phase 1b で全リポジトリ
- 関連: SEC-AUD-003（変更履歴の改ざん耐性）/ OPS-CID-004

現状、tier1 リポの main 保護は CODEOWNERS のみで、起案者が自己承認できるバイパス経路が残る。commit 署名も任意となっており、偽造コミットを検知する仕組みがない。

要件達成後の世界では、GitHub リポジトリの `Branch protection rules` で main に対して以下を強制する。(1) PR 必須、(2) **承認レビュー 2 名** かつ CODEOWNERS 経由、(3) `Require signed commits` で GPG/SSH 署名必須、(4) 全ステータスチェック（OPS-CID-001 の 6 種）pass、(5) `Include administrators` をオンにして管理者でもバイパス不可、(6) force push 禁止、(7) 削除禁止。署名鍵は Keycloak OIDC でマッピングされた SSH 鍵のみを許可し、Backstage の `catalog-info.yaml` と一致する開発者のみがコミット可能。

崩れた時、起案者が深夜に独断で main を変更でき、SEC-AUD が要求する監査ログの改ざん耐性が成立しない。Phase 2 の SOX 対応監査で「2 者責任分離が担保されていない」と指摘され、契約顧客（tier3 エンドユーザー）への k1s0 提供が法務ブロックされる。

**受け入れ基準**

- 全リポジトリの main ブランチ保護が Terraform `github-branch-protection` モジュールで宣言管理
- 承認レビュー 2 名（起案者除外）、署名必須、管理者含む全員にバイパス不可
- force push / branch delete が過去 90 日で 0 件（GitHub Audit Log で確認）
- 緊急パッチ時のバイパス手順は「CAB（Change Advisory Board）の書面承認 + 4h 後の事後 PR」の 2 段階に限定
- 保護設定の変更は Terraform PR を経由し、組織 Admin 2 名の承認必須

**検証方法**

- 四半期ごとに GitHub Audit Log から force push / 直 push / bypass 件数を SRE リードが監査
- 年次 SOX 監査で「2 者責任分離」の証跡として Branch protection rules のスクリーンショットを提出

---

### OPS-CID-003: Argo CD GitOps による宣言的デプロイ

- 優先度: MUST（手動 kubectl apply を許すと、本番構成とリポジトリが乖離し、ロールバック不能になる）
- Phase: Phase 1a（tier1 のみ）/ Phase 1b で tier2/3 展開
- 関連: OPS-ENV-002 / OPS-REL-001

現状、開発環境では手動 `kubectl apply` が許容されており、本番との構成差分が Git から見えない。障害時に「本番で動いていた設定」を特定するのに 30 分以上かかる。

要件達成後の世界では、全環境（dev/stg/prod）の k8s リソースは GitOps リポジトリ `k1s0/gitops` のマニフェストのみをソース・オブ・トゥルースとし、Argo CD が 3 分間隔で同期する。`ApplicationSet` でテナント × tier × 環境の組み合わせを宣言的に生成し、tier2/3 サービスの追加は catalog-info.yaml の追記 + ApplicationSet テンプレートの展開で自動化する。緊急時の `kubectl apply` は Kyverno の `audit` ポリシーで検知・Slack 通知され、24 時間以内に GitOps リポに反映されない場合は Argo CD が自動的に上書き同期する（`selfHeal: true`）。

崩れた時、本番構成が Git から乖離したまま障害が発生すると、ロールバック先が存在せず MTTR が倍増する。また、監査時に「本番で動いている全マニフェストを提出せよ」と問われた際、Git 履歴だけでは答えられず、監査不適合となる。

**受け入れ基準**

- 本番環境の全 k8s リソースの `argocd.argoproj.io/tracking-id` アノテーションが 100%（Argo CD 管理外 0 件）
- Argo CD Application の `syncPolicy.automated.selfHeal` が prod/stg で有効
- GitOps リポから本番反映までの時間 P95 が 5 分以内（push → Argo Events → sync 完了）
- 手動 kubectl apply は Kyverno audit で検知され、過去 90 日の件数が月 5 件以下
- Argo CD Application 数と catalog-info.yaml のサービス数が日次で整合（差分 0）

**検証方法**

- Argo CD ダッシュボード `argocd-app-coverage` で trackingId カバレッジを日次監視
- 四半期ごとに prod の `kubectl diff` を実行し、Git との差分 0 を確認

---

### OPS-CID-004: 本番リリースの 2 段承認と緊急パッチ例外

- 優先度: MUST（本番の構成変更は 2 者承認が SOX 要件。例外経路を事前定義しないと深夜障害で即応できない）
- Phase: Phase 1c（本番稼働開始時）
- 関連: OPS-CID-002 / OPS-REL-002 / OPS-INC-002

現状、本番向けのリリース承認は GitOps リポの PR レビュー 1 名で通過するため、個人の判断で本番にデプロイが流れる。深夜の緊急パッチ時の承認経路は未定義で、「誰が承認するのか」を毎回調整している。

要件達成後の世界では、本番（prod）向けの GitOps PR は **2 段階承認** を要する。第 1 段は tier1 リード（または代理 2 名）の技術レビュー、第 2 段はプロダクトオーナーまたは SRE リードの事業影響レビュー。両方の Approve が揃って初めて Argo CD `prod` プロジェクトへの sync が解禁される（Kyverno ポリシーで 2 Approve 未満はブロック）。緊急パッチ（Sev1/Sev2 のホットフィックス）は**インシデントコマンダーが 1 人 Approve で暫定承認**し、4 時間以内に事後 PR で通常の 2 段承認を取得する「事後追認」経路を許容する。

崩れた時、単一承認で本番に変更が入ると、SOX 監査で「本番変更の 2 者責任分離が不十分」と指摘され、上場会社の情シス部門として法令違反となる。緊急パッチ経路が未定義だと、Sev1 発生時に承認者探しに 30 分以上要し、OPS-INC-002（Sev1 初期応答 15 分）を守れない。

**受け入れ基準**

- prod GitOps PR は CODEOWNERS 2 グループ（tier1-leads + product-owners）から各 1 名以上の Approve が必須
- Kyverno ポリシー `prod-two-approval` が本番 Application の sync をブロック（単一承認時）
- 緊急パッチ経路は Runbook `emergency-hotfix.md` に手順化、事後 PR は 4h 以内 100%
- 承認の Audit Log は GitHub Audit Log + Argo CD History の両方に 365 日保管
- 四半期の承認リードタイム P95 が 4 時間以内（業務影響が軽微なパッチの迅速化）

**検証方法**

- 月次で prod PR の承認者数を集計し、2 名未満で merge された PR が 0 件
- 四半期ごとに緊急パッチの事後 PR 遵守率（4h 以内）をレビュー

---

### OPS-CID-005: パイプライン実行時間 P95 30 分以内

- 優先度: SHOULD（30 分を超えるパイプラインは開発者の PR 待ちが発生、DX 低下で tier2 リリース速度が落ちる）
- Phase: Phase 1b（tier1 のみ 30 分）/ Phase 2 で全リポ 30 分
- 関連: DEV-DX-002 / OPS-CID-001

現状、パイプラインのキャッシュ最適化が未実施で、Rust のフルビルド + Trivy スキャンで 50 分を要する PR もある。レビュアは PR を開いてから 1 時間待つ体験となり、DX が低い。

要件達成後の世界では、GHA self-hosted runner のキャッシュを `actions/cache` + Rust `sccache` + Trivy DB オフラインミラーで整備し、差分ビルド時のパイプライン実行時間 P95 を 30 分以内に圧縮する。キャッシュヒット率 90% 以上を維持し、ヒット率が落ちたリポはキャッシュ戦略の再設計を SRE が主導する。

崩れた時、PR 待ち時間が開発者 1 人あたり 1 日 2〜3 時間に達し、tier2 サンプル実装の進捗が 30% 低下する（業界の DX 調査で PR 待ち時間 1h 超は生産性 25-35% 低下）。

**受け入れ基準**

- 全リポのパイプライン P95 実行時間を Prometheus `gha_workflow_duration_seconds` で観測
- P95 > 30 分のリポは月次レビューで原因分析レポート提出
- キャッシュヒット率が 90% 以上（`actions/cache` の `cache-hit` outputs で集計）
- パイプラインの並列ステップ数 4 以上（Lint/Test/Build/Scan を並列実行）
- 雛形生成 CLI が並列化済みのワークフローを生成

**検証方法**

- Grafana ダッシュボード `gha-pipeline-performance` で週次レビュー
- 新規リポは雛形生成 CLI 経由でのみ作成し、手書きワークフローは CI で検知・警告

---

## 章末サマリ

### ID 一覧

| ID | タイトル | 優先度 | Phase |
|---|---|---|---|
| OPS-CID-001 | 全 PR で 6 種検査必達 | MUST | 1a/1b |
| OPS-CID-002 | main ブランチ保護 | MUST | 1a/1b |
| OPS-CID-003 | Argo CD GitOps デプロイ | MUST | 1a/1b |
| OPS-CID-004 | 本番 2 段承認と緊急パッチ | MUST | 1c |
| OPS-CID-005 | パイプライン P95 30 分 | SHOULD | 1b/2 |

### 優先度分布

| 優先度 | 件数 | 代表 ID |
|---|---|---|
| MUST | 4 | OPS-CID-001, 002, 003, 004 |
| SHOULD | 1 | OPS-CID-005 |

### Phase 達成度

| Phase | 必達件数 | 未達影響 |
|---|---|---|
| 1a | 3 | tier1 開発開始不可、GPL 混入リスク |
| 1b | 4 | 全リポ展開時のライセンス・CVE 管理が崩壊 |
| 1c | 5 | 本番稼働の SOX 適合不可、業務稼働 NG |
