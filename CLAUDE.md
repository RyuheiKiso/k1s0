# プロジェクト規約

本ファイルは k1s0 プロジェクトにおける開発・ドキュメント作成の共通規約を定める。

## 技術スタック

- Rust Edition 2024（自作領域: ZEN Engine 統合 / crypto / 雛形 CLI / JTC 固有機能）
- Go（Dapr ファサード層: stable Dapr Go SDK を使用）
- tier1 内部のサービス間通信は Protobuf gRPC 必須
- tier2/tier3 から内部言語は不可視（クライアントライブラリと gRPC エンドポイントのみ公開）

## コーディング規約

### コメント

- コードの各行の 1 行上に、必ず日本語でコメントを記載すること。例外はない。
- コードファイルの先頭には、必ず日本語でファイルの説明コメントを記載すること。例外はない。

### ファイル構成（ソースコード）

- 1 ファイルあたりの行数は 300 行以内とすること。いかなる例外も認めない。（ただし docs 内のファイルは例外的に許可する）
- 300 行を超える場合は、適切にファイルを分割すること。

## ドキュメント規約

### 基本方針

- アスキーによる図や表の表現は禁止。
- 図表を掲載する場合は、md と同じ階層に `img` フォルダを作成し、drawio を作成してから svg を出力して md 内に埋め込むこと。
- 端末には drawio がインストールされているため、これを利用すること。
- 資料は可能な限り細分化すること。

### 散文と図の必須化

- 表を並べるだけの構成は不可。部外者が読んで文脈を理解できる品質が必須。
- 各章・各節の冒頭に、何を解決するかの導入段落を必ず置くこと。
- 表の前後に「なぜこの分類なのか」「どう読むか」「重要な関係性」の散文を添えること。
- 重要な概念・関係性・フロー・構造は drawio 図を作成し SVG で埋め込むこと。

### 要件ドキュメントの記述

- 各要件の本体を表セルに流し込むことは禁止。散文で「現状 → 要件達成後の世界 → 崩れた時の影響」を段落で展開すること。
- 表は章末のサマリ（ID 一覧 / Phase 達成度 / 優先度分布）にのみ使うこと。
- 受け入れ基準や手順など「条件の束」は散文の後の箇条書きで列挙すること。

### テーブルの品質基準

- ラベル羅列のテーブルは不可。各セルに根拠・数値・出典を入れ、1 行読んで納得できる密度にすること。
- 優先度セルは判定理由を同居させること（例: 「MUST（稟議通過の前提。未達で Phase 0 承認不可）」）。
- 数値セルは根拠を併記すること（例: 「48 時間以内（業界平均 72 時間を参考に短縮目標）」）。
- 期待効果セルは前後比較を書くこと（例: 「年次ライセンス費 2,000 万円削減、稟議期間 3 か月 → 0」）。
- 事例参照が可能な場合は事例そのものをセルに入れること。

## プロジェクト構造

```
docs/
├── 00_format/          # フォーマットテンプレート・規約
├── 01_企画/            # 企画書・設計ドキュメント
│   ├── 01_背景と目的/
│   ├── 02_アーキテクチャ/
│   ├── 03_tier1設計/
│   ├── 04_技術選定/
│   ├── 05_CICDと配信/
│   ├── 06_競合と差別化/
│   └── 07_ロードマップと体制/
├── 02_要件定義/        # 要件定義書（7 カテゴリ構造）
│   ├── 00_共通/              # 横断要件・assumption・constraint・risk・stakeholder・error_taxonomy・glossary
│   ├── 10_アーキテクチャ/    # infra・tier1-3・integration・api/sdk contract・rule_engine・eventing
│   ├── 20_品質特性/          # performance・availability・observability・sla_slo・tenant_isolation・DR_BCP・accessibility
│   ├── 30_セキュリティ_データ/ # security・IAM・kms_crypto・audit・data・privacy・data_residency・backup_restore・artifact_integrity・compliance
│   ├── 40_運用ライフサイクル/ # operation・CICD・release・environment_config・migration・incident_response・support・deprecation_EOL・exit_strategy
│   ├── 50_開発者体験/        # test・devex・feature_management
│   └── 60_事業_契約/         # UX・i18n_l10n・tenant_onboarding・training・cost・licensing・billing_metering・legal_contract・vendor_supplychain・governance・ai_governance・analytics・sustainability
├── 90_knowledge/       # 技術学習用ドキュメント（/knowledge コマンドで生成）
└── 99_壁打ち/          # ブレスト・検討メモ
```

## コンテキスト管理

- コンテキストウィンドウの限界に近づいた場合、Claude Code ハーネスが自動的にコンパクト（古いツール出力の削除 → 会話の要約）を実行する。モデル側からの制御はできない。
- 作業の区切りなどで明示的にコンパクトしたい場合は、`/compact` スラッシュコマンドを利用する。焦点を指定したい場合は `/compact <focus>` の形式で実行する。
- 会話履歴を完全に破棄して作業を再開したい場合は、`/clear` を利用する。

## drawio

白の矢印は視認性を損なうため禁止する。
矢印がボックスやテキストの上に重なることを禁止する。
矢印は常にボックスやテキストに重ならないように折り曲げたりして配置すること。
`orthogonalEdgeStyle` の自動ルーティングは他の要素を迂回しないため、依存してはならない。
XML 生成後、SVG エクスポート前に以下の交差判定を必ず実施すること:

1. 全ボックス・テキストの占有領域（x, y, width, height）を列挙する
2. 全矢印の経路（source 出口 → 中間点 → target 入口）が通過する座標範囲を算出する
3. 矢印の経路が他のボックス・テキストの領域と交差しないことを確認する
4. 交差が見つかった場合は矢印の迂回ではなく要素の配置自体を見直すこと

矢印のラベルが矢印自体を覆い隠すことを禁止する。
ラベル付き矢印の接続元と接続先の間隔は、ラベルの文字列幅より十分に広くとること（目安: ラベル幅の 1.5 倍以上）。
間隔が不足する場合は要素の配置を見直して間隔を確保すること。

GitHub のダークテーマに対応するために全体の背景は白で統一すること。
背景色を透明にすることは禁止する。
SVG エクスポート時、mxGraphModel の `background` 属性は無視される。`<root>` 直下の最初の要素としてページ全体を覆う白矩形を配置すること。

```xml
<mxCell id="bg" value="" style="rounded=0;whiteSpace=wrap;html=1;fillColor=#FFFFFF;strokeColor=none;" vertex="1" parent="1">
  <mxGeometry x="0" y="0" width="{pageWidth}" height="{pageHeight}" as="geometry" />
</mxCell>
```