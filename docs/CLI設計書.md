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
| system-region   | Library → Rust   | `system-region/library/rust/`                             |
| system-region   | Library → Go     | `system-region/library/go/`                               |
| system-region   | Service → Rust   | `system-region/service/rust/`                             |
| system-region   | Service → Go     | `system-region/service/go/`                               |
| business-region | 既存 → Rust      | `system-region/` + `business-region/{選択した領域}/rust/`  |
| business-region | 既存 → Go        | `system-region/` + `business-region/{選択した領域}/go/`    |
| business-region | 新規追加 → Rust  | `system-region/` + `business-region/{入力した領域}/rust/`  |
| business-region | 新規追加 → Go    | `system-region/` + `business-region/{入力した領域}/go/`    |
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
    PT1 --> LLang["言語を選択"]
    LLang --> LLang1["Rust"]
    LLang --> LLang2["Go"]
    LLang1 --> SC1aR["sparse-checkout set system-region/library/rust/"]
    LLang2 --> SC1aG["sparse-checkout set system-region/library/go/"]
    PT2 --> SLang["言語を選択"]
    SLang --> SLang1["Rust"]
    SLang --> SLang2["Go"]
    SLang1 --> SC1bR["sparse-checkout set system-region/service/rust/"]
    SLang2 --> SC1bG["sparse-checkout set system-region/service/go/"]
    G2 --> BR{"部門固有領域の選択"}
    BR --> BR1["既存の部門固有領域"]
    BR --> BR2["新規追加"]
    BR1 --> BRL["既存の部門固有領域一覧から選択"]
    BRL --> BRLang["言語を選択"]
    BRLang --> BRLang1["Rust"]
    BRLang --> BRLang2["Go"]
    BRLang1 --> SC2R["sparse-checkout set system-region/ business-region/{選択した領域}/rust/"]
    BRLang2 --> SC2G["sparse-checkout set system-region/ business-region/{選択した領域}/go/"]
    BR2 --> BRN["部門固有領域名を入力"]
    BRN --> BRNLang["言語を選択"]
    BRNLang --> BRNLang1["Rust"]
    BRNLang --> BRNLang2["Go"]
    BRNLang1 --> SC2NR["sparse-checkout set system-region/ business-region/{入力した領域}/rust/"]
    BRNLang2 --> SC2NG["sparse-checkout set system-region/ business-region/{入力した領域}/go/"]
    G3 --> SC3["sparse-checkout set system-region/ business-region/ service-region/"]
    F -->|No| H["ワークスペースパス設定へ促す"]

    D --> I["ワークスペースパス確認"]
    D --> L["ワークスペースパス設定"]
    D --> J["終了"]

    I --> I1["現在のワークスペースパスを表示"]

    E --> K["プログラム終了"]
```