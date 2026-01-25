## 2. 前提（開発環境）

### OS / エディタ

- OS: Windows
- エディタ: VSCode

### 独立した開発環境（MUST）

単に Kubernetes namespace で分離するだけでは不十分とし、各チームが追加作業なしで「依存起動・シード・疎通・観測」まで揃うことをテンプレと framework で固定する。

#### ローカル開発の標準（固定）

- Dev Container + Docker Compose を標準とする（Windows/VSCode 前提で環境差を最小化）。
- Dev Container を使えない場合のみ例外としてローカル実行を許可し、差分は `scripts/` のラッパーで吸収する。

#### 依存起動と疎通（ワンコマンド・固定）

テンプレは次を必ず同梱し、1コマンドで起動できる状態を既定とする。

- 依存起動: `scripts/dev-up.ps1`
- 依存停止: `scripts/dev-down.ps1`
- マイグレーション + 初期データ投入: `scripts/dev-seed.ps1`
- 疎通チェック（HTTP/gRPC/DB）: `scripts/dev-check.ps1`

ローカルで起動する依存は最小構成を固定する（例: PostgreSQL / Redis / OTel Collector / Trace UI）。

各サービスは起動引数で `--env dev` と `--config` を明示し、暗黙選択しない。

#### 規約の自動検査（lint）（MUST）

規約を「ドキュメント」だけで終わらせず、テンプレ/CLI/CI で自動的に逸脱を検知して落とす。

- リポジトリおよび各サービスは、次のいずれか（または両方）を必ず提供する。
	- `k1s0 lint`（CLIコマンド）
	- `scripts/lint.ps1`（Windows 向けラッパー。Dev Container 外でも動くこと）

lint の検査対象（最低限）

- ディレクトリ構造・必須ファイルの存在（テンプレ規約との差分）
	- 例: `config/{default,dev,stg,prod}.yaml` / `deploy/base` / `deploy/overlays/{dev,stg,prod}` / `src/{application,domain,infrastructure,presentation}`
- `.k1s0/manifest.json` の整合性（存在・必須キー・managed/protected/suggest_only の妥当性）
- 禁止事項の検出（例）
	- アプリ実装での環境変数参照（例: Rust の `std::env`、Go の `os.Getenv` 等）
	- Kubernetes マニフェストでの `envFrom` / `secretKeyRef` による Secret 注入（原則禁止）
	- ConfigMap（`config/{env}.yaml`）への機密値直書き（`*_file` 参照のみ許可）

#### CIでの必須化（固定）

- CI は `k1s0 lint`（または同等の `scripts/lint.ps1`）を必須チェックとして実行し、失敗時はマージ不可にする。

#### dev namespace 払い出し（Kubernetes・固定）

- チームごとに dev 用 namespace を払い出す（例: `k1s0-{team}-dev`）。
- `deploy/overlays/dev/` に namespace・ホスト名・依存先などの差分を閉じ込め、サービス名で名前解決する。

DB 分離

- 原則: マイクロサービスごとに DB を分離する（dev でも同じ）。
- 共有 DB クラスタを使う場合でも、最低限「namespace（チーム）× service」で DB（または schema）を分離し、衝突しない命名規則を固定する。

Secret 配布（環境変数禁止の代替）

- Secret は Kubernetes Secret 等から volume mount し、`/var/run/secrets/k1s0/` 配下のファイルとして配布する。
- ConfigMap は `/etc/k1s0/config/{env}.yaml` に mount し、YAML には `*_file`（参照）だけを置く。

---


