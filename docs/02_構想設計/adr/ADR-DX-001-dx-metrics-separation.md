# ADR-DX-001: DX メトリクスを稼働系 SLI と分離して管理

- ステータス: Accepted
- 起票日: 2026-04-24
- 決定日: 2026-04-24
- 起票者: kiso ryuhei
- 関係者: Platform/SRE / セキュリティチーム / DevEx チーム / EM / Product Council

## コンテキスト

k1s0 は NFR-A-AVL（可用性）・NFR-B-PERF（性能）・NFR-C-NOP（運用）の各 SLO を `docs/03_要件定義/30_非機能要件/` で宣言しており、本番稼働後は Grafana LGTM（ADR-OBS-001）および OTel Collector（ADR-OBS-002）の上で SLO / SLI として観測される。一方、10 年保守を前提とする組織運営では「稼働の健全性」とは別次元に「開発者体験の健全性」が存在する。DORA 4 keys（Lead Time for Changes / Deploy Frequency / MTTR / Change Failure Rate）、SPACE フレームワーク、および本プロジェクト固有の time-to-first-commit（ADR-DEV-001 で DX の一次 SLI として定義）・Scaffold CLI 利用率・Paved Road 適用率は、後者に属する。

ここで問題となるのは、稼働系 SLI と DX メトリクスを同一ダッシュボードに並置した場合の「優先順位判断の混濁」である。具体的には以下の病理が発生する。

- **優先順位の取り違え**: SRE が可用性 SLO と Lead Time を同じ画面で見ると、エラーバジェット消費中に「Lead Time が悪化しているからリリース頻度を上げよう」という判断が混入する。エラーバジェット管理は「消費中はリリース抑制」が原則であり、Lead Time 改善と真っ向から衝突する。同じダッシュボードに並ぶと「両方大事」という意思決定コストのかかるトレードオフが日常化する。
- **読者層の不一致**: 稼働系 SLI は SRE / オンコール / Security が見る。DX メトリクスは EM / Product Council / 開発チーム本人が見る。読者層が違うメトリクスを同じ画面に置くと、どちらの読者にも「関係ないメトリクスのノイズ」が発生する。特に EM はチームの健全性を見たいのに可用性グラフでスクロールが埋まる状況は、DX 改善の意思決定速度を直接低下させる。
- **アラート閾値の設計ミス**: 稼働系 SLI は即時対応型（エラーバジェット消費速度、P99 レイテンシ）で分単位のアラートを出す。DX メトリクスは傾向型（Lead Time の四半期推移、Scaffold 利用率の月次推移）で、分単位アラートは意味をなさない。両者を同じダッシュボードに置くと、アラート設計思想の衝突が必ず起きる。
- **SLO 判断ラインの汚染**: Security / SRE は「SLO 違反時に何をするか」の意思決定を行う。ここに DX メトリクス（開発者本人の活動記録）が混入すると、個人の活動を SLO 監視と同じ経路で扱うことになり、評価制度との距離感がバグる。SPACE フレームワークの原著（ACM Queue 2021）も「個人評価に直結させる使い方を禁止」と明記している。

Google SRE Book・Atlassian DevOps 成熟度モデル・DORA State of DevOps Report はいずれも DX メトリクスと稼働 SLO を別管理する運用を前提としている。k1s0 リリース時点で観測基盤の設計を確定する以上、メトリクスの所有境界を明文化しなければ、Grafana 上の「とりあえず全部置くダッシュボード」が 10 年分の技術負債として残る。

採用側の小規模運用段階では Platform/SRE と EM が兼任に近い状態で回るため、同一ダッシュボードでも運用上は回る錯覚が生じる。しかしロール分離が進んだ瞬間に、ダッシュボードの読者層不一致が顕在化する。10 年保守では分離が不可避であり、リリース時点でメトリクスの所有境界を決めることは、将来のダッシュボード再設計コスト（大規模な再構築規模）を事前に排除する意味を持つ。

## 決定

**DORA 4 keys、SPACE、time-to-first-commit、Scaffold 利用率、Paved Road 適用率などの DX メトリクスは、稼働系 SLI のダッシュボードとは別経路で管理する。**

- **ダッシュボード分離**: 稼働系 SLI は `docs/05_実装/60_観測性設計/` 配下の仕様に従い Grafana LGTM スタック上で管理する。DX メトリクスは Backstage Scorecards（ADR-BS-001）を第一手段とし、Backstage 内に独立ダッシュボードを持つ。Grafana の一般ユーザ向けホーム画面には DX メトリクスを表示しない。
- **メトリクス範囲**: DX メトリクス側に含めるのは以下。
  - DORA 4 keys: Lead Time for Changes / Deploy Frequency / MTTR / Change Failure Rate
  - SPACE の一部: Satisfaction（定期アンケート）/ Performance（PR merge 率）/ Activity（Scaffold 利用率）/ Collaboration（レビュー到達時間）/ Efficiency（time-to-first-commit）
  - プロジェクト固有: time-to-first-commit（ADR-DEV-001）/ Scaffold CLI 利用率 / Paved Road 適用率 / catalog-info.yaml 網羅率
- **混在禁止の理由明文化**: 「可用性 SLO と Lead Time を同じダッシュボードに置くと、エラーバジェット管理と Lead Time 改善が正面衝突する」ことを設計原則として Runbook に明記する。稼働ダッシュボードに DX メトリクスを追加する PR は原則却下する。
- **計測基盤の第一手段**: Backstage Scorecards を第一手段とする。Backstage が直接収集できないメトリクス（catalog-info.yaml 網羅率のような横断統計）は Backstage plugin として実装し、Backstage 内で完結させる。Grafana を使う場合も「DX 専用インスタンス」あるいは「明示的に分離された folder」に置き、SRE オンコールの閲覧動線と交差させない。
- **所有境界の明文化**: DX メトリクスは EM レイヤーが読み、改善意思決定を下すための情報として位置付ける。Security / SRE は DX メトリクスを SLO 判断ラインに使わない。逆に EM は稼働 SLI を本番判断に使わない。両者が交差するのは「DX 悪化が稼働悪化の先行指標になっているか」を四半期レビューで横断分析するときのみ。
- **個人評価との分離**: SPACE 原著に従い、DX メトリクスを個人評価に直結させることを禁止する。チーム単位の傾向として扱い、個人ランキング化は行わない。これは HR 領域との境界を引く意味でも重要。
- **DX メトリクスのレビューサイクル**: 四半期ごとに EM / DevEx / Product Council で DX メトリクス全体をレビューする。悪化傾向があれば Paved Road（ADR-DEV-001）や Scaffold CLI の再設計に反映する。稼働 SLI のレビューサイクル（週次 / 月次）とは独立。
- **相関分析の扱い**: DX 悪化が稼働悪化の先行指標として機能するケース（例: Change Failure Rate 上昇が MTTR 悪化を先取り）は確実に存在する。この相関分析は四半期レビューで横断的に行い、SRE オンコール判断には持ち込まない。相関分析の結果「稼働リスクが具体化した」と判断された場合は、別途 SRE 側の SLO レビューに情報を引き渡す。

### DX メトリクスをチーム単位で扱う理由

SPACE フレームワーク原著（Forsgren et al., ACM Queue 2021）は、開発生産性メトリクスを「個人評価に直結させる使い方を明確に禁止」と明言している。個人ランキング化すると、PR 数や merge 速度など観測可能な量を最大化する行動が誘発され、本質的な品質・協調が損なわれるためである。DORA 4 keys も組織単位の成熟度指標として設計されており、個人に適用することは想定されていない。本 ADR ではこの原則を組織運営レベルで採用し、DX メトリクスはチーム単位の傾向として扱う。個人の活動記録として利用することは禁止し、HR の評価制度とは API レベルで分離する。

### 稼働ダッシュボードとの接続点

完全分離といっても、DX と稼働の接続点はゼロにはできない。本 ADR では以下の 2 点のみを接続点として定義する。

- **障害時タイムライン**: 本番インシデント発生時、Change Failure Rate の集計対象として DX 側に反映する。ただし反映は事後バッチ処理であり、リアルタイム連携は行わない。
- **四半期横断分析**: DX メトリクスの四半期レビュー時に、同じ四半期の稼働 SLI のサマリを参考情報として添付する。添付は静的レポートであり、ダッシュボード上のリアルタイム連動ではない。

この 2 点以外の接続点は、意思決定の混濁を誘発するため禁止する。

### スコープ

本 ADR は「メトリクスの所有境界と表示経路」の意思決定である。個別メトリクスの算出ロジック・収集頻度・ダッシュボードレイアウトは実装段階（`docs/05_実装/50_開発者体験設計/` 等）で扱う。DORA 4 keys の具体的な計測方法は別 ADR で扱う可能性がある。

## 検討した選択肢

### 選択肢 A: DX と稼働 SLI の分離（採用）

- 概要: DX メトリクスは Backstage Scorecards、稼働 SLI は Grafana LGTM。ダッシュボード・所有境界・レビューサイクルを分離
- メリット:
  - エラーバジェット管理と Lead Time 改善の正面衝突を構造的に回避
  - SRE オンコール画面のノイズを削減、一次対応速度を維持
  - EM が DX 健全性を独立して判断でき、改善サイクルが高速化
  - SPACE 原著の「個人評価に直結させない」原則を組織構造に埋め込める
  - 10 年保守で DX / SRE チームが完全分離しても所有境界が曖昧化しない
- デメリット:
  - 両ダッシュボードの相関分析が必要なケース（障害と Lead Time の関係など）で横断作業が発生
  - ツール二系統（Backstage Scorecards と Grafana）の運用保守コスト

### 選択肢 B: 統合ダッシュボード（稼働と DX を同居）

- 概要: Grafana 上に稼働 SLI と DX メトリクスを一緒に並べる
- メリット: 表面的には「全部見える」ので初期満足度が高い
- デメリット:
  - エラーバジェット管理と Lead Time 改善の意思決定が画面上で衝突
  - SRE オンコール時に DX メトリクスがノイズとなり、一次対応が遅延
  - EM が関心のない可用性グラフをスクロールで飛ばす必要があり、DX 改善の意思決定が遅延
  - DORA / SPACE 原著が禁じている「個人評価と SLO の接続」を組織的に誘発するリスク

### 選択肢 C: DX 計測なし

- 概要: 稼働 SLI のみを計測し、DX は定性評価で済ませる
- メリット: 観測コストがゼロ
- デメリット:
  - Paved Road（ADR-DEV-001）の time-to-first-commit が計測不能となり、DX 改善が仮説検証型で進められない
  - 10 年保守で開発者入れ替わりが発生した際、健全性の劣化が可視化されず手遅れになる
  - DORA / SPACE ベンチマークとの比較が不可能、業界水準との乖離が判明しない

### 選択肢 D: DX のみ計測（稼働 SLI を外部委託）

- 概要: DX メトリクスのみ自前で計測し、稼働 SLI はクラウド提供元に依存
- メリット: 自前運用の観測基盤が軽量化
- デメリット:
  - オンプレ・ベアメタル前提（ADR-STOR-001 / ADR-STOR-002）の k1s0 では稼働 SLI の外部委託が技術的に成立しない
  - 稼働と DX の相関分析ができず、DX 悪化の原因が「稼働負荷」なのか「組織要因」なのか切り分けられない
  - SLO 判断ラインの主権を失い、SRE の本来機能が成立しない

### 選択肢 E: DX メトリクスを個人評価に直結

- 概要: DX メトリクスを個人の評価制度（人事考課）に組み込む
- メリット: メトリクス改善のインセンティブが強力に働く
- デメリット:
  - SPACE 原著が明確に禁止しているアンチパターン
  - PR 数・merge 速度の最大化が誘発され、本質的な品質・協調が損なわれる
  - HR 領域と SLO 観測が混在し、職務分掌・個人情報保護が破綻
  - メトリクスが「評価から逃げるための最適化対象」となり、DX 改善の意思決定データとして機能不全になる

## 帰結

### ポジティブな帰結

- エラーバジェット管理と Lead Time 改善の衝突が構造的に回避され、SRE の意思決定が単純化
- EM レイヤーが DX 健全性を独立して判断でき、Paved Road 再設計や Scaffold 改善の意思決定が四半期サイクルで回る
- SPACE 原著の「個人評価に直結させない」原則を組織構造に埋め込め、HR 領域との境界が明確化
- Backstage Scorecards が DX の一次情報源となり、ADR-BS-001 の投資が実質的に活かされる
- 10 年保守で DX チームと SRE チームが分離しても、ダッシュボード所有境界が曖昧化しない
- 稼働 SLI ダッシュボードのノイズが最小化され、オンコール一次対応の認知負荷が下がる
- 四半期横断分析で DX 悪化が稼働悪化の先行指標として機能するケースを検出でき、予兆管理が成立する

### ネガティブな帰結

- 稼働 SLI と DX メトリクスの相関を見たい四半期レビュー時に、横断分析の手作業が発生する
- Backstage Scorecards と Grafana の二系統運用で、監視基盤の保守コストが分散する
- 「全部同じダッシュボードに置いてほしい」という現場要望への説明コストが恒常的に発生する
- DX メトリクス計測のための Backstage plugin 開発が運用蓄積後の改善項目として残る
- DX と稼働の所有境界が明確になる反面、境界をまたぐ改善提案（例: Change Failure Rate 改善のための本番観測強化）が「どちらの担当か」で止まるリスクが発生
- Backstage Scorecards plugin のエコシステム成熟度が Grafana に比べて低く、可視化表現力の制約を受けるケースが発生しうる

### 移行・対応事項

- DX メトリクスの計測基盤を Backstage Scorecards で実装（リリース後に整備、ADR-BS-001 連動）
- DORA 4 keys 計測 plugin を Backstage に導入（OSS plugin の採用検討 + 必要に応じて内製）
- time-to-first-commit の計測ロジックを [../../../docs/05_実装/50_開発者体験設計/](../../../docs/05_実装/50_開発者体験設計/) に定義
- 「稼働 SLI ダッシュボードに DX メトリクスを追加しない」原則を Runbook に明記し、PR テンプレートでも注意喚起
- Backstage 経由の DX ダッシュボードと Grafana 稼働ダッシュボードの IdP 権限を分離（SRE は両方、EM は DX のみ、開発者は自チーム DX のみ）
- 四半期 DX レビューの議事フォーマットを `docs/00_format/` に配置
- 個人評価との接続を禁止する旨を就業規則 / 評価制度ガイドに反映（HR 連携タスク）
- 採用側の運用蓄積後に DX メトリクスと稼働 SLI の相関（DX 悪化が稼働悪化の先行指標か）を分析し、レビューサイクルの適正性を検証
- Paved Road（ADR-DEV-001）の time-to-first-commit / Scaffold 利用率と本 ADR の DX メトリクスの整合性を四半期レビュー時に確認し、定義の二重化・不整合を排除
- Backstage Scorecards plugin のメジャーバージョン更新時は DX メトリクス計算ロジックの後方互換を確認する Runbook を整備

## 参考資料

- [ADR-DEV-001: 開発者体験の根幹思想として Paved Road を採用](ADR-DEV-001-paved-road.md)
- [ADR-BS-001: 開発者ポータルに Backstage を採用](ADR-BS-001-backstage.md)
- [ADR-OBS-001: 観測性基盤に Grafana LGTM を採用](ADR-OBS-001-grafana-lgtm.md)
- [ADR-OBS-002: OTel Collector による計装統一](ADR-OBS-002-otel-collector.md)
- [CLAUDE.md](../../../CLAUDE.md)
- DORA "State of DevOps Report" 2023 / 2024
- Forsgren, Storey, Maddila et al. "The SPACE of Developer Productivity", ACM Queue 2021
- Google SRE Book Ch. 4 "Service Level Objectives"
- Google SRE Workbook Ch. 5 "Alerting on SLOs"
- Atlassian DevOps Maturity Model
- Backstage Scorecards plugin: [backstage.io/docs/features/scorecards](https://backstage.io)
