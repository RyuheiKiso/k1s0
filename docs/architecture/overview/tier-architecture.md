# Tier Architecture

k1s0 のリポジトリは **system → business → service** の3階層（Tier）で構成される。
上位層ほど汎用的・共通的であり、下位層は上位層に依存する。

> **用語の定義**: アーキテクチャ上の階層概念を「**Tier**」と呼ぶ。コードベースのディレクトリ名は歴史的経緯により `regions/` を使用しているが、同じ概念を指す。ドキュメント内では「Tier」で統一する。

## 階層の概要

<img src="diagrams/tier-overview.svg" width="1000" />

## 各階層の役割

### system（最上位）

全プロジェクトで横断的に利用する共通基盤。

| 種別     | 役割                                           |
| -------- | ---------------------------------------------- |
| server   | 共通のサーバー基盤（認証、API ゲートウェイ等） |
| library  | server・client 双方で使う共通ライブラリ        |
| database | 共通基盤のデータストア（ユーザー、認証等）     |

### business（中位）

部門名または業務領域（例：経理、FA）ごとにディレクトリを切り、その領域固有の共通基盤を置く。

| 種別     | 役割                                         |
| -------- | -------------------------------------------- |
| server   | 領域固有の共通サーバー基盤                   |
| client   | 領域固有の共通クライアント基盤               |
| library  | 領域内で server・client 双方が使う共通コード |
| database | 領域固有の共通データストア                   |

### service（最下位）

実際にデプロイ・稼働する個別サービス。

| 種別     | 役割                                        |
| -------- | ------------------------------------------- |
| server   | 業務サービスの実装（Rust）                  |
| client   | ユーザー向けクライアント（React / Flutter） |
| database | サービス固有のデータストア                  |

## 依存関係

### 依存方向のルール

依存は **下位 → 上位** の一方向のみ許可する。逆方向や同階層間の直接依存は原則禁止。

| 階層     | 依存先           |
| -------- | ---------------- |
| system   | なし（独立）     |
| business | system           |
| service  | system, business |

- 全 Tier から system Tier への依存は許可する（認証・config 取得等の共通基盤へのアクセスは全 Tier から必要）
- service Tier から system Tier への直接通信は、business Tier を経由せずに許可する

### 同階層間通信の例外規定

基本原則「同階層間の直接依存は禁止」を維持した上で、以下の例外を認める。

| Tier     | 例外規定                                                                                           |
| -------- | -------------------------------------------------------------------------------------------------- |
| system   | 共通基盤サービス間の通信は許可（例: 認証サービス → config サービス）                               |
| business | 同一ドメインコンテキスト内のサービス間通信は許可（異なるドメインコンテキスト間は非同期メッセージングを使用） |
| service  | BFF 間の通信は原則禁止                                                                             |

**補足:**
- system Tier 内の通信は、共通基盤としての一体的な動作を保証するために許可する
- business Tier 内では、同一ドメインコンテキスト（例: 同じ `accounting/` 配下）のサービス間に限り同期通信を許可する。異なるドメインコンテキスト間（例: `accounting/` と `fa/`）では Kafka 等の非同期メッセージングを使用する
- service Tier 内の BFF は、それぞれが独立したエンドポイントとして動作するため、BFF 間の直接通信は禁止する

### Server 間の依存イメージ

system server が提供する共通機能を、business server が利用する構成。

<img src="diagrams/server-dependency.svg" width="1000" />

**依存の具体例：**

| business server が system server から利用するもの | 方式                       |
| ------------------------------------------------- | -------------------------- |
| 認証トークンの検証                                | system library を import   |
| ユーザー情報の取得                                | system server へ gRPC 呼出 |
| ログ・トレースの送信                              | system library を import   |
| レート制限・ルーティング                          | API ゲートウェイを経由     |

### Client 間の依存イメージ

service client が business client の共通UIコンポーネントと system client SDK を利用する構成。
system/client は UI を持たない共通 SDK として、認証・API クライアント・共通 Widget を提供する。

<img src="diagrams/client-dependency.svg" width="1000" />

**依存の具体例：**

| service/business client が system client SDK から利用するもの | 方式                         |
| ------------------------------------------------------------ | ---------------------------- |
| 認証状態の管理・ログイン/ログアウト                          | system client SDK を import  |
| API リクエストの送信（Cookie / CSRF 対応済み）               | system client SDK を import  |
| ルーティングガード（未認証リダイレクト）                     | system client SDK を import  |
| 共通 Widget / Component                                      | system client SDK を import  |

| service client が business client から利用するもの | 方式                       |
| -------------------------------------------------- | -------------------------- |
| 領域共通のUIコンポーネント                         | business client を import  |
| 領域共通のレイアウト・テーマ                       | business client を import  |

## データベース

各階層は独立したデータベースを持つ。RDBMS は要件に応じて以下から選択する。

| RDBMS      | 主な用途                                         |
| ---------- | ------------------------------------------------ |
| PostgreSQL | 本番環境の標準選択肢。高機能・高信頼性           |
| MySQL      | 既存システムとの互換性が求められる場合            |
| SQLite     | ローカル開発・テスト・軽量な組込み用途            |

### 階層ごとのデータベース責務

| 階層     | データベースが管理する主なデータ                           |
| -------- | --------------------------------------------------------- |
| system   | ユーザー、認証・認可、監査ログなど横断的なデータ          |
| business | 領域固有のマスタデータ、領域内で共有するトランザクション  |
| service  | サービス固有の業務データ                                  |

各階層のデータベースは **その階層の server からのみアクセス** する。下位層が上位層のデータを必要とする場合は、上位層の server が提供する API（gRPC 等）を経由する。

## ディレクトリ構成

```
regions/
├── system/
│   ├── server/
│   │   └── rust/
│   ├── client/
│   │   ├── flutter/               # 共通 Flutter SDK（system_client パッケージ）
│   │   └── react/                 # 共通 React SDK（system-client パッケージ）
│   ├── library/
│   │   ├── go/
│   │   ├── rust/
│   │   ├── typescript/
│   │   └── dart/
│   └── database/
├── business/
│   └── {領域名}/          # 例: accounting, fa
│       ├── server/
│       │   └── rust/
│       ├── client/
│       │   ├── react/
│       │   └── flutter/
│       ├── library/
│       │   ├── go/
│       │   ├── rust/
│       │   ├── typescript/
│       │   └── dart/
│       └── database/
└── service/
    └── {サービス名}/          # 例: order, inventory
        ├── server/
        │   └── rust/
        ├── client/
        │   ├── react/
        │   └── flutter/
        └── database/
```

## 開発ツールの位置づけ

k1s0 CLI や Tauri GUI（[TauriGUI設計](../../cli/gui/TauriGUI設計.md)）などの開発ツールは、Tier アーキテクチャの外に位置する。これらは業務システムの構成要素ではなく、開発者のローカル環境で動作するツールである。Tier 内のアプリケーション（server / client / library / database）とは明確に区別する。

## 設計メモ

- **system/client は UI を持たない共通 SDK** — system/client はエンドユーザー向けの画面アプリではなく、business/service client が共通して使う認証・API クライアント・共通 Widget・ルーティングガードを提供する共有ライブラリパッケージである。直接デプロイする対象ではなく、下位層が依存パッケージとして import して使用する。
- **business 層に server/client を置く意義** — 同一業務領域内で複数サービスが共通のサーバー処理やクライアントコンポーネントを共有するケースに対応する。共通コードが library だけで足りる場合は library のみ配置すればよい。
- **各階層に database を置く理由** — データの所有権を階層ごとに明確にし、他階層のデータベースへの直接アクセスを禁止する。これにより階層間の結合度を低く保ち、スキーマ変更の影響範囲を限定できる。
- **RDBMS の選択** — プロジェクト全体で統一する必要はなく、階層・サービスの要件に応じて PostgreSQL / MySQL / SQLite から選択する。ただし同一階層内ではなるべく統一することを推奨する。

## 関連ドキュメント

- [ディレクトリ構成図](ディレクトリ構成図.md)
- [kubernetes設計](../../infrastructure/kubernetes/kubernetes設計.md)
- [メッセージング設計](../messaging/メッセージング設計.md)
- [サービスメッシュ設計](../../infrastructure/service-mesh/サービスメッシュ設計.md)
- [認証認可設計](../../auth/design/認証認可設計.md)
- [APIゲートウェイ設計](../../api/gateway/APIゲートウェイ設計.md)
- [API設計](../../api/gateway/API設計.md)
- [CLIフロー](../../cli/flow/CLIフロー.md)
- [TauriGUI設計](../../cli/gui/TauriGUI設計.md)
- [可観測性設計](../../observability/overview/可観測性設計.md)
- [CI-CD設計](../../infrastructure/cicd/CI-CD設計.md)
