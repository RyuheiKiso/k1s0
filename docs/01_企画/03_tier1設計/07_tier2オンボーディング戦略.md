# tier2 オンボーディング戦略

## 目的

tier2 開発者が tier1 公開 API とマイクロサービスアーキテクチャを段階的に習得できるようにする。「11 種の API を一度に覚えさせる」のではなく、**習熟度レベルに応じて解放される学習パス**と**既知概念からの橋渡しドキュメント**で認知負荷を制御する。

---

## 1. 背景となる課題

tier2 開発者の想定バックグラウンドは C# + REST + SQL Server ([ペルソナ 5: 長谷川亮](../01_背景と目的/01_ペルソナ.md))。k1s0 への参画にあたり、以下の新概念を**同時に**習得する必要がある。

| 新概念 | 従来との差分 |
|---|---|
| 11 種の tier1 公開 API | 各 API の動作・エラーハンドリング方針が異なる |
| gRPC + Protobuf | REST + JSON からの移行 |
| イベント駆動 (PubSub) | 同期 API 呼び出しからの移行 |
| 結果整合性 + Saga | ACID トランザクションからの移行 |
| Kubernetes リソース管理 | IIS / オンプレミスからの移行 |
| 分散トレーシング | テキストログ検索からの移行 |

これらを一括で要求すると、リファレンス実装すら読めない状態が発生する。本戦略はこの課題に対処する。

---

## 2. API 習熟度レベル

### 2.1 レベル定義

tier1 公開 API を 5 段階に分類し、前のレベルを習得してから次に進む構成とする。

| レベル | 使用 API | 習得する概念 | 目安期間 |
|---|---|---|---|
| **L1: 基礎** | `Log`, `Telemetry`, `Auth` | 構造化ログ、認証トークン検証、基本的なリクエスト処理 | 1–2 日 |
| **L2: データ** | + `State`, `Secrets` | KV ストア操作、シークレット取得、キャッシュ戦略 | 2–3 日 |
| **L3: 通信** | + `PubSub`, `Service.Invoke` | イベント駆動、サービス間呼び出し、冪等性 | 1 週間 |
| **L4: 業務制御** | + `Workflow`, `Decision`, `Audit` | Saga パターン、ルールエンジン、監査証跡 | 1–2 週間 |
| **L5: 運用最適化** | + `Feature`, `Settings` | Feature Flag、段階ロールアウト、設定同期 | 随時 |

### 2.2 各レベルの構成物

各レベルに対応する以下の 3 点セットを提供する。

| 構成物 | 内容 | 目的 |
|---|---|---|
| **サンプルサービス** | そのレベルの API だけを使う小規模な動作サービス | 「写経」で手を動かして覚える |
| **橋渡しドキュメント** | 従来技術 (C# / REST) との対比表 (後述 3 章) | 未知を既知に紐付ける |
| **確認課題** | サンプルを改変して完成させる小課題 | 理解度を自己確認する |

### 2.3 サンプルサービスの配置

```
examples/
├── tier2-reference-service/    # 既存: 全 API 網羅の模範サービス (最終形)
├── onboarding-L1/              # Log + Telemetry + Auth のみ
├── onboarding-L2/              # L1 + State + Secrets
├── onboarding-L3/              # L2 + PubSub + Service.Invoke
├── onboarding-L4/              # L3 + Workflow + Decision + Audit
└── onboarding-L5/              # L4 + Feature + Settings (= reference と同等)
```

各サンプルは前レベルの知識だけで読める構成とする。L5 完了後にリファレンス実装を読むと「全体像がつながる」体験を設計する。

### 2.4 雛形生成 CLI との統合

雛形生成 CLI に `--level` オプションを追加する。

```bash
# L1 向け: Log / Telemetry / Auth のみ初期化済み
k1s0-scaffold --service-name my-service --language csharp --level 1

# L3 向け: L1 + L2 + PubSub / Service.Invoke 初期化済み
k1s0-scaffold --service-name my-service --language csharp --level 3

# 本番向け (デフォルト): 全 API 初期化済み
k1s0-scaffold --service-name my-service --language csharp
```

`--level` 指定時は不要な API の DI 登録とサンプルコードを除外し、tier2 開発者の視界に入る情報量を制限する。本番サービス立ち上げ時は `--level` なし (全 API 有効) を使用する。

---

## 3. 既知概念からの橋渡しドキュメント

### 3.1 方針

tier2 開発者の大半は C# + REST + SQL Server のバックグラウンドを持つ ([ペルソナ調査](../01_背景と目的/01_ペルソナ.md)に基づく)。新概念を「既知の概念との差分」として説明し、学習曲線の断崖を緩和する。

### 3.2 API 別マッピング表

各 tier1 API のドキュメントに「従来はこう書いていた → k1s0 ではこうなる」セクションを追加する。

**ログ**:

| 従来 (C# + ILogger) | k1s0 |
|---|---|
| `_logger.LogInformation("注文受領: {OrderId}", orderId);` | `k1s0.Log.Info("注文受領", new { orderId, userId });` |
| テンプレート文字列で自由形式 | 構造化フィールドが必須。メッセージは固定文字列 |
| ログレベルは開発者判断 | `Info` / `Warn` / `Error` の使い分け基準を tier1 が定義 |

**データ取得**:

| 従来 (Entity Framework) | k1s0 |
|---|---|
| `var order = await _db.Orders.FindAsync(orderId);` | `var order = await k1s0.State.GetAsync<Order>("orders", orderId);` |
| DB に直接 SQL 発行 | KV ストア (Valkey) への操作。RDB は別途自サービス DB に直接接続 |
| `SaveChanges()` でトランザクション | `SaveAsync()` は単一キー操作。複数キーの原子性は Workflow で担保 |

**サービス間通信**:

| 従来 (HttpClient + REST) | k1s0 |
|---|---|
| `var res = await _http.GetAsync("https://inventory-api/stock/123");` | `var res = await k1s0.Service.Invoke<StockResponse>("inventory", "GetStock", req);` |
| URL を直接指定 | サービス名で解決。URL は tier1 が管理 |
| リトライ・タイムアウトを Polly 等で自前実装 | tier1 がポリシーを内包。呼び出し側では設定不要 |

**イベント発行**:

| 従来 (なし / DB トリガー) | k1s0 |
|---|---|
| `INSERT INTO outbox_table ...` または直接 DB 変更通知 | `await k1s0.PubSub.PublishAsync("order-events", "created", order);` |
| 同期的。発行先を意識しない | 非同期。購読者がいつ処理するかは保証しない (結果整合性) |
| 購読者はなし (DB ポーリング) | 購読者は `k1s0.PubSub.Subscribe` でイベントを受信 |

**認証**:

| 従来 (ASP.NET Identity / AD 認証) | k1s0 |
|---|---|
| `[Authorize(Roles = "Admin")]` | `k1s0.Auth.RequireRole("admin")` または gRPC interceptor |
| Cookie ベースのセッション管理 | JWT (Keycloak 発行) の検証。セッションレス |
| 自前でユーザーテーブル管理 | Keycloak が管理。tier2 はトークン検証のみ |

**設定管理**:

| 従来 (appsettings.json) | k1s0 |
|---|---|
| `Configuration["ConnectionStrings:Default"]` | `k1s0.Settings.GetAsync<string>("connection-timeout")` または ConfigMap 環境変数 |
| ファイルをデプロイ時に差し替え | ConfigMap / Secret を k8s で管理。コード内ハードコード禁止 |

**ワークフロー (Saga)**:

| 従来 (DB トランザクション) | k1s0 |
|---|---|
| `using var tx = await _db.BeginTransactionAsync();` | `await k1s0.Workflow.StartAsync("order.approval", input);` |
| 全操作を 1 トランザクションで原子的に実行 | 各ステップを独立実行し、失敗時は補償アクションで巻き戻す |
| ロールバックは DB が自動実行 | 補償ロジックは tier2 開発者が定義 (ただし実行は Workflow エンジンが保証) |

### 3.3 エラーハンドリング方針の橋渡し

従来の「全て try-catch で捕捉する」パターンから、tier1 API 種別ごとのエラー方針への移行を説明する。

| API 種別 | 方針 | 従来の対応 | k1s0 での対応 | tier2 がすること |
|---|---|---|---|---|
| 観測系 (`Log`, `Telemetry`, `Audit`) | fire-and-forget | try-catch で握りつぶし | tier1 が自動で無視。例外を投げない | 何もしない |
| 参照系 (`State.Get`, `Settings`, `Feature`) | cache-fallback | try-catch + 独自フォールバック | tier1 がキャッシュから自動返却 | 戻り値が陳腐化している可能性を考慮 |
| 状態変更系 (`State.Save`, `PubSub.Publish`) | fail-fast | try-catch + リトライ | tier1 がリトライ後に例外を投げる | 例外を上位に伝搬し、呼び出し元で対処 |
| ワークフロー系 (`Workflow.Start`) | 補償 + リトライ | トランザクションロールバック | Workflow エンジンが自動リトライ + 補償 | 補償アクションを定義 |

### 3.4 gRPC / Protobuf の導入タイミング

tier2 開発者が gRPC / Protobuf に接触するタイミングを習熟度レベルに合わせて制御する。

| レベル | Protobuf との関係 | 作業内容 |
|---|---|---|
| L1–L2 | **生成済みクライアントを呼ぶだけ** | tier1 が提供するクライアントライブラリの型を使用。Proto ファイルには触れない |
| L3 | **自サービスの API 定義を開始** | tier3 向けに公開する API の `.proto` を定義。橋渡しドキュメントで「REST 定義 → Proto 定義」の対比を提供 |
| L4–L5 | **Protobuf のライフサイクルを理解** | フィールド追加・非推奨化・`buf lint` / `buf breaking` の運用 |

---

## 4. 段階的導入

| フェーズ | 施策 | 備考 |
|---|---|---|
| Phase 2 開始前 | L1–L2 サンプルサービスと橋渡しドキュメントを整備 | tier2 開発者参画の前提条件 |
| Phase 2 | L3–L4 サンプルを追加。雛形 CLI に `--level` オプション追加 | Kafka / Temporal の本格運用開始に合わせる |
| Phase 3 | L5 サンプルを追加。リファレンス実装を L5 サンプルとの差分で再整理 | チーム拡大に合わせてオンボーディング教材を完成 |

---

## 関連ドキュメント

- [`00_tier1のスコープ.md`](./00_tier1のスコープ.md) — tier1 公開 API の一覧
- [`03_API設計原則.md`](./03_API設計原則.md) — リファレンス実装と多層防御
- [`06_クライアントライブラリ配布戦略.md`](./06_クライアントライブラリ配布戦略.md) — Protobuf 自動生成とクライアント配布
- [`08_診断機能設計.md`](./08_診断機能設計.md) — 診断コンテキストとエラー切り分け
- [`09_デバッグ支援ツール.md`](./09_デバッグ支援ツール.md) — Tilt ダッシュボードとランブック
- [`../01_背景と目的/01_ペルソナ.md`](../01_背景と目的/01_ペルソナ.md) — tier2 開発者のペルソナ
- [`10_Saga補償パターン支援.md`](./10_Saga補償パターン支援.md) — Saga 補償パターンの階層化と習熟度レベルとの統合
- [`11_ローカル開発CLI設計.md`](./11_ローカル開発CLI設計.md) — `k1s0 dev` CLI によるローカル開発環境操作
- [`12_開発者アダプション戦略.md`](./12_開発者アダプション戦略.md) — ワークショップ・ファーストコンタクト体験・チャンピオン制度
- [`14_tier3開発者体験設計.md`](./14_tier3開発者体験設計.md) — tier3 向けの専用オンボーディングパス (T1-T3) とフレームワーク SDK
