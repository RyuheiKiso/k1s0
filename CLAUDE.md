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

- 1 ファイルあたりの行数は 500 行以内とすること。いかなる例外も認めない。（ただし docs 内のファイルは例外的に許可する）
- 500 行を超える場合は、適切にファイルを分割すること。

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

k1s0 は一次ディレクトリ `src/` を持つモノレポで、tier1〜tier3 の実装と横断資産を集約する。
ADR-DIR-001/002/003 により以下のレイアウトが Phase 0 時点で確定されている。スパースチェックアウト（cone mode）での役割別運用が標準。

```
k1s0/
├── .github/                        # GitHub Actions / CODEOWNERS / PR テンプレート
├── .sparse-checkout/roles/         # 9 役別 cone 定義（tier1-rust-dev.txt 等）
├── .devcontainer/                  # 役割別 Dev Container プロファイル（軽量 docs-writer をルート既定）
├── .claude/                        # Claude Code ハーネス
├── CLAUDE.md                       # 本ファイル
├── docs/                           # ドキュメント（後述）
├── src/                            # ソースコード一次ディレクトリ
│   ├── contracts/                  # Protobuf 契約（ADR-DIR-001 でルート昇格）
│   │   ├── tier1/v1/               # tier1 公開 11 API
│   │   └── internal/v1/            # tier1 内部 gRPC
│   ├── tier1/                      # tier1 システム基盤層
│   │   ├── go/                     # Dapr ファサード層（DS-SW-COMP-124）
│   │   └── rust/                   # 自作 Rust 領域（DS-SW-COMP-129）
│   ├── sdk/                        # 多言語クライアント SDK（4 言語）
│   │   ├── dotnet/                 # C# NuGet（K1s0.Sdk）
│   │   ├── go/                     # Go module
│   │   ├── rust/                   # Rust crate（k1s0-sdk）
│   │   └── typescript/             # npm（@k1s0/sdk）
│   ├── tier2/                      # ドメイン共通業務ロジック
│   │   ├── dotnet/services/        # C# ドメインサービス
│   │   ├── go/services/            # Go ドメインサービス
│   │   └── templates/              # 雛形 CLI 参照テンプレ
│   ├── tier3/                      # エンドアプリ
│   │   ├── web/                    # React + TypeScript + pnpm workspace
│   │   ├── native/                 # .NET MAUI
│   │   ├── bff/                    # Backend For Frontend（Go）
│   │   └── legacy-wrap/            # .NET Framework ラップ（ADR-MIG-001）
│   └── platform/                   # 雛形 CLI / analyzer / Backstage プラグイン
├── infra/                          # クラスタ素構成（ADR-DIR-002 でルート昇格）
│   ├── k8s/                        # bootstrap / namespaces / networking / storage
│   ├── mesh/                       # istio-ambient / envoy-gateway
│   ├── dapr/                       # control-plane / components
│   ├── data/                       # cloudnativepg / kafka / valkey / minio
│   ├── security/                   # keycloak / openbao / spire / cert-manager / kyverno
│   ├── observability/              # LGTM + Pyroscope + OTel Collector
│   ├── feature-management/         # flagd
│   ├── scaling/                    # KEDA
│   └── environments/               # dev / staging / prod 差分
├── deploy/                         # GitOps 配信定義
│   ├── apps/                       # ArgoCD Application / ApplicationSet
│   ├── charts/                     # 共通 Helm charts
│   ├── kustomize/                  # base + overlays
│   ├── rollouts/                   # Argo Rollouts AnalysisTemplate
│   ├── opentofu/                   # ベアメタル / IaaS プロビジョン
│   └── image-updater/
├── ops/                            # 運用領域
│   ├── runbooks/                   # Runbook（incidents / daily / weekly / monthly）
│   ├── chaos/                      # LitmusChaos シナリオ
│   ├── dr/                         # DR 手順・スクリプト
│   ├── oncall/                     # オンコール運用
│   ├── load/                       # 負荷試験（k6）
│   └── scripts/                    # 運用スクリプト
├── tools/                          # 横断ツール
│   ├── devcontainer/profiles/      # 8 役別 Dev Container
│   ├── local-stack/                # kind / k3d ローカルクラスタ
│   ├── codegen/                    # buf / openapi / scaffold
│   ├── sparse/                     # スパースチェックアウト CLI
│   ├── ci/                         # CI helper
│   ├── git-hooks/
│   └── migration/
├── tests/                          # tier 横断テスト
│   ├── e2e/                        # Go で記述
│   ├── contract/                   # Pact / OpenAPI 契約
│   ├── integration/                # testcontainers
│   ├── fuzz/                       # cargo-fuzz / go-fuzz
│   ├── golden/                     # scaffold 出力 snapshot
│   └── fixtures/
├── examples/                       # Golden Path 実稼働例
└── third_party/                    # 社内 OSS フォーク vendoring（UPSTREAM.md / PATCHES.md 必須）
```

依存方向: `tier3 → tier2 → (sdk ← contracts) → tier1 → infra` の一方向。逆向きは禁止。

### docs 配下

```
docs/
├── 00_format/          # フォーマットテンプレート・規約
├── 01_企画/            # 稟議向け企画資料（薄い提案書）
├── 02_構想設計/        # 技術深掘り資料（ADR 索引含む）
│   └── adr/            # ADR 全体（ADR-DIR-001/002/003 含む）
├── 03_要件定義/        # 要件定義書（IPA 共通フレーム 2013 準拠）
│   ├── 00_要件定義方針/  # スコープ・前提制約・ステークホルダ・用語集・エラータクソノミ
│   ├── 10_業務要件/      # 業務背景・ビジネスプロセス・主要ユースケース
│   ├── 20_機能要件/      # tier1 公開 11 API / 外部連携 / 情報要件
│   ├── 30_非機能要件/    # A 可用性 / B 性能 / C 運用 / D 移行 / E セキュリティ / F エコロジー / G プライバシー / H 完整性
│   ├── 40_運用ライフサイクル/
│   ├── 50_開発者体験/
│   ├── 60_事業契約/
│   ├── 80_トレーサビリティ/
│   └── 90_付録/
├── 04_概要設計/        # 概要設計（DS-SW-COMP-*）
├── 05_実装/            # 実装フェーズ設計
│   └── 00_ディレクトリ設計/ # IMP-DIR-* 全 145 予約、57 件採番済
├── 90_knowledge/       # 技術学習用ドキュメント（/knowledge コマンドで生成）
└── 99_壁打ち/          # ブレスト・検討メモ
```

### スパースチェックアウト運用（ADR-DIR-003）

- 全開発者は役割を選び `./tools/sparse/checkout-role.sh <role>` で cone を適用する
- 標準 9 役: `tier1-rust-dev` / `tier1-go-dev` / `tier2-dev` / `tier3-web-dev` / `tier3-native-dev` / `platform-cli-dev` / `infra-ops` / `docs-writer` / `full`
- 初回 clone は `git clone --filter=blob:none --sparse` を推奨（partial clone）
- Git 2.40+ の sparse index を有効化（`core.sparseCheckoutCone=true`）

実装ディレクトリ設計の詳細は [docs/05_実装/00_ディレクトリ設計/](docs/05_実装/00_ディレクトリ設計/) を参照。

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

## 図解レイヤ記法規約

複数レイヤのコンポーネントが登場する drawio は [docs/00_format/drawio_layer_convention.md](docs/00_format/drawio_layer_convention.md) の記法規約に従うこと。
同じビジュアル (例: サイドカー) で責務レイヤの異なるコンポーネントを描くと、読み手が論理的差異を視覚的同一性に覆い隠されて誤解する。色・線種・配置の 3 軸で責務レイヤを常に明示的に分離すること。

- アプリ層: 暖色 (`#fff2cc` / `#d79b00`) — アプリから明示的に呼び出す抽象 (Dapr 等)
- ネットワーク層: 寒色 (`#dae8fc` / `#6c8ebf`) — アプリから透過的に効く機構 (Istio Ambient 等)
- インフラ層: 中性灰 (`#f5f5f5` / `#666666`) — Kubernetes / Node
- データ層: 薄紫 (`#e1d5e7` / `#9673a6`) — 状態保持コンポーネント (Postgres / Kafka 等)

線種: 実線太=明示呼び出し / 点線細=透過捕捉 / 破線=非同期。詳細と全レイヤ定義は規約ファイルを参照。

## ポリシー

目先の楽な提案や作業ではなく、プロフェッショナルとしての最適解を追求すること。
プロジェクトの成功にとって重要なことは何かを常に考え、短期的な妥協を避けること。
プロジェクトの品質と成功に対して妥協しないこと。必要な場合は、困難な決定を下すことを恐れないこと。