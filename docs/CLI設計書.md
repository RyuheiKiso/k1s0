# CLI設計書

## 技術スタック

Rust 2024Edition

## コマンド一覧

引数無しを前提とする対話式CLI

## リポジトリ構成

```
k1s0/
├── CLI/
├── system-region/       ← システム共通領域
├── business-region/     ← 部門固有領域
├── service-region/      ← 業務固有領域
├── docs/
└── README.md
```

## Region間の依存関係

```
system-region           ← 依存なし（独立）
    ↑
business-region         ← system-region に依存
    ↑
service-region          ← system-region, business-region に依存
```

## Region別チェックアウト範囲

Git Sparse Checkout を利用し、選択したRegionに必要なディレクトリのみ取得する。
system-region 選択時はさらにプロジェクト種別（Library / Service）を選択する。

| Region選択      | プロジェクト種別 | チェックアウト対象                                        |
| --------------- | ---------------- | --------------------------------------------------------- |
| system-region   | Library          | `system-region/library/`                                  |
| system-region   | Service          | `system-region/service/`                                  |
| business-region | —                | `system-region/` + `business-region/`                     |
| service-region  | —                | `system-region/` + `business-region/` + `service-region/` |

## フロー図

```mermaid
flowchart TD
    A["k1s0.exe"] --> B["メインメニュー"]

    B --> C["プロジェクト作成"]
    B --> D["設定"]
    B --> E["終了"]

    C --> F{"ワークスペース設定あり?"}
    F -->|Yes| G["どの領域の開発を実施しますか？"]
    G --> G1["system-region : システム共通領域"]
    G --> G2["business-region : 部門固有領域"]
    G --> G3["service-region : 業務固有領域"]
    G1 --> PT["プロジェクト種別を選択"]
    PT --> PT1["Library : ライブラリ"]
    PT --> PT2["Service : サービス"]
    PT1 --> SC1a["sparse-checkout set system-region/library/"]
    PT2 --> SC1b["sparse-checkout set system-region/service/"]
    G2 --> BR{"部門固有領域の選択"}
    BR --> BR1["既存の部門固有領域"]
    BR --> BR2["新規追加"]
    BR1 --> BRL["既存の部門固有領域一覧から選択"]
    BRL --> SC2["sparse-checkout set system-region/ business-region/{選択した領域}/"]
    BR2 --> BRN["部門固有領域名を入力"]
    BRN --> SC2N["sparse-checkout set system-region/ business-region/{入力した領域}/"]
    G3 --> SC3["sparse-checkout set system-region/ business-region/ service-region/"]
    F -->|No| H["ワークスペースパス設定へ促す"]

    D --> I["ワークスペースパス確認"]
    D --> L["ワークスペースパス設定"]
    D --> J["終了"]

    I --> I1["現在のワークスペースパスを表示"]

    E --> K["プログラム終了"]
```