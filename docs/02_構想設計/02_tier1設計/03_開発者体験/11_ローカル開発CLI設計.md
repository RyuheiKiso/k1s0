# ローカル開発 CLI 設計

## 目的

tier1 は「tier2 / tier3 から infra を隠蔽する」と宣言しているが、ローカル開発環境の Tilt 自体が Kubernetes 上で動作するため、**k8s のエラーメッセージや概念が tier2 開発者に素通りで届く**問題がある。Pod、Container、Sidecar、CrashLoopBackOff 等の k8s 用語がエラーに現れた時点で「隠蔽されていない」という不信感が生まれる。

本資料では、Tilt / k8s を完全に隠蔽する **`k1s0 dev` CLI** を定義する。tier2 開発者が日常的に触るコマンドから k8s 用語を排除し、エラーの自動翻訳と自動復旧を提供する。

---

## 1. 設計原則

| 原則 | 内容 |
|---|---|
| **k8s 用語の完全排除** | `k1s0 dev` の出力に Pod / Container / Namespace / Deployment / Sidecar 等の k8s 用語を含めない |
| **Tilt の隠蔽** | tier2 開発者は `tilt` コマンドを直接実行しない。`k1s0 dev` が内部で Tilt を操作する |
| **エラーの翻訳** | k8s レベルのエラーを業務開発者の語彙に翻訳して表示する |
| **自動復旧** | よくある環境問題を検知して自動的に復旧する |
| **上級者への道を塞がない** | `k1s0 dev --verbose` で内部の k8s / Tilt 情報を表示可能にする |

---

## 2. コマンド体系

### 2.1 基本コマンド

| コマンド | 動作 | 内部で実行されること |
|---|---|---|
| `k1s0 dev start` | ローカル開発環境を起動 | `tilt up` + tier1 基盤コンポーネント起動待ち |
| `k1s0 dev stop` | ローカル開発環境を停止 | `tilt down` + リソースクリーンアップ |
| `k1s0 dev status` | 全コンポーネントの状態表示 | `kubectl get pods` の結果を翻訳して表示 |
| `k1s0 dev logs <service>` | 指定サービスのログ表示 | `kubectl logs` の結果をフィルタして表示 |
| `k1s0 dev restart <service>` | 指定サービスを再起動 | `kubectl rollout restart` |

### 2.2 設定管理コマンド

| コマンド | 動作 | 内部で実行されること |
|---|---|---|
| `k1s0 dev config list <service>` | サービスの設定値一覧 | ConfigMap の内容を表示 |
| `k1s0 dev config set <service> <key> <value>` | 設定値の変更 (ローカル限定) | ConfigMap の更新 + Pod 再起動 |
| `k1s0 dev secrets list <service>` | シークレット一覧 (値はマスク) | Secret の内容をマスク表示 |

### 2.3 診断コマンド

| コマンド | 動作 | 内部で実行されること |
|---|---|---|
| `k1s0 dev health` | tier1 バックエンドの健全性一括確認 | `k1s0.Diag.HealthCheckAsync()` の呼び出し |
| `k1s0 dev dashboard` | Tilt 診断ダッシュボードをブラウザで開く | Tilt Web UI の URL を開く |
| `k1s0 dev trace <correlation-id>` | 指定 ID の分散トレースを表示 | ローカル Grafana Tempo への問い合わせ |

### 2.4 ビルドコマンド

| コマンド | 動作 | 内部で実行されること |
|---|---|---|
| `k1s0 dev rebuild <service>` | 指定サービスを強制再ビルド | Tilt のビルドキャッシュクリア + 再ビルド |
| `k1s0 dev rebuild --all` | 全サービスを強制再ビルド | 全リソースの再ビルド |

---

## 3. `k1s0 dev status` の出力設計

### 3.1 正常時の出力

```
k1s0 ローカル環境ステータス
────────────────────────────
  my-service           稼働中   ポート: 8080
  tier1-gateway        稼働中   ポート: 9090
  データストア         稼働中
  メッセージブローカー 稼働中
  認証サーバー         稼働中   UI: http://localhost:8180

  全コンポーネント正常
```

k8s 用語の排除: `valkey` → `データストア`、`kafka` → `メッセージブローカー`、`keycloak` → `認証サーバー` と表示する。tier2 開発者が知る必要のない実装詳細は隠す。

### 3.2 異常時の出力

```
k1s0 ローカル環境ステータス
────────────────────────────
  my-service           起動失敗   直近のエラー: ポート 8080 が別プロセスで使用中
  tier1-gateway        稼働中     ポート: 9090
  データストア         稼働中
  メッセージブローカー 起動中...  (初回起動に 30 秒程度かかります)
  認証サーバー         稼働中     UI: http://localhost:8180

  対処が必要なコンポーネントがあります
  詳細: k1s0 dev logs my-service
```

### 3.3 tier1 基盤障害時の出力

```
k1s0 ローカル環境ステータス
────────────────────────────
  my-service           稼働中    ポート: 8080
  tier1-gateway        異常      自動復旧を試行中...
  データストア         異常      自動復旧を試行中...
  メッセージブローカー 稼働中
  認証サーバー         稼働中    UI: http://localhost:8180

  tier1 基盤に問題が発生しています (あなたのコードの問題ではありません)
  自動復旧を試行中です。解消しない場合: k1s0 dev restart --infra
```

**「あなたのコードの問題ではありません」**の一文により、tier2 開発者が自分のコードを疑って無駄な調査に時間を費やすことを防ぐ。

---

## 4. エラー自動翻訳

### 4.1 翻訳ルール

k8s / Tilt レベルのエラーを検知し、tier2 開発者向けのメッセージに翻訳する。

| k8s のエラー | 翻訳後のメッセージ | 補足情報 |
|---|---|---|
| `CrashLoopBackOff` | `{service} が起動に失敗しています` | 直近のアプリケーションエラーログを併記 |
| `ImagePullBackOff` | `{service} のビルドイメージが見つかりません` | `k1s0 dev rebuild {service} を実行してください` |
| `OOMKilled` | `{service} がメモリ不足で停止しました` | `処理するデータ量を確認するか、k1s0 dev restart {service} を試してください` |
| `CreateContainerConfigError` | `{service} の設定に問題があります` | 不正な設定項目を特定して表示 |
| `ErrImageNeverPull` | `{service} のビルドが完了していません` | `k1s0 dev rebuild {service} を実行してください` |
| `Evicted` | `ローカル環境のディスク容量が不足しています` | `Docker のディスク使用量を確認してください` |
| Connection refused (`valkey:6379`) | `データストアが起動中です` | `k1s0 dev status で全体の状態を確認してください` |
| Connection refused (`kafka:9092`) | `メッセージブローカーが起動中です` | 同上 |
| Dapr sidecar not ready | `tier1 基盤コンポーネントが起動中です` | `30 秒後に自動リトライします...` |

### 4.2 翻訳できないエラー

翻訳ルールに該当しないエラーは以下のフォーマットで表示する。

```
tier1 基盤の内部エラーが発生しました (あなたのコードの問題ではありません)

以下の情報を tier1 チームに共有してください:
  エラーコード: INFRA_UNKNOWN_ERROR
  タイムスタンプ: 2026-04-13T10:00:00+09:00
  CorrelationId: abc-123-def-456
  詳細ログ: k1s0 dev logs --internal
```

tier2 開発者に「自分で調べろ」と言わず、**「これを共有すれば tier1 チームが対処できる」**という明確な行動指示を与える。

### 4.3 `--verbose` モード

k8s / Tilt の生の情報が必要な場合 (tier1 チームの調査時など) は `--verbose` フラグで表示する。

```bash
# 通常 (tier2 開発者向け)
k1s0 dev status

# 詳細 (tier1 / infra チーム向け)
k1s0 dev status --verbose
```

`--verbose` 時は k8s の Pod 名、ステータス、イベント、Dapr サイドカーの状態など、生の情報を表示する。

---

## 5. 自動復旧

### 5.1 自動復旧対象

よくあるローカル環境の問題を `k1s0 dev` が自動検知し、復旧を試みる。

| 検知する問題 | 自動復旧アクション | tier2 開発者への表示 |
|---|---|---|
| Dapr サイドカーがクラッシュ | Pod を自動再起動 | `tier1 基盤コンポーネントを自動復旧しました` |
| Valkey が応答しない | Valkey コンテナを再起動 | `データストアを再起動しました` |
| Kafka ブローカーが応答しない | Kafka コンテナを再起動 | `メッセージブローカーを再起動しました` |
| ビルドキャッシュ破損 | キャッシュをクリアして再ビルド | `ビルドキャッシュをリセットしました。再ビルド中...` |
| ポートが既に使用中 | 競合プロセスを表示して選択肢を提示 | `ポート 8080 は別のプロセスが使用中です。停止しますか? [Y/n]` |
| Docker Desktop が停止 | エラーと起動手順を表示 | `Docker が起動していません。Docker Desktop を起動してください` |

### 5.2 自動復旧の制限

| 制限 | 理由 |
|---|---|
| 自動復旧は **最大 3 回** まで | 無限リトライによるリソース消費を防止 |
| 3 回失敗後は手動対処を案内 | `k1s0 dev restart --infra` または tier1 チームへの連絡を案内 |
| tier2 サービスの自動再起動は **行わない** | tier2 のコードのバグを自動復旧で隠蔽しない |

### 5.3 `k1s0 dev restart --infra`

tier1 基盤コンポーネントのみを一括再起動する。tier2 サービスのデータや状態には影響しない。

```bash
# tier1 基盤のみ再起動 (tier2 サービスは維持)
k1s0 dev restart --infra
```

```
tier1 基盤を再起動しています...
  tier1-gateway        再起動完了
  データストア         再起動完了
  メッセージブローカー 再起動完了
  認証サーバー         再起動完了

  基盤の再起動が完了しました。サービスの動作を確認してください。
```

---

## 6. 設定管理の抽象化

### 6.1 `k1s0.config.yaml`

tier2 開発者が k8s の ConfigMap / Secret を直接操作する必要をなくすため、サービスの設定を **`k1s0.config.yaml`** ファイルで管理する。雛形生成 CLI がこのファイルを自動生成し、`k1s0 dev` がローカル環境で k8s リソースに変換する。

```yaml
# k1s0.config.yaml (tier2 開発者が編集する)
service:
  name: my-service
  port: 8080

config:
  ORDER_TIMEOUT: 30s
  MAX_RETRY_COUNT: "3"
  FEATURE_NEW_UI: "true"

secrets:
  DB_CONNECTION_STRING: "ref:local/my-service-db"
  API_KEY: "ref:local/external-api-key"
```

### 6.2 環境別の設定解決

| 環境 | `k1s0.config.yaml` の解決方法 |
|---|---|
| ローカル (Tilt) | `k1s0 dev` が ConfigMap / Secret に変換して適用 |
| ステージング / 本番 | CI/CD パイプラインが ConfigMap / Secret に変換してデプロイ |

tier2 開発者は `k1s0.config.yaml` だけを編集すればよく、ConfigMap / Secret の YAML 構文を学ぶ必要がない。

### 6.3 シークレットのローカル管理

ローカル環境のシークレットは `k1s0 dev secrets` コマンドで管理する。

```bash
# ローカル用シークレットの設定
k1s0 dev secrets set my-service DB_CONNECTION_STRING "Server=localhost;..."

# 設定済みシークレットの確認 (値はマスク)
k1s0 dev secrets list my-service
```

出力:
```
my-service のシークレット
─────────────────────────
  DB_CONNECTION_STRING   設定済み   ****
  API_KEY                設定済み   ****
```

---

## 7. 雛形生成 CLI との統合

雛形生成 CLI (`k1s0-scaffold`) が出力するプロジェクト構造に `k1s0 dev` 向けのファイルを自動生成する。

| 生成されるファイル | 内容 | tier2 が編集するか |
|---|---|---|
| `k1s0.config.yaml` | サービス設定 | はい (設定値の追加・変更) |
| `Tiltfile` | Tilt 設定 (k1s0 dev が内部で使用) | いいえ (自動生成のまま) |
| `deploy/base/*.yaml` | k8s マニフェスト | いいえ (自動生成のまま) |
| `.k1s0/local-secrets.yaml` | ローカルシークレット (gitignore 対象) | `k1s0 dev secrets` コマンドで操作 |

tier2 開発者が直接編集するのは `k1s0.config.yaml` のみ。k8s マニフェストや Tiltfile は自動生成されたものをそのまま使用する。

---

## 8. 実装方針

### 8.1 CLI の実装言語

`k1s0 dev` は雛形生成 CLI と同一の Rust バイナリのサブコマンドとして実装する。

```bash
# k1s0 CLI のサブコマンド構成
k1s0 scaffold ...   # 雛形生成 (既存)
k1s0 dev ...         # ローカル開発環境操作 (本資料)
k1s0 saga-test ...   # Saga テストランナー (10_Saga補償パターン支援.md)
```

### 8.2 Tilt との連携

`k1s0 dev` は Tilt をバックエンドとして利用するが、Tilt の API / CLI を直接呼び出すのではなく、Tilt の設定ファイルを生成して `tilt up` / `tilt down` を実行する形式とする。

| 方式 | 理由 |
|---|---|
| Tilt CLI のラップ | Tilt のバージョンアップに追従しやすい |
| Tilt API 直接呼び出し | 不採用。Tilt API の安定性が保証されていない |

### 8.3 エラー翻訳の仕組み

`k1s0 dev` は以下の方法でエラーを収集・翻訳する。

| 情報ソース | 収集方法 | 用途 |
|---|---|---|
| k8s Events | `kubectl get events --watch` | Pod の状態変化を検知 |
| Tilt ログ | `tilt logs` のストリーム | ビルドエラーの検知 |
| アプリケーションログ | `kubectl logs --follow` | tier2 サービスのエラー表示 |

翻訳ルール (4.1 節) はルールファイルとして管理し、新しいエラーパターンが判明するたびに追加する。

---

## 9. 段階的導入

| 段階 | 施策 | 備考 |
|---|---|---|
| 採用初期 | `k1s0 dev start` / `stop` / `status` / `logs` の実装 | tier2 開発開始の前提条件 |
| 採用初期 | エラー自動翻訳の初期ルール (4.1 節) の実装 | 主要エラー 10 件をカバー |
| 採用初期 | 自動復旧 (Dapr / Valkey / Kafka) の実装 | 最小構成で提供開始 |
| 採用後の運用拡大時 | `k1s0 dev config` / `secrets` の実装 | `k1s0.config.yaml` の導入 |
| 採用後の運用拡大時 | `k1s0 dev health` / `dashboard` / `trace` の実装 | 診断機能との統合 |
| 採用側のマルチクラスタ移行時 | `k1s0 dev rebuild` の実装 | ビルドキャッシュ問題の対処 |
| 継続的 | エラー翻訳ルールの追加 | 新しいエラーパターン発見時に随時追加 |

---

## 関連ドキュメント

- [`09_デバッグ支援ツール.md`](../04_診断と補償/09_デバッグ支援ツール.md) — Tilt 診断ダッシュボードとランブック
- [`08_診断機能設計.md`](../04_診断と補償/08_診断機能設計.md) — 診断コンテキストと Diag API
- [`07_tier2オンボーディング戦略.md`](07_tier2オンボーディング戦略.md) — 習熟度レベルとの連動
- [`../../04_CICDと配信/04_ローカル開発環境.md`](../../04_CICDと配信/04_ローカル開発環境.md) — Tilt ローカル開発環境の全体構成
- [`10_Saga補償パターン支援.md`](../04_診断と補償/10_Saga補償パターン支援.md) — Saga テストランナー CLI
- [`../../04_CICDと配信/05_共有開発インフラサーバー.md`](../../04_CICDと配信/05_共有開発インフラサーバー.md) — 共有サーバー導入によるローカル k8s 排除と CLI 内部実装の変更
- [`13_パフォーマンス帰属設計.md`](../04_診断と補償/13_パフォーマンス帰属設計.md) — `k1s0 dev trace` の拡張 (ウォーターフォール出力・遅延リクエスト一覧)
