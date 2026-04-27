# 05. k1s0 CLI 利用（暫定版）

本ファイルは IPA 共通フレーム 2013 の **7.1.2.4 利用者文書（暫定版）の作成** に対応する利用者文書のうち、k1s0 CLI（`k1s0` コマンド）の利用手引き初版である。CLI は Rust 実装で、雛形生成・ローカル開発補助・運用補助 の 3 役割を担う。

## 本ファイルの位置付け

開発者・運用者が日々 k1s0 基盤と対話する際、「Backstage ポータルを毎回ブラウザで開く」「kubectl で Dapr Component を直接操作する」といった方式は、前者は頻繁操作で手間が増え、後者は tier1 の内部抽象を破るため禁止されている。これらの隙間を埋めるのが `k1s0` CLI である。開発者は `k1s0 new` で新規サービスを立ち上げ、`k1s0 dev up` でローカル起動し、`k1s0 feature` で Flag を切り替える。運用者は `k1s0 secret rotate` でシークレットローテーションし、`k1s0 audit query` で監査ログを検索する。

CLI は Rust で実装する。選定理由は構想設計 ADR-TIER1-001 で確定した「自作領域は Rust」に準拠する点、単一バイナリで依存ランタイムが不要である点、クロスプラットフォーム（Windows / macOS / Linux）のバイナリ提供が容易である点、tier1 自作領域と同一言語でコード再利用が可能である点の 4 点である。

本章は リリース時点で 「サブコマンド体系・認証方式・出力形式・インストール経路」を確定し / 基本コマンド群を実装、採用後の運用拡大時 で拡張コマンド群を追加する計画である。各段階 での完成度を明示することで、未実装のコマンドを「使えるはず」と誤解されるリスクを抑える。

## 対象読者と前提スキル

対象読者は tier2 / tier3 開発者と、採用側組織の情シス運用者の両方である。前提スキルはターミナル（PowerShell / bash / zsh）の基本操作、Git の基礎、環境変数の概念である。CLI 内部の Rust 実装の知識は一切不要である。

読者はまず `k1s0 doctor` で環境前提を確認する。CLI がローカル環境の不足（Docker 未インストール、Dev Container 未対応のエディタ、kubeconfig 未設定）を検出し、修復ガイドを案内する。

## 設計項目 DS-SW-DOC-140 サブコマンド体系 — 階層化された 8 グループ

`k1s0` CLI のサブコマンドは 8 グループに階層化する。`k1s0 <group> <action> [options]` の書式で統一し、`--help` で各層のヘルプを自動生成する。

- **`new`**: Backstage Template 相当の雛形をローカル生成。オフラインで動作、Backstage 接続不要。
- **`dev`**: ローカル開発補助。`dev up` で Tilt 連動起動、`dev down` で停止、`dev logs` でログ集約表示。
- **`login`** / **`logout`**: OIDC 認証。Keycloak と連携し、Device Code flow で認証完了。
- **`secret`**: OpenBao と tier1 Secrets API を仲介。`secret get` / `secret put` / `secret rotate`。
- **`state`**: tier1 State API の CLI ラッパ。`state get` / `state set` / `state delete` / `state list`。
- **`feature`**: flagd と tier1 Feature API を仲介。`feature enable` / `feature disable` / `feature status`。
- **`audit`**: Audit-Pii API の検索フロントエンド。`audit query` / `audit export`。
- **`doctor`**: 環境前提の自己診断。Docker 確認、kubeconfig 確認、Keycloak 接続確認、Backstage 接続確認。

グループ分割の基準は「同じ tier1 API 種別に紐付くコマンドをまとめる」である。tier1 の 11 API のうち、CLI 経由で直接操作する意味があるのは Secrets / State / Feature / Audit の 4 系統である。PubSub / Workflow / Decision / Service Invoke / Telemetry / Binding / Log の 7 系統はアプリ組み込みが主用途のため、CLI からは状態確認のみを提供する（今後の拡張範囲）。

## 設計項目 DS-SW-DOC-141 認証 — OIDC Device Code Flow

CLI の認証は Keycloak の OIDC Device Code Flow で実装する。CLI はユーザが Web ブラウザを開けないターミナル限定環境（SSH 経由・CI ランナー内・Docker コンテナ内）でも認証できる必要があるため、Device Code Flow を選択する。

認証フローは以下である。ユーザが `k1s0 login` を実行すると、CLI が Keycloak に Device Code を要求し、ユーザに URL とコードを表示する。ユーザは別端末のブラウザで URL を開き、コードを入力して AD 認証を完了する。CLI 側はポーリングで認証完了を検知し、取得したアクセストークン・リフレッシュトークンをローカルの `~/.k1s0/credentials.json` に保存する（ファイル権限 0600）。

トークンの有効期限はアクセストークン 1 時間、リフレッシュトークン 24 時間とする。期限切れ時は CLI が自動でリフレッシュし、失敗時のみ再ログインを要求する。

機密端末（共有 PC・貸与端末）での利用を想定し、`k1s0 logout` でローカル保存のトークンを即時削除可能とする。さらに `--session-only` オプションで環境変数にのみトークンを保存し、プロセス終了時に破棄するモードも提供する。

## 設計項目 DS-SW-DOC-142 出力形式 — JSON / YAML / Table の切替

CLI の出力形式は `--output=json` / `--output=yaml` / `--output=table`（デフォルト）で切替可能とする。パイプ連携・スクリプト組み込みでは JSON が便利、人間閲覧では Table が読みやすい、設定ファイルとして保存するには YAML が適切、という使い分けを想定する。

JSON 出力は機械可読を徹底し、色付けやスペース整形は抑えめにする。ただし `jq` との連携を想定して最低限のインデントは付ける。YAML 出力は k8s マニフェストと同等のスキーマに揃え、そのまま kubectl apply で流用できる形を目指す（ただし直接 kubectl apply は禁止ポリシーのため、GitOps 経由での反映を案内する）。Table 出力は幅自動調整し、狭い端末では列を省略する。

エラー出力は全て stderr に送り、exit code は 0（成功）/ 1（一般エラー）/ 2（引数エラー）/ 3（認証エラー）/ 4（ネットワークエラー）/ 5（サーバエラー）で分類する。シェルスクリプトからの制御が可能となるよう、exit code の分類を固定化する。

## 設計項目 DS-SW-DOC-143 代表コマンドの使用例

代表的な 6 コマンドの使用例を以下に示す。リリース時点で は疑似出力 / 実出力に差し替える。

**`k1s0 new`** — tier2 Go サービスの雛形をローカル生成する。`--dry-run` で生成予定ファイル一覧のみ表示。

```text
$ k1s0 new tier2-microservice --name order-service --lang go --team team-order
Generated: order-service/
  - .github/workflows/ci.yml
  - .devcontainer/devcontainer.json
  - Dockerfile
  - cmd/order-service/main.go
  - go.mod / go.sum
  - docs/README.md
Next: cd order-service && k1s0 dev up
```

**`k1s0 dev up`** — Tilt 連動でローカル起動。tier1 モック・Dapr sidecar・自コンテナを同時起動。

```text
$ k1s0 dev up
[dev] Starting Tilt dashboard at http://localhost:10350
[dev] k1s0 tier1 mock: ready (11 APIs on port 50051)
[dev] daprd sidecar: ready (port 3500)
[dev] order-service: ready (port 8080)
[dev] All services ready. Press Ctrl+C to stop.
```

**`k1s0 state get`** — tier1 State API 経由で Valkey の値を取得。

```text
$ k1s0 state get --key order-123 --tenant jtc-demo
{
  "key": "order-123",
  "value": { "orderId": "123", "amount": 5000 },
  "etag": "abc",
  "metadata": { "ttl": "3600s" }
}
```

**`k1s0 secret rotate`** — OpenBao 上のシークレットをローテート。運用者権限が必要。

```text
$ k1s0 secret rotate --name jwt-signing-key
Rotating secret 'jwt-signing-key'...
New version created: v5 (previous: v4)
Tier1 Secrets API will propagate in 30 seconds.
Audit logged: secret.rotate event recorded.
```

**`k1s0 feature enable`** — Feature Flag を有効化。段階ロールアウト指定可能。

```text
$ k1s0 feature enable --flag new-checkout --rollout 10%
Feature 'new-checkout' set to enabled=true at 10% rollout.
flagd propagation: ~5 seconds.
Audit logged: feature.change event recorded.
```

**`k1s0 audit query`** — Audit-Pii API で監査ログを検索。tenant / user / 時間範囲で絞込。

```text
$ k1s0 audit query --user user-001 --since "24h ago"
2026-04-20 09:15:23 | user-001 | secret.rotate | name=jwt-signing-key
2026-04-20 10:22:18 | user-001 | feature.change | flag=new-checkout, enabled=true
2026-04-20 14:03:41 | user-001 | state.delete | key=temp-xxx
3 events. hash_chain: OK (validated 3/3).
```

各コマンドは `--help` で詳細オプションと出力形式を確認可能。`--output=json` 追加で機械可読形式に切り替わる。

## 設計項目 DS-SW-DOC-144 インストール経路 — OS 別パッケージマネージャ対応

CLI は以下 5 経路で配布する。採用側組織の社内の OS 比率（Windows 70% / macOS 20% / Linux 10%）を考慮し、各 OS の標準パッケージマネージャに対応する。

- **Windows**: `scoop install k1s0` または `winget install k1s0`
- **macOS**: `brew install k1s0/tap/k1s0`（採用側組織専用 Homebrew tap）
- **Linux (Debian / Ubuntu)**: `apt install k1s0`（採用側組織内部 APT リポジトリ経由）
- **Linux (RHEL / CentOS)**: `dnf install k1s0`（採用側組織内部 DNF リポジトリ経由）
- **単体バイナリ**: GitHub Releases から tar.gz / zip をダウンロードし、任意のディレクトリに配置

いずれの経路でも、インストール後に `k1s0 --version` で正しいバージョンが表示されることを確認可能とする。バージョン番号は `major.minor.patch` の SemVer に準拠し、破壊的変更は major アップで明示する。

自動更新は、`k1s0 self update` コマンドで最新版をチェック・更新する。強制更新は行わず、ユーザの操作を必須とする。CI 環境では固定バージョンを使う前提で、自動更新は無効化される。

## 設計項目 DS-SW-DOC-145 セキュリティと権限境界

CLI の操作は、認証済みユーザの Keycloak ロールに応じて制限される。ロールを持たないユーザが特権操作（`secret rotate` / `feature enable` / `audit export`）を実行した場合、CLI は 403 を受けて操作を拒否する。

ローカル保存のトークン（`~/.k1s0/credentials.json`）の保護は、ファイル権限 0600 + 採用側組織の社内端末の Windows Hello / macOS Touch ID によるデバイス認証を リリース時点 で追加する。共有端末からのトークン流出を防ぐため、`k1s0 logout` を自動化するスクリーンセーバ連動も検討する。

CLI 自身の改ざん防止として、コード署名（Windows は Microsoft Authenticode、macOS は Apple Developer ID）を全プラットフォームで実施する。署名なしバイナリは OS レベルで実行をブロックされる。

運用者が代行操作を行う場面（本人不在時の緊急対応）では、ユーザ間の権限委譲を Keycloak の token-exchange 機能で実現する。委譲操作は監査ログに明示的に記録し、後から逆引き可能とする。

## 設計項目 DS-SW-DOC-146 段階別完成度

CLI は リリース時点 で基本コマンド群、採用後の運用拡大時 で拡張を実装する段階計画である。

- **リリース時点（採用検討時点）**: 本章骨子のみ。実装着手前。
- **採用初期 (基本 4 コマンド)**: `new` / `dev` / `login` / `doctor` の 4 グループを実装。scoop / brew / 単体バイナリのみ配布。
- **採用初期 (state/feature/secret 追加)**: `state` / `feature` / `secret get / put` を追加。apt / dnf 配布を開始。
- **採用初期 (監査拡充)**: `secret rotate` / `audit query` / `audit export` を追加。監査対応強化。
- **採用後の運用拡大時**: PubSub / Workflow / Decision / Telemetry 系の状態確認コマンドを追加。AI 補助（`k1s0 explain`）検討。

各段階 移行時に、CLI の後方互換性を CI で検証する。前段階 のコマンドが 段階移行後に動かなくなる破壊的変更は major バージョンアップ時のみに限定する。

## 設計項目 DS-SW-DOC-147 CLI 設計思想 — Unix 哲学への準拠

CLI は Unix 哲学（小さく完結、テキスト入出力、パイプ連携）に準拠する。具体的には以下 5 原則を設計基準とする。

- **単一目的**: 1 コマンドは 1 責務に留める。`k1s0 do-everything` のような統合コマンドは作らない。
- **テキスト入出力**: 入力は引数または stdin、出力は stdout（結果）と stderr（メッセージ）の 2 経路に統一。
- **パイプ連携**: `k1s0 state list | grep foo | k1s0 state delete --stdin` のような連結が可能な設計。
- **べき等性**: 同じ操作を繰り返し実行しても同じ結果となる設計。変更系コマンドは `--force` なしでは確認プロンプトを出す。
- **失敗時の明確なエラー**: 失敗時は exit code と人間可読なエラーメッセージを両方返す。

この哲学は、シェルスクリプトや CI パイプラインへの組み込みを容易にする。GUI 操作だけでなくスクリプト自動化が可能であることは、運用自動化の前提であり、企画で約束した「採用側の小規模運用」の成立条件の一つである。

## 対応要件一覧

本ファイルで採番した設計 ID（`DS-SW-DOC-140` 〜 `DS-SW-DOC-147`）と、充足する要件 ID を以下に列挙する。

- `DS-SW-DOC-140`（サブコマンド体系）: `DX-LD-003` / `DX-LD-004` / `BR-PLATUSE-012`
- `DS-SW-DOC-141`（OIDC Device Code Flow）: `NFR-E-AC-001` / `NFR-E-ENC-001`
- `DS-SW-DOC-142`（出力形式 JSON / YAML / Table）: `DX-LD-005`
- `DS-SW-DOC-143`（代表コマンド使用例）: `DX-LD-006` / `FR-T1-STATE-001` / `FR-T1-SECRETS-001` / `FR-T1-FEATURE-001` / `FR-T1-AUDIT-003`
- `DS-SW-DOC-144`（インストール経路）: `DX-LD-007` / `BR-PLATUSE-013`
- `DS-SW-DOC-145`（セキュリティと権限境界）: `NFR-E-AC-002` / `NFR-E-AC-004`
- `DS-SW-DOC-146`（段階別完成度）: 間接対応（段階進行管理）
- `DS-SW-DOC-147`（Unix 哲学準拠）: `DX-LD-008` / 企画「採用側の小規模運用」コミット
