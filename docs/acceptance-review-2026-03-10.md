# 外注成果物 受入検査報告書

作成日: 2026-03-10

## 判定

不合格（現時点では検収不可）

テストとビルドは一通り通過しているが、`docs` に記載された要求と実装の間に重大な不適合が残っている。特に、実行失敗を成功表示してしまう点と、デプロイ機能の未実装は受入不可の水準である。

## 検査対象

- `README.md`
- `docs/cli/gui/TauriGUI設計.md`
- `docs/cli/flow/CLIフロー.md`
- `CLI/` 配下の Rust/Tauri/React 実装
- `infra/demo/kiali/ui` のデモ UI ビルド

## 重大不適合

### 1. Build / Test / Deploy の成功判定が破綻している

仕様上、実行失敗は失敗として扱われるべきだが、実装は失敗後でも最終的に成功扱いになる。

根拠:

- `CLI/crates/k1s0-core/src/commands/build.rs:124-135`
  - `ProgressEvent::Error` を出しても最後に `Finished { success: true }` を送る。
- `CLI/crates/k1s0-core/src/commands/test_cmd.rs:158-185`
  - テストコマンドの終了コードを失敗として扱わず、最後に `Finished { success: true }` を送る。
- `CLI/crates/k1s0-core/src/commands/deploy.rs:303-345`
  - Docker build 失敗を Warning 扱いに留め、最後に `Finished { success: true }` を送る。
- `CLI/crates/k1s0-gui/ui/src/pages/BuildPage.tsx:45-49`
- `CLI/crates/k1s0-gui/ui/src/pages/TestPage.tsx:50-54`
- `CLI/crates/k1s0-gui/ui/src/pages/DeployPage.tsx:51-55`
  - フロント側は `await` が返れば無条件で `success` を表示する。
- `CLI/crates/k1s0-core/src/commands/build.rs:325-329`
- `CLI/crates/k1s0-core/src/commands/test_cmd.rs:316-332`
  - 不正な対象でも `Finished { success: true }` を正としてテストしている。

受入上の問題:

- 失敗したビルド/テスト/デプロイをオペレータに成功と誤認させる。
- 検収後の運用事故を直接誘発するため、受け入れられない。

是正要求:

- 1件でも失敗したら最終結果を失敗にすること。
- `Finished.success` を実結果に合わせて算出すること。
- GUI は `Finished.success` と `Error` イベントを見て最終状態を決定すること。
- 失敗ケースを成功として固定化しているテストを修正し、回帰テストを追加すること。

### 2. デプロイ実装が仕様と一致していない

`docs` ではデプロイは Docker build だけではなく、push、Cosign 署名、Helm デプロイまで含む。実装はそこに達していない。

根拠:

- `docs/cli/gui/TauriGUI設計.md:87`
  - deploy は「Docker ビルド・プッシュ・Cosign 署名・Helm デプロイ」と定義されている。
- `docs/cli/flow/CLIフロー.md:728`
- `docs/cli/flow/CLIフロー.md:735`
- `docs/cli/flow/CLIフロー.md:753`
- `docs/cli/flow/CLIフロー.md:778`
  - Cosign、Helm deploy、`helm status`、rollback まで運用フローに含まれている。
- `CLI/crates/k1s0-core/src/commands/deploy.rs:236-266`
- `CLI/crates/k1s0-core/src/commands/deploy.rs:297-335`
  - 実際に行っているのは `docker build`、または `dry-run` ログだけである。
- `CLI/crates/k1s0-core/src/commands/deploy.rs:371-375`
  - Dockerfile がなくても deploy 対象として提示される。

受入上の問題:

- 仕様に書かれた主要工程が未実装であり、デモが通っても要件達成にはならない。
- 実運用に必要な署名、配布、リリース、ロールバック確認ができない。

是正要求:

- `docs` 通りに push、Cosign、Helm deploy、prod rollback 導線まで実装すること。
- そこまで実装しないなら、仕様書と受入条件を先に改訂し、範囲縮小を合意すること。
- deploy 対象のスキャン条件と実行条件を一致させること。

### 3. GUI がワークスペース基準で動作しておらず、配布アプリとして成立していない

Tauri GUI はワークスペースを明示的に扱うべきだが、実装は `.` と `current_dir()` に依存している。

根拠:

- `CLI/crates/k1s0-gui/src/commands.rs:57-80`
  - Tauri コマンドは `base_dir` を受け取れる設計になっている。
- `CLI/crates/k1s0-gui/ui/src/pages/BuildPage.tsx:22-24`
- `CLI/crates/k1s0-gui/ui/src/pages/TestPage.tsx:22-29`
- `CLI/crates/k1s0-gui/ui/src/pages/DeployPage.tsx:23-25`
- `CLI/crates/k1s0-gui/ui/src/pages/GeneratePage.tsx:99-102`
  - 実際の UI はすべて `'.'` を渡している。
- `CLI/crates/k1s0-core/src/commands/generate/execute.rs:18-20`
  - generate は `current_dir()` を固定使用している。

受入上の問題:

- パッケージ化した GUI をどのディレクトリから起動したかで挙動が変わる。
- 利用者が対象ワークスペースを切り替えられず、GUI ツールとして不完全。

是正要求:

- ワークスペース選択機能を追加し、選択したルートを全画面で共有すること。
- scan / generate / build / test / deploy の全処理で明示的な `base_dir` を渡すこと。
- `current_dir()` 依存を廃止し、Tauri 側から渡されたワークスペースルートで動作させること。

### 4. GUI 認証フローが仕様書上は必須だが、実装が存在しない

認証は README と GUI 設計に記載されているが、GUI 実装には対応機能がない。

根拠:

- `README.md:69`
- `README.md:376`
  - OIDC PKCE / Device Code を含む認証方針が明記されている。
- `docs/cli/gui/TauriGUI設計.md:197-202`
  - GUI の認証フローとして Device Authorization Grant、候補として Authorization Code + PKCE が記載されている。
- `CLI/crates/k1s0-gui/ui/src/router.tsx:17-80`
  - ルーティングに login/auth 画面がない。
- `CLI/crates/k1s0-gui/src/commands.rs:19-230`
  - 認証開始、コールバック処理、トークン取得/保持に相当する Tauri コマンドがない。

受入上の問題:

- 仕様通りに認証された GUI として使えない。
- 将来対応予定では検収根拠にならない。

是正要求:

- 仕様で要求する認証方式を実装すること。
- 受入対象外にするなら、設計書と README の記載を改訂して範囲を明確化すること。

## 追加懸念

### 開発モード build が `npm run dev` になっている

根拠:

- `CLI/crates/k1s0-core/src/commands/build.rs:52-58`

懸念:

- build 操作で開発サーバー起動を呼ぶのは意味が異なる。
- CI や GUI からの実行で終了しない可能性がある。

是正要求:

- build は成果物生成に限定し、開発サーバー起動は dev コマンドへ分離すること。

## デモ検査結果

- `infra/demo/kiali/ui` の `npm run build` は成功。
- ただし、Kubernetes / Istio / Kiali / Jaeger / Grafana を含むデモ全体の受入シナリオ、期待結果、実行手順書はリポジトリ内で確認できなかった。
- そのため、デモについては静的ビルド確認までであり、運用シナリオの完了確認までは未検証。

## 実施した検証

- `cargo test --workspace` in `CLI` : pass
- `npm test -- --run` in `CLI/crates/k1s0-gui/ui` : pass
- `npx tsc --noEmit` in `CLI/crates/k1s0-gui/ui` : pass
- `npm run build` in `CLI/crates/k1s0-gui/ui` : pass
- `npm run build` in `infra/demo/kiali/ui` : pass

## 受入条件として要求する再提出物

1. 上記重大不適合を是正したコード
2. 失敗時に失敗表示となる回帰テスト
3. deploy の実装範囲を示す実行ログまたはデモ
4. GUI のワークスペース選択と認証動線を確認できるデモ
5. デモ実施手順書と期待結果一覧
