# ADR-DEV-001: 開発者体験の根幹思想として Paved Road を採用

- ステータス: Accepted
- 起票日: 2026-04-24
- 決定日: 2026-04-24
- 起票者: kiso ryuhei
- 関係者: 全開発チーム / DevEx チーム / Platform/SRE / Product Council / EM

## コンテキスト

k1s0 は採用側の運用体制が小規模から拡大することを想定する 採用側組織向け PaaS であり、10 年の保守期間を前提とする。tier1〜tier3 と SDK・infra・deploy・ops が集約されたモノレポ（[../../../CLAUDE.md](../../../CLAUDE.md) 参照）の上で、開発者は Protobuf 契約の追加・ドメインサービスの新設・UI 画面の追加といった日常作業を週次で繰り返す。これらの日常作業がセットアップから最初の commit まで数日を要する状態では、採用初期の小規模運用で立ち上げが失速し、運用拡大期にレビュー負荷が雪だるま式に膨張する。

この状態で頻発するのが以下の摩擦である。

- **雛形コピーの属人化**: 先行プロジェクトの src ツリーをコピーして Protobuf IDL・CI/CD・Dockerfile・観測設定・CODEOWNERS を手修正する文化が発生し、初期構成が開発者ごとに分岐する。結果として tier1 内部言語ハイブリッド（ADR-TIER1-001）や Protobuf 強制（ADR-TIER1-002）の規約遵守がレビュー段階まで先送りされる。
- **「正解がどこにあるか分からない」問題**: README / ADR / examples / Runbook / Backstage TechDocs にノウハウが散在し、新規参加者は「このケースの標準はどれか」を探すだけで半日を溶かす。10 年保守を想定する場合、5 年後の新規参加者が同じ探索を強いられる構造は維持不能である。
- **Golden Path 逸脱の検知遅延**: Kyverno（ADR-CICD-003）や Backstage（ADR-BS-001）による制御は runtime / catalog 段階の検知であり、開発者がローカルで逸脱構成を書いた瞬間を止める仕組みではない。逸脱が admission 段階で拒否された時点で既に数時間の手戻りが発生する。
- **支援コストの無限化**: 「このケースはどう書くのが正解か」という問い合わせを Platform/SRE が全件引き受けると、小規模運用下では Platform の本来業務（クラスタ運用・ポリシー整備）が停止する。採用側の運用拡大期では問い合わせ量が小規模運用期の 5 倍では済まず、Platform チームの瓦解が現実的リスクとなる。
- **Backstage Software Template と自作 Scaffold の二重化**: ADR-BS-001 で Backstage Software Template を採用済みだが、ローカル CLI での Scaffold も必要になる（Dev Container 内・オフライン環境・CI 経由生成など）。両者を別実装で整備すると、テンプレート定義が二重化し、片方の更新忘れが品質ばらつきの温床になる。

この摩擦は「テンプレート不足」ではなく「開発者体験の根幹思想が未定」の症状である。思想が決まらないまま雛形を増やすと、選択肢が増えるだけで摩擦は増す。Netflix が提唱した Paved Road（舗装道路）の考え方は、「舗装された道を歩く限り全支援を受けられるが、外れた瞬間に支援対象外となる」という明確な境界線を引くことで、支援コストを有限化しつつ自由度を残す実証済みのパターンである。Google の `blaze` + `rules_*`、Spotify の Backstage Golden Path、Microsoft の `dotnet new` テンプレートもすべて Paved Road の一変種として説明できる。

k1s0 リリース時点で開発者体験の根幹思想を確定させなければ、採用側が小規模運用で立ち上げる過程で「個々の流儀」が定着し、運用拡大期にそれが引き継がれて以後の再統一が極めて困難になる。思想決定はリリース時点で行うのが合理的である。

採用側組織の固有の観点も本 ADR の前提に含まれる。日本企業の内製開発では「標準化の強制」が組織文化と相性が悪く、過去の全社開発標準の多くが「厳格固定 → 誰も守らない → 形骸化」という経路で失敗してきた。Paved Road はこの失敗パターンに対して「強制ではなく経済的インセンティブ（支援コストの差）」で標準を維持するアプローチであり、採用側組織の組織文化にも適合しやすい。同時に 10 年保守の後半で開発責任が子会社・関連会社に移管されるケースを想定すると、「外れた経路は自己責任」の線引きが契約上の責任分界点としても機能する。

## 決定

**開発者体験の根幹思想として Netflix 方式の Paved Road を採用する。**

- **Golden Path の一本化**: 各ユースケース（tier2 ドメインサービス新設 / tier3 Web 画面追加 / SDK 追加言語対応 / Protobuf 契約追加）につき Paved Road を 1 本だけ定義する。Golden Path の一次ソースは `examples/` 配下の実稼働サンプルとし、`docs/` は解説のみを持つ。実コードと解説が二重化しない構成とする。
- **Scaffold CLI による配布**: Backstage Software Template（ADR-BS-001）互換の Scaffold CLI（`src/platform/scaffold/`）で `examples/` をコピー配布する。CLI は Backstage UI からも叩けるし、ローカルでも `scaffold new tier2-service --name=xxx` で叩ける。Scaffold は同じテンプレート定義を参照するため Backstage 経由と CLI 経由で生成物が一致する。
- **catalog-info.yaml の自動生成**: Scaffold は生成と同時に `catalog-info.yaml` を書き、PR を作る。PR マージ時点で Backstage カタログに即時登場する。catalog 登録の手作業は廃止する。
- **Paved Road を外れる自由**: Paved Road に収まらない特殊要件（legacy-wrap 連携、third_party OSS フォーク、実験的 crate 追加など）は禁止しない。ただし外れた経路については以下の意味で「自己責任」となる。
  - Platform/SRE の support コストは Paved Road 利用者にのみ適用する（問い合わせ優先度・SLA）
  - Paved Road 外の構成に起因する CI 失敗・本番障害の一次対応は起案チームが行う
  - Paved Road に戻す道筋（exit plan）を ADR として事前提出することを推奨する
- **DX SLI の設定**: time-to-first-commit（新規参加者が環境構築から最初の PR マージまでに要する時間）を DX の一次 SLI とする。リリース時点で 4 時間、運用拡大期に 2 時間を目標とする。計測は Backstage Scorecards（ADR-DX-001 連動）で行い、四半期ごとにレビューする。
- **Paved Road 利用率の目標**: 新規サービス起票時の Scaffold CLI 利用率を KPI とする。リリース後の早期段階で 80%、運用蓄積後に 95% を目標とする。未達が続く場合は「Paved Road が現場の実態に合っていない」兆候として Paved Road 側を見直す。

### examples/ を一次ソースとする理由

Golden Path の一次ソースを `docs/` ではなく `examples/` に置く設計は、「動くコードこそが最新の真実」という原則に基づく。docs 側に手順を書き下すと、コード変更とドキュメント変更の間に必ず時間差が生じ、数ヶ月後に乖離する。`examples/` を一次ソースにすれば、CI が examples をビルド・テストすることでドキュメント整合性が自動的に強制される。Scaffold CLI は examples をコピーする実装であるため、CI が壊れていない限り Scaffold 出力も壊れない。docs 側は「なぜこの構成なのか」「どこを改造してよいか」の解説のみを持ち、コードそのものは持たない。

### Paved Road と規約強制の関係

Paved Road は規約強制（ADR-CICD-003 Kyverno、ADR-POL-001 の二分所有モデル）と矛盾しない。両者の役割は階層的である。Paved Road は「正しく作ればレビューの大半を省ける入口」を提供し、規約強制は「入口を通らずに本番に到達しようとする経路を admission で止める」最終防衛線を担う。入口を整備すれば最終防衛線での拒否が減り、Security と Platform の両方の負荷が下がる。逆に入口を整備せず規約強制だけを強化すると、開発者は「どう書けば通るか」を admission エラーの試行錯誤で学ぶことになり、DX が崩壊する。

### スコープ

本 ADR は思想レベルの意思決定である。具体的な Template の内容、Scaffold CLI の実装詳細、catalog-info.yaml のスキーマは別 ADR（ADR-DEV-002 以降）および実装ドキュメント（`docs/05_実装/`）で扱う。

## 検討した選択肢

### 選択肢 A: Paved Road 一本化（採用）

- 概要: 上記の通り。思想を単一化し、Golden Path は `examples/` に一本化、Scaffold CLI で配布、逸脱は自己責任とする
- メリット:
  - 支援コストが有限化し、小規模運用でも Platform が本来業務を維持できる
  - 採用側の運用拡大期に新規参加者が迷う選択肢が原理的に存在しない
  - `examples/` が一次ソースのため、docs と実装の乖離が構造的に発生しない
  - Netflix / Google / Spotify の実証済みパターンであり、10 年保守で陳腐化しにくい
  - Paved Road を外れる自由を残すため、創造性と統制のバランスが取れる
  - Backstage Software Template と Scaffold CLI が同一テンプレート定義を参照するため、ADR-BS-001 の投資が実質的に活かされる
- デメリット:
  - Paved Road の初期整備に相応の準備期間を要する（リリース時点までに集中整備）
  - 「自己責任」の線引きが政治的に扱いにくいケースが発生しうる
  - Paved Road 自身が老朽化する恐れがあり、定期的な再整備が必要
  - Scaffold CLI と examples の同期を保つ CI 工数が恒常的に発生

### 選択肢 B: 複数テンプレート共存（方針分裂）

- 概要: tier2 向け・tier3 向け・実験用など複数の Golden Path を並立させる
- メリット: 各ユースケースに最適化できる、一見柔軟
- デメリット:
  - 「どれが正解か」が決まらないため属人化が復活する
  - Template 間の思想矛盾が発生し、横断リファクタリングが困難化
  - 小規模運用の保守コストが Template 数に比例して増加
  - 採用側の運用拡大期に「A チームは T1、B チームは T2」という局所最適化が進み、組織分裂を誘発する

### 選択肢 C: 自由主義（テンプレートなし）

- 概要: Golden Path を明示せず、開発者が README と ADR を読んで自分で組み立てる
- メリット: Platform の整備コストがゼロ、短期的には楽
- デメリット:
  - 新規参加者の学習コストが線形に増加、time-to-first-commit が数日オーダーになる
  - 初期構成が開発者ごとに分岐し、横断レビュー負荷が爆発する
  - Kyverno / CI による後段統制で弾くしかなく、手戻りが常態化する
  - 10 年保守で参加者が入れ替わるたびに品質が劣化する構造的欠陥を持つ

### 選択肢 D: 厳格固定（逸脱禁止）

- 概要: Paved Road 以外の構成を技術的・制度的に禁止する
- メリット: 規約遵守が機械的に保証される
- デメリット:
  - legacy-wrap（ADR-MIG-001）や third_party 連携など、本質的に Paved Road に収まらない要件が詰む
  - 新技術評価が不可能になり、10 年保守で技術負債化する
  - 「禁止」の運用コスト（例外申請ワークフロー）が支援コストを上回る逆転現象が起きる
  - 採用側組織の固有の組織文化で「厳格固定 → 形骸化」の失敗パターンを過去事例で確認済み

### 選択肢 E: Paved Road を docs のみで表現（Scaffold CLI 不採用）

- 概要: Golden Path を docs のテキスト・コードブロックとして表現し、開発者がコピペする
- メリット: 実装コストが最小
- デメリット:
  - コピペ過程で抜けや誤りが混入し、生成物が開発者ごとに差分を持つ
  - docs 更新と生成物反映の間に時間差が必ず発生
  - catalog-info.yaml の自動生成が成立せず、Backstage カタログの網羅率が開発者の徹底度に依存する
  - 採用側の運用拡大期に Scaffold CLI の不在がボトルネックとなるのが確実

## 帰結

### ポジティブな帰結

- time-to-first-commit の計測可能化により、DX 改善が仮説検証型で進められる
- Scaffold CLI と Backstage Software Template の二重メンテが避けられ、単一テンプレート定義から両経路が派生する
- catalog-info.yaml 自動生成により、Backstage Software Catalog の網羅率が構造的に 100% に保たれる
- Paved Road 逸脱を罰するのではなく「支援範囲外」と定義するため、創造性と統制のバランスが取れる
- 10 年後の新規参加者も `examples/` を見て同じ起点から開始でき、知識の世代交代が成立する
- Platform/SRE の問い合わせ対応コストが Paved Road 利用者に集中し、工数見積りが安定する
- Kyverno（ADR-CICD-003）や Protobuf 強制（ADR-TIER1-002）の規約が「最初から守られている状態で生成される」ため、レビュー時の規約確認負荷が下がる

### ネガティブな帰結

- Paved Road 自身の品質維持が Platform/SRE の恒常業務として必要（`examples/` の CI、scaffold の golden snapshot test など）
- 「Paved Road の外」に進んだチームが孤立するリスクがあり、EM レイヤーでのフォローが必要
- Paved Road の更新時に既存サービスとの差分が発生し、マイグレーション戦略が必要（破壊的変更は ADR 化必須）
- 「自己責任」の線引きに不満を持つチームが発生した場合、組織内の政治的調整コストが EM / Product Council に発生する
- Scaffold CLI が一時的に壊れた場合、新規サービス起票が全社で止まるリスク（運用蓄積後に可用性 99% を目標とする）

### 移行・対応事項

- `examples/` 配下に tier2 ドメインサービス・tier3 Web 画面・SDK 追加言語・Protobuf 契約の 4 種類の Golden Path サンプルを配置（リリース時点）
- Scaffold CLI（`src/platform/scaffold/`）を Rust で実装し、Backstage Software Template の YAML 定義と共通のテンプレート参照を持たせる（リリース時点）
- Scaffold CLI の golden snapshot test を `tests/golden/` に配置し、テンプレート改変時の差分を PR で可視化する
- catalog-info.yaml のスキーマを定義し、Scaffold 生成テンプレートに組み込む
- time-to-first-commit 計測を Backstage Scorecards に実装（ADR-DX-001 と連動）
- Paved Road 逸脱時の exit plan ADR テンプレートを `docs/00_format/` に配置
- 採用側の運用蓄積後に Scaffold 利用率と time-to-first-commit を PMC でレビューし、目標未達の場合は Paved Road 側を再設計する
- Paved Road 利用者向けの問い合わせ窓口（Slack チャネル / Backstage 内ヘルプ）を Platform/SRE 側で整備し、SLA を明示する
- Paved Road 外運用を選択するチームに向けた「exit plan ADR テンプレート」のレビュープロセスをリリース時点で確立
- 新規参加者オンボーディング資料に「Paved Road とは何か・なぜ外れる自由があるか」の説明を必ず含め、思想を世代交代に耐える形で言語化する

## 参考資料

- [ADR-BS-001: 開発者ポータルに Backstage を採用](ADR-BS-001-backstage.md)
- [ADR-CICD-003: ポリシー適用に Kyverno を採用](ADR-CICD-003-kyverno.md)
- [ADR-TIER1-001: tier1 Go/Rust ハイブリッド](ADR-TIER1-001-go-rust-hybrid.md)
- [ADR-TIER1-003: tier1 内部言語の不可視化](ADR-TIER1-003-language-opacity.md)
- [CLAUDE.md](../../../CLAUDE.md)
- Netflix Tech Blog: "Full Cycle Developers at Netflix" (Paved Road 概念の提唱)
- Spotify Engineering: "Golden Path to Production" (Backstage Software Template の原型)
- Google SRE Book Ch. 32 "The Evolving SRE Engagement Model" (Paved Road とサポート境界)
- Thoughtworks Technology Radar: "Paved roads" (Adopted 2020)
- Martin Fowler: "PavedRoad" (martinfowler.com/bliki/PavedRoad.html)
