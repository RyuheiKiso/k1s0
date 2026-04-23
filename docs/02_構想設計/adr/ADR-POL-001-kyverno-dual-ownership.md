# ADR-POL-001: Kyverno ポリシーを技術側提案・統制側承認の二分所有で運用

- ステータス: Accepted
- 起票日: 2026-04-24
- 決定日: 2026-04-24
- 起票者: kiso ryuhei
- 関係者: Platform/SRE / セキュリティチーム / 運用チーム / コンプライアンス担当 / EM

## コンテキスト

k1s0 は ADR-CICD-003 で Kyverno を admission ポリシーの標準として採用した。Kyverno は ClusterPolicy / Policy の CRD によって validate / mutate / generate の 3 種類のポリシーを宣言的に記述でき、NFR-H-INT-001（イメージ署名検証）や PSS restricted の強制、namespace 作成時の NetworkPolicy 自動生成などを admission 段階で統一的に扱える。ツール選定自体は決着したが、「誰がポリシーを書き、誰が承認し、誰が本番に適用するか」というガバナンス設計は未決定のまま Phase 1a に入る。

この所有権の未設計が残した場合、以下の病理が確実に発生する。

- **Security 単独所有モデル**: Security が全 Kyverno ポリシーを書き承認し適用する場合、ポリシーのドラフト時点で「Platform/SRE が欲しい運用補助（label 自動付与・リソースクォータ自動補完）」と「Security が欲しい統制（特権禁止・署名検証）」が同一ワークフロー上で競合する。Security は統制を優先し運用補助を後回しにするため、Platform 側は shadow に近い運用スクリプトで補完するようになり、admission ポリシーと運用スクリプトが二重に存在する状態に陥る。
- **Platform 単独所有モデル**: Platform/SRE が全 Kyverno ポリシーを書き本番適用する場合、validate ポリシー（特権禁止・署名検証・PSS restricted）が「運用の都合」で緩和されるインシデントが必ず起きる。J-SOX や NFR-H-COMP-002 の監査対応で「誰が統制を承認したか」を問われた際、Platform = 技術実装側が自身で統制を定めたことになり、職務分掌上の説明責任が成立しない。
- **全員レビューモデル**: Platform・Security・運用・EM の全員が全ポリシーをレビューする場合、1 ポリシーあたりのリードタイムが週単位に延び、緊急パッチ（CVE 対応など）に間に合わない。2 名フェーズでは実質的に全員が当事者となり、相互承認が成立しない。

この問題は「validate / mutate / generate の 3 種類が性質として異なる」という観察から解ける。validate は統制の表現であり、mutate / generate は運用の自動化である。両者を同じ承認フローに乗せること自体が設計ミスであり、二種類の承認経路を用意すべきである。OPA / Gatekeeper を採用している組織（Goldman Sachs、Capital One 等の公開事例）でも、Policy-as-Code のガバナンスは「統制系ポリシー」と「運用補助系ポリシー」を別所有にするパターンに収斂している。

k1s0 は 2 名運用フェーズで Platform と Security が兼任に近い状態で始動するが、10 年保守の間に両ロールは必ず分離する。分離した時点で「ポリシー所有権」の前例が曖昧だと、組織分掌の再交渉が毎回発生する。Phase 0 の今、モデルを明文化しておくことが 10 年後の運用負荷を決定的に下げる。

JTC 固有の観点として、統制系意思決定に関する「職務分掌」は内部統制報告（J-SOX）・金融商品取引法・PCI-DSS 等の遵守対象となる。実装者と統制承認者が同一人物である場合、監査指摘の対象となる可能性が高い。本 ADR の二分所有モデルは、この職務分掌要件を admission ポリシーレベルで自然に成立させる意味でも合理的である。親会社・関連会社との間で開発委託が発生する 10 年保守の後半を想定しても、「統制は Security、運用補助は Platform」という境界は契約上の責任分界点として機能する。

## 決定

**Kyverno ポリシーは技術側（Platform/SRE）提案・統制側（Security）承認の二分所有モデルで運用する。**

- **validate ポリシー（統制系）**: Security が最終承認権限を持つ。起案自体は Platform/SRE が行ってよいが、PR merge には Security のレビュアー承認が必須。対象は PSS restricted、verifyImages（Sigstore cosign 署名検証）、resource limits 必須化、特権コンテナ禁止、host namespace 禁止、監査対象 label 強制など、「違反すれば統制違反」となるすべてのポリシー。
- **mutate / generate ポリシー（運用補助系）**: Platform/SRE が主導する。起案・実装・PR merge は Platform/SRE 側で完結するが、Security レビューを通過させる。Security は「このミューテーションが統制を迂回しないか」の観点のみをレビューし、運用上の妥当性は判断しない。対象は label 自動付与、namespace 作成時の NetworkPolicy / ResourceQuota 自動生成、欠落 annotation 補完など。
- **ポリシー改廃 = ADR 起票**: ポリシーの新設・破壊的変更・廃止は ADR 起票と同格のプロセスで扱う。PR に ADR 番号を記載し、変更理由・影響範囲・ロールバック手順を明示する。audit → enforce 昇格も同プロセスに従う。ADR-CICD-003 の「audit モードで 1 週間監視 → enforce 昇格」プロセスは本 ADR で強制化される。
- **緊急パッチ例外**: CVE 対応・本番インシデント収束など 24 時間以内の対処が必要な場合は、Platform/SRE の判断で enforce を一時適用できる。ただし 72 時間以内に事後 ADR を起票し、Security 承認を取得する。未提出の場合は自動的に元に戻す CI ジョブを設置する。
- **Kyverno 本体のアップグレード**: Kyverno コンポーネント（Controller / Admission Webhook）のバージョン更新も本モデルに従う。バージョン更新は validate ポリシーの挙動変化を伴う可能性があるため、Security 承認対象。マイナーバージョン以上の更新は ADR 化必須、パッチバージョンは Runbook 記録のみ。
- **Policy Exception の取扱い**: Kyverno Policy Exception も本モデルに従う。Exception の起票は Platform/SRE が行い、Security が承認する。Exception の有効期限は最大 90 日とし、延長は再申請扱い。
- **監査証跡**: 全ポリシー変更は GitOps（ADR-CICD-001 ArgoCD）経由で反映されるため、PR の merge 記録・ArgoCD の sync 履歴が一次証跡となる。Policy Report を Loki（ADR-OBS-001）に連携し、admission 段階の違反・許可を恒久保存する。

### validate / mutate の境界判断原則

境界が曖昧なポリシーは実運用で頻出する。例えば「Pod に `team` label が無ければ admission 拒否する validate」と「Pod に `team` label を自動付与する mutate」は、観測上は同じ目的を達成する。本 ADR では以下の原則で境界を判断する。

- **拒否すべき対象**: 「この構成は作ってはいけない」という表現は validate に寄せる。外部監査で「なぜこの操作が通ったのか」を問われた際、明示的な拒否の記録が残るため説明責任が成立する。
- **補完すべき対象**: 「この情報は省略してもよいが、書くと便利」という表現は mutate に寄せる。ただし mutate で補完した値が後段の validate で検証される構成を推奨する。
- **両方が必要なケース**: validate + mutate の組合せは Security と Platform の両方のレビューを通す。PR テンプレートで「validate 側 / mutate 側のそれぞれで何が起きるか」を明示する欄を設ける。

### audit → enforce 昇格の二段階適用

ADR-CICD-003 で規定された「audit モードで 1 週間監視 → enforce 昇格」プロセスは、本 ADR で「ポリシー所有モデルに従った昇格承認」として再定義される。validate 系ポリシーの audit → enforce 昇格は Security の承認を要し、mutate/generate 系は Platform/SRE の判断で昇格できる（ただし Security レビューは事後でも実施）。昇格判断の根拠として、audit 期間中の違反件数・影響 namespace 数・想定される本番影響の 3 点を PR 本文に記載する。

### スコープ

本 ADR は Kyverno ポリシーの所有権設計に限定する。同じ思想は将来的に OPA / Gatekeeper や Falco ルールに拡張可能だが、それらは別 ADR で扱う。ネットワークポリシー（Istio AuthorizationPolicy、Cilium NetworkPolicy）は admission ではなく runtime 統制であり、別のガバナンスに従う。

## 検討した選択肢

### 選択肢 A: 二分所有モデル（採用）

- 概要: validate は Security 承認、mutate/generate は Platform/SRE 主導で Security レビュー、改廃は ADR 同格プロセス
- メリット:
  - 統制系と運用補助系の承認経路が分離し、リードタイムが本質的に異なる性質を反映できる
  - Security の承認権限が validate に集中し、職務分掌上の説明責任が成立（J-SOX 対応）
  - Platform/SRE が運用補助ポリシーを自律的に改善でき、DX が阻害されない
  - 緊急パッチ例外が 72 時間で事後 ADR 化されるため、統制の迂回が恒久化しない
  - 10 年保守で Platform / Security が完全分離した後も同じモデルが機能する
- メリット:
  - ポリシー種別ごとに承認経路を判断するコストが発生（新設時にカテゴリ判断が必要）
  - validate と mutate の境界が曖昧なケース（例: mutate で label を強制付与して validate で検証する組合せ）で両ロールの調整が必要

### 選択肢 B: Security 単独所有モデル

- 概要: 全 Kyverno ポリシーの起案・承認・適用を Security が行う
- メリット: 統制の一貫性、承認経路が単一
- デメリット:
  - mutate / generate の運用補助改善が Security の工数で律速される
  - Platform 側が shadow スクリプトで補完するようになり、admission と runtime の統制が二重化
  - 10 名フェーズで Security 1 名体制では明らかに詰まる
  - 運用補助ポリシーの微調整（label 名変更等）にも Security レビューが必要となり非効率

### 選択肢 C: Platform 単独所有モデル

- 概要: 全 Kyverno ポリシーを Platform/SRE が所有、Security は事後監査のみ
- メリット: 実装ペースが最速
- デメリット:
  - validate ポリシー（統制）を実装側が自作自演で決定することになり、職務分掌が破綻
  - J-SOX / NFR-H-COMP-002 の監査で「統制承認者」が存在しないと指摘される
  - 運用都合での validate 緩和が発生しやすく、インシデント時の責任所在が曖昧化
  - 10 年保守の後半で経営層から統制強化要求が来た際、再分離のコストが巨大化

### 選択肢 D: 全員レビューモデル

- 概要: Platform・Security・運用・EM の全員が全ポリシーをレビュー・承認
- メリット: 透明性が高い、合意形成が確実
- デメリット:
  - 2 名フェーズでは全員が当事者となり、独立レビュアーが存在しない
  - 10 名フェーズでもリードタイムが週単位になり、緊急パッチに間に合わない
  - 責任の拡散により意思決定が形骸化する（全員責任 = 誰も責任を取らない）
  - Kyverno ポリシーの変更頻度（Phase 1c で月数十件を想定）に対して運用不能

### 選択肢 E: 三分所有（validate / mutate / generate を個別所有）

- 概要: generate を運用チーム、mutate を Platform、validate を Security と三分する
- メリット: 責務がさらに明確化
- デメリット:
  - 2 名フェーズで 3 ロールの独立運用が不可能
  - generate と mutate の境界が曖昧なケース（例: namespace 生成時に label 付与）で 2 ロールをまたぐ調整が頻発
  - ロール数の増加が承認リードタイムを線形に悪化させる
  - 10 年保守で運用チームが分離・再編される際、generate 所有権の移管コストが巨大化

## 帰結

### ポジティブな帰結

- validate ポリシーの Security 承認が明文化され、J-SOX / NFR-H-COMP-002 の監査証跡が GitOps 履歴として恒久保存される
- mutate / generate の改善サイクルが Platform/SRE の裁量で回り、DX 改善が阻害されない
- 緊急パッチ 72 時間事後 ADR 化ルールにより、統制迂回の恒久化が構造的に防がれる
- Kyverno 本体のアップグレードが Security レビュー対象となり、ポリシー挙動の非互換変更を事前検知できる
- 2 名フェーズから 10 名フェーズへの拡大時に、所有権の再交渉が不要（同じモデルがスケールする）
- 親会社・関連会社への開発委託が発生した際、「統制は Security、運用補助は Platform」が契約上の責任分界点として機能する
- Policy Exception の 90 日上限ルールにより、例外が恒久化する技術負債を構造的に排除

### ネガティブな帰結

- validate / mutate の境界が曖昧なポリシーで両ロールの調整コストが発生
- 緊急パッチ事後 ADR 化の未提出を検知する CI ジョブの実装・保守が必要
- Security のレビュアー不在時（長期休暇等）のバックアップ体制が必要（最低 2 名体制の確保）
- Policy Exception の 90 日上限が運用実態に合わないケースで再申請が負荷になる可能性

### 移行・対応事項

- CODEOWNERS に Kyverno ポリシーディレクトリ（`infra/security/kyverno/policies/validate/` 等）の所有権を記載し、validate は Security、mutate/generate は Platform/SRE を必須レビュアーに設定
- PR テンプレートに「ポリシー種別」「ADR 番号」「audit/enforce」「ロールバック手順」のチェック欄を追加
- 緊急パッチ例外の事後 ADR 未提出検知 CI ジョブを `tools/ci/` に実装
- Policy Exception の 90 日期限切れを監視する CronJob を `infra/security/kyverno/` に配置
- Security レビュアーのバックアップ体制（最低 2 名）を Phase 1a で確保
- validate / mutate の境界判断ガイドを [../../../docs/05_実装/](../../../docs/05_実装/) 配下に配置
- Phase 1b 終了時点でポリシー改廃件数・リードタイム・例外事後 ADR 率を PMC でレビューし、モデルの実効性を検証
- Paved Road（ADR-DEV-001）の Scaffold 生成物が validate ポリシーに初期状態で適合するよう、Scaffold テンプレートと Kyverno ポリシーの両方を参照する CI ジョブを設置
- 新規参加者向けの「Kyverno ポリシー改廃フロー」研修資料を Phase 1a で整備し、所有モデルの理解を制度として根付かせる

## 参考資料

- [ADR-CICD-003: ポリシー適用に Kyverno を採用](ADR-CICD-003-kyverno.md)
- [ADR-CICD-001: GitOps に ArgoCD を採用](ADR-CICD-001-argocd.md)
- [ADR-OBS-001: 観測性基盤に Grafana LGTM を採用](ADR-OBS-001-grafana-lgtm.md)
- [ADR-SEC-001: IdP に Keycloak を採用](ADR-SEC-001-keycloak.md)
- [CLAUDE.md](../../../CLAUDE.md)
- Kyverno 公式: [kyverno.io](https://kyverno.io)
- CNCF Policy-as-Code Working Group 資料
- "Policy as Code: An Introduction" (Open Policy Agent Blog)
- NIST SP 800-53 Rev.5 AC-5 Separation of Duties
- J-SOX 実施基準 II-3-(3) 統制活動の職務分掌
- Capital One Tech Blog: "Policy as Code Governance at Scale"
