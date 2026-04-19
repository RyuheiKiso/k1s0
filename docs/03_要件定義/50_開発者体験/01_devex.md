# DEV-DEX: 開発者体験要件

本ファイルは、tier2/tier3 の業務アプリ開発者、および tier1 自身のランタイム開発者が、k1s0 上で日常的な開発ワークフローを高速・低摩擦で遂行できるための要件を定義する。雛形 CLI によるスケルトン生成、Tilt ベースのローカル開発環境、Backstage カタログ自動登録、ドキュメントサイト網羅性、デバッグ体験、SDK 多言語対応の 6 軸で要件を立てる。

開発者体験は定性的評価に流れやすいが、本ファイルでは**定量指標を要件の中核**に据える。具体的には「tier3 新規開発者オンボーディング〜初回デプロイ時間 30 分以内」「PR 開始〜マージ中央値リードタイム 1 営業日以内」「ローカル E2E テスト成功率 95% 以上」「雛形 CLI 生成直後の CI 通過確率 100%」を各要件の検証基準に紐付ける。これらの指標は `20_品質特性/02_observability.md` で定義される可観測性要件に連動し、Grafana ダッシュボードで運用時に追跡される。

達成できなかった場合の帰結は単なる不快感ではなく、JTC 顧客部門での**普及停滞**である。tier3 開発者が最初の 30 分で挫折するプラットフォームは、社内普及の口コミ速度が業務部門の抵抗速度を下回り、稟議通過後も使われないまま塩漬けになる。本要件群はその臨界点を 30 分に置き、オンボーディング導線の各ステップで摩擦を測る。

---

## 前提

- [`../../02_構想設計/04_CICDと配信/01_開発者ポータル_Backstage.md`](../../02_構想設計/04_CICDと配信/01_開発者ポータル_Backstage.md) — Backstage 位置付け
- [`../../02_構想設計/04_CICDと配信/04_ローカル開発環境.md`](../../02_構想設計/04_CICDと配信/04_ローカル開発環境.md) — Tilt 採用根拠とワークフロー
- [`../../02_構想設計/02_tier1設計/02_API契約/03_API設計原則.md`](../../02_構想設計/02_tier1設計/02_API契約/03_API設計原則.md) — 雛形 CLI の位置付け
- [`../20_品質特性/02_observability.md`](../20_品質特性/02_observability.md) — トレース可視化 / ログ検索基盤
- [`../10_アーキテクチャ/05_api_sdk_contract.md`](../10_アーキテクチャ/05_api_sdk_contract.md) — SDK 契約管理
- [`../00_共通/00_glossary.md`](../00_共通/00_glossary.md) — 雛形 CLI / 配信ポータル / Dapr ファサードの定義

---

## 要件本体

### DEV-DEX-001: 雛形 CLI による tier3 スケルトン生成

- 優先度: MUST（tier3 の一貫性と「雛形から外れた独自実装」の CI 検知の前提）
- Phase: Phase 1a
- 関連: `COM-GLO-003`, `COM-GLO-004`, `DEV-TST-009`

現状、tier3 の業務アプリは企画段階のため 1 件も存在しない。雛形 CLI を用意しないまま着手すると、各チームが独自のディレクトリ構成・CI 設定・Tiltfile で立ち上げ、後から横断ルールを被せるコストが倍化する。加えて `COM-GLO-003`（tier3 は雛形 CLI のスケルトンから派生）の定義が絵に描いた餅になる。

達成後は、`k1s0 scaffold <service-name> --lang go|ts` の 1 コマンドで、tier3 スケルトンが完全に立ち上がる。生成物には `.github/workflows/ci.yml` / `deploy/base/*.yaml` / `catalog-info.yaml` / `Tiltfile` / `testdata/` / `README.md` / SDK 初期化コードが含まれ、生成直後の PR が CI 全通過する（`DEV-TST-009` の要件と対応）。MVP-0 では Go / TypeScript の 2 種類に限定する。

崩れた場合、tier3 の構造が属人化し、運用横断の監査・SBOM 集約・Backstage カタログ同期が動かない。結果として**「tier3 を管理する仕組みがない」状態で複数テナント運用フェーズに突入**し、運用工数試算が根拠を失う。

**受け入れ基準**

- `k1s0 scaffold <service-name> --lang go` / `--lang ts` の 2 パターンで生成完了まで 60 秒以内
- 生成物に `.github/workflows/ci.yml` / `deploy/base/*.yaml` / `catalog-info.yaml` / `Tiltfile` / `testdata/` の 5 点が含まれる
- 生成直後の PR 作成で CI が 100% グリーン（`DEV-TST-009` と整合）
- 生成時に tier1 gRPC コンシューマとしての Pact 契約雛形も同時生成

**検証方法**

- MVP-0 完了時点で 2 種類の雛形を実際に生成し、初回 PR の CI 結果を artifact として保存
- 四半期ごとに雛形 CLI の回帰テスト（`DEV-TST-009`）結果をレビュー

---

### DEV-DEX-002: ローカル開発環境 30 分以内の立ち上がり

- 優先度: MUST（稟議通過後の社内普及速度を決める臨界点）
- Phase: Phase 1a
- 関連: `DEV-DEX-001`, `DEV-DEX-005`

構想設計 04 章（ローカル開発環境）は Tilt 採用を定めたが、新規開発者が**環境構築〜初回デプロイを完了するまでの総所要時間**が要件化されていない。JTC 開発者の PC スペック（メモリ 8 GB、CPU 4 コア想定）と、社内ネットワーク経由のイメージプル速度を踏まえて上限値を置く必要がある。

達成後は、`git clone` から `tilt up` で開発サービスが `localhost:8080` に応答するまでが **30 分以内**（median）に収まる。内訳は前提ツールインストール（Docker Desktop / kind / kubectl / Tilt）10 分、雛形 CLI 実行と依存 pull 15 分、`tilt up` による全 Pod Ready 5 分を上限とする。社内ミラーレジストリ（Harbor）経由でのイメージプルを前提にし、インターネット直取得には依存しない。

崩れた場合、新規メンバーのオンボーディングが複数日に及び、企画書で試算した **協力者バス係数 2 の達成**が半年遅延する。Phase 2 でパイロットテナントの数を増やす段階で人的ボトルネックが顕在化し、事業計画が下方修正される。

**受け入れ基準**

- `scripts/bootstrap.sh`（Mac/Linux）/ `scripts/bootstrap.ps1`（Windows）が前提ツールをワンショットでインストール
- `kind create cluster --config k1s0-dev.yaml && tilt up` の全工程が median 30 分以内（5 名の異なる PC で計測）
- 全イメージが社内 Harbor ミラーから pull 可能（インターネット直取得依存ゼロ）
- 手順書（Backstage TechDocs）が「トラブルシュートを 5 ステップ以内に収める」FAQ 付き

**検証方法**

- 四半期ごとに「新入社員 2 名が同じ手順で環境立ち上げ」実測演習を実施し、中央値を測定
- 計測結果は `DEV-DEX` ダッシュボード（Grafana）で常時公開

---

### DEV-DEX-003: Backstage カタログの自動登録

- 優先度: MUST（サービス発見性の欠如は Phase 2 以降の運用工数を指数関数的に悪化させる）
- Phase: Phase 1b
- 関連: `COM-GLO-007`, `DEV-DEX-001`

構想設計 01 章（Backstage）で Software Catalog 採用は決定済みだが、カタログへの**登録方法**が要件化されていない。手動登録だと登録漏れが発生し、Backstage の価値（全サービス一覧性）が破綻する。

達成後は、雛形 CLI が生成する `catalog-info.yaml` が main ブランチへのマージ時に自動で Backstage に取り込まれ、サービスオーナー / 依存関係 / API / Lifecycle が自動可視化される。加えて、既存サービスの `catalog-info.yaml` 変更も 5 分以内に Backstage へ反映される（Backstage の `EntityProvider` ポーリング周期）。

崩れた場合、Backstage に登録されないサービスが蛇口のように発生し、運用チームが「どこに何があるか」を把握する手段を失う。インシデント時の影響範囲特定が手動調査になり、MTTR（平均復旧時間）が数時間単位で悪化する。

**受け入れ基準**

- `catalog-info.yaml` が main マージから 5 分以内に Backstage に反映される
- 登録漏れサービス（`deploy/base/` に存在するが Backstage 未登録）を検知する dry-run スキャナが週次で実行
- サービスオーナーと lifecycle が必須項目として雛形 CLI に組み込まれ、空欄では生成を拒否
- Backstage カタログから tier1 公開 API / OpenAPI / gRPC proto へリンク可能

**検証方法**

- Phase 1b 完了時点で tier1 / tier2 リファレンス実装 / tier3 サンプル 3 件が Backstage に登録されていることを確認
- 登録漏れスキャナの実行ログを月次レビュー

---

### DEV-DEX-004: ドキュメントサイトの全公開 API 網羅

- 優先度: MUST（tier1 公開 API の発見性が tier2/3 開発者の生産性を決める）
- Phase: Phase 1b
- 関連: `ARC-T1-*`, `DEV-DEX-003`

tier1 は `k1s0.Log` / `k1s0.State` / `k1s0.PubSub` / `k1s0.Workflow` 等の複数 API を公開する。ドキュメントサイトが不完全だと、tier2/3 開発者は tier1 のコードを直接読むか、口伝に頼ることになる。これは tier1 チームが tier2/3 の質問対応に追われて本務が進まない悪循環を生む。

達成後は、Backstage TechDocs に tier1 公開 API の全エンドポイント（gRPC メソッド / クライアントライブラリ関数）のリファレンス、使用例、エラーコード、レート制限が掲載され、**公開 API カバレッジ 100%** が CI で自動検証される（`.proto` ファイルから抽出したメソッドリストと TechDocs の見出しを突き合わせる）。

崩れた場合、tier2/3 開発者が tier1 の内部実装（Go / Rust）を読みにいき、tier1 の言語中立性という設計原則が実質的に崩壊する。tier1 チームへの問い合わせが月 100 件超となり、本務の進捗が 40% 遅延する。

**受け入れ基準**

- tier1 の全 `.proto` ファイルのメソッドが TechDocs に 1:1 対応して記載
- `proto-doc-coverage` スクリプトが CI で実行され、未ドキュメント API を検出時は PR ブロック
- エラーコード（`COM-ERR-*` 参照）と例外ハンドリング例がサンプルコードに含まれる
- ドキュメントの最終更新日が表示され、90 日以上更新されていないページは自動で警告ラベル付与

**検証方法**

- `proto-doc-coverage` レポートを GHA artifact に保存し、Phase 1b 完了時点で 100% を確認

---

### DEV-DEX-005: デバッグ体験（トレース可視化とログ検索）

- 優先度: MUST（Dapr 隠蔽により tier2/3 は daprd のログに直接アクセスできない）
- Phase: Phase 1b
- 関連: `COM-GLO-005`, `QUA-OBS-*`

Dapr ファサードが tier2/3 から daprd を隠蔽するため、tier2/3 開発者がエラーを追いかける際、そのままでは「tier1 のどこで何が起きたか」が見えない。可観測性要件（`QUA-OBS-*`）で分散トレース / ログ集約は整備されるが、**開発者がそれを検索・参照する UI**の要件を別建てで定義する。

達成後は、Backstage のサービスページから Grafana Tempo（分散トレース）と Loki（ログ）へのリンクが埋め込まれ、リクエスト ID 1 つで tier3 → tier2 → tier1 Go ファサード → daprd → バックエンド の全ホップが可視化される。ローカル開発時も Tilt Web UI から同等のリンクが辿れる。

崩れた場合、tier3 開発者がバグ調査のたびに tier1 チームへエスカレーションすることになり、tier1 チームの質問対応工数が爆発する。開発者体験の悪化は口コミで広がり、JTC 社内の他部門が k1s0 採用を見送る要因になる。

**受け入れ基準**

- 分散トレースに全ホップ（tier3 / tier2 / tier1 / daprd / バックエンド）が含まれる
- Backstage サービスページから 1 クリックで Tempo / Loki に遷移可能
- Tilt Web UI からも同等リンクが提供される
- リクエスト ID によるログ検索がローカル・ステージング・本番で統一インターフェース

**検証方法**

- tier3 サンプルアプリで意図的にエラーを発生させ、開発者が 5 分以内に根本原因を特定できるか実測

---

### DEV-DEX-006: SDK 多言語対応（初期 C# / Go / TypeScript）

- 優先度: MUST（JTC 顧客の既存言語資産に合わせる方針の直接的根拠）
- Phase: Phase 1a
- 関連: `COM-GLO-002`, `ARC-T1-*`, `DEV-TST-003`

tier2 の言語選択自由は企画書の主要勝ち筋であり、これを裏付けるには最低 3 言語の SDK が必要である。言語を増やしすぎると保守コストが線形に膨らむため、初期スコープを厳密に定める必要がある。

達成後は、tier1 公開 SDK が **C# / Go / TypeScript の 3 言語**で提供され、いずれも Protobuf から自動生成されたコード + 薄いラッパーで構成される。各 SDK は Pact 契約テスト（`DEV-TST-003`）でプロバイダ（tier1 Go ファサード）との整合が検証される。Java / Python は Phase 2 以降の拡張とし、MVP-0 では対象外と明記する。

崩れた場合、tier2 開発者が「自分の使いたい言語がないから tier1 を直接叩く」ルートに流れ、Dapr 隠蔽の原則（`COM-GLO-005`）が破壊される。企画の差別化根拠が実装レベルで崩壊する。

**受け入れ基準**

- C# / Go / TypeScript の 3 言語 SDK が Phase 1a 完了時に公開
- 各 SDK のバージョニングが Semantic Versioning に従い、リリース履歴が GitHub Releases に記録
- 各 SDK のサンプルコードが Backstage TechDocs に掲載
- Phase 2 以降の言語拡張ロードマップが文書化（Java / Python を候補として明記）

**検証方法**

- 3 言語 SDK それぞれで「初期化 → tier1 API 呼び出し → エラーハンドリング」のサンプルが動作することを Phase 1a 完了時に確認

---

### DEV-DEX-007: 雛形 CLI のアップデート通知

- 優先度: SHOULD（Phase 1b で必達。MVP-0 では手動通知で代替）
- Phase: Phase 1b
- 関連: `DEV-DEX-001`, `OPS-DEP-*`

雛形 CLI が進化する過程で、既に生成済みの tier3 プロジェクトは古い雛形のまま取り残される。放置するとセキュリティパッチや CI 改善が tier3 に波及せず、全体のセキュリティ姿勢が劣化する。

達成後は、雛形 CLI が新バージョンを公開した際、既存 tier3 リポジトリに対して `k1s0 scaffold doctor` コマンドで差分を検知し、GitHub Issue または Backstage 通知で対応を促す。自動 PR 作成（`renovate` 相当）までは Phase 2 以降の拡張とする。

崩れた場合、tier3 リポジトリが雛形の古いバージョンに固定化し、Phase 2 以降で雛形が進化してもセキュリティパッチが行き届かない。結果として「雛形を育てれば全 tier3 がついてくる」という運用設計の前提が崩壊する。

**受け入れ基準**

- `k1s0 scaffold doctor` コマンドが生成済みリポジトリとの差分を出力
- 差分検出時に GitHub Issue が自動起票（テンプレート `scaffold-upgrade`）
- 雛形バージョンが `catalog-info.yaml` に記録され、Backstage から古いバージョンを一覧化可能
- 月次レビューで古い雛形の数をダッシュボード化

**検証方法**

- Phase 1b 完了時に 3 件以上の tier3 リポジトリで `doctor` コマンドを実行し、検知漏れがないことを確認

---

## 章末サマリ

### ID 一覧

| ID | タイトル | 優先度 | Phase |
|---|---|---|---|
| DEV-DEX-001 | 雛形 CLI による tier3 スケルトン生成 | MUST | 1a |
| DEV-DEX-002 | ローカル開発環境 30 分以内の立ち上がり | MUST | 1a |
| DEV-DEX-003 | Backstage カタログの自動登録 | MUST | 1b |
| DEV-DEX-004 | ドキュメントサイトの全公開 API 網羅 | MUST | 1b |
| DEV-DEX-005 | デバッグ体験（トレース可視化とログ検索） | MUST | 1b |
| DEV-DEX-006 | SDK 多言語対応（初期 C# / Go / TypeScript） | MUST | 1a |
| DEV-DEX-007 | 雛形 CLI のアップデート通知 | SHOULD | 1b |

### 優先度分布

| 優先度 | 件数 | 代表 ID |
|---|---|---|
| MUST | 6 | DEV-DEX-001, DEV-DEX-002, DEV-DEX-003, DEV-DEX-004, DEV-DEX-005, DEV-DEX-006 |
| SHOULD | 1 | DEV-DEX-007 |
| MAY | 0 | — |

### Phase 達成度

| Phase | 必達件数 | 達成見込み | 未達影響 |
|---|---|---|---|
| 1a | 3 | 稟議通過の前提（雛形 / 環境 / SDK 3 言語） | オンボーディング崩壊 / 差別化根拠喪失 |
| 1b | 4 | パイロット展開の必要条件（Backstage / TechDocs / デバッグ / Upgrade） | 運用工数爆発 / 雛形の陳腐化 |
