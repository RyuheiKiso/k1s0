## 11. 設定（config）と秘密情報

### framework が提供する設定読み込み機能

framework で設定読み込み機能を提供する（config モジュールと config-service）。

framework 自身の設定は YAML ファイルで制御し、feature 固有の設定値は DB で管理する。環境変数は使用しない。

### 設定の優先順位と耐障害（固定）

YAML（ConfigMap 等で配布するファイル設定）と DB（`fw_m_setting`）を併用する場合、優先順位と障害時挙動を固定する。

優先順位（override 順）（固定）

1. CLI 引数（例: `--config` / `--env` / `--secrets-dir`。参照先指定に限定）
2. YAML（`config/{env}.yaml`。非機密の静的設定）
3. DB（`fw_m_setting`。feature 固有の動的設定）

※ 原則として機密値そのものを YAML/DB へ置かず、参照（`*_file` / secret_ref）に留める。

config-service 障害時の挙動（固定）

- 起動時
	- 取得失敗時は設定単位で次のいずれかを選べる（既定は A）。
		- A: キャッシュがあれば起動可（キャッシュなしなら起動不可）
		- B: フェイルオープン（DB 設定がなくても起動し、YAML 既定値で動作）
		- C: 起動不可（設定取得が必須なサービス/機能に適用）
- 稼働中
	- 取得失敗時は直前のキャッシュ（メモリ/ローカル永続）を使い、一定時間後にリトライする
	- キャッシュ TTL と最大保持世代数は framework が既定値を持ち、YAML/DB 設定で上書き可能にする
	- 失敗時はメトリクス/ログ/トレースで必ず観測できるようにする

setting のスキーマ進化（固定）

- キー変更・廃止
	- 既存キーのリネームは原則禁止（互換層を設ける）。必要なら段階移行
	- 廃止は `status=0`（無効）等で段階的に行い、即時削除しない
	- 廃止時はサービスの README の設定一覧を更新し、移行手順を明記する
- 型変更
	- 既存キーの `value_type` 変更は原則禁止
	- 変更が必要なら新キーを追加し、旧キーは互換のために一定期間維持する
- デフォルト値
	- DB 設定が存在しない場合の既定値は YAML もしくはアプリ側の既定（framework で固定）とし、README で明示する

### 秘密情報の扱い（環境変数禁止の代替）

方針: 秘密情報（DB パスワード、API キー、JWT 秘密鍵等）はファイルとして配布し、YAML/DB には値そのものを置かず参照（ファイルパス/参照名）のみを保持する。

MUST（必須ルール）

- ConfigMap（`config/{env}.yaml`）に秘密情報を直接書かない
- 秘密情報は Kubernetes Secret 等から volume mount してコンテナ内ファイルとして配布する
- アプリは秘密情報を環境変数で受け取らない（envFrom/secretKeyRef を使わない）
- 秘密情報の既定の配置先を固定する（例: `/var/run/secrets/k1s0/`）
- YAML には `*_file` キー（ファイルパス）を置き、値は起動時にファイルから読み込む

推奨（Git 管理する場合）

- 必要があれば SOPS（age/GPG）等で暗号化したファイルとして管理し、CI/CD で復号して Secret を生成する
- Git に置かない運用の場合は External Secrets Operator / Secret Store CSI Driver 等を利用し、volume mount する

K8s での標準配布方式（例）

- `config/{env}.yaml`: ConfigMap（非機密）
- `secret/`: Kubernetes Secret（機密）
- Pod では次を行う
	- ConfigMap を `/etc/k1s0/config/{env}.yaml` に mount
	- Secret を `/var/run/secrets/k1s0/` に mount
	- 起動引数で `--env {env}` と `--config /etc/k1s0/config/{env}.yaml` を渡す（暗黙選択しない）

ローカル開発での標準配布方式（例）

- `config/{env}.yaml` を参照（非機密）
- 機密値は `secrets/.gitignore` 配下に置き、起動時に `--secrets-dir ./secrets/{env}` を指定する
	- 例: `secrets/dev/db_password`, `secrets/dev/jwt_private_key.pem`

YAML キー例（値を置かず参照のみ）

- DB: `db.host`, `db.port`, `db.name`, `db.user`（非機密）, `db.password_file`（機密）
- JWT: `auth.jwt_private_key_file`, `auth.jwt_public_key_file`
- 外部 API: `integrations.some_api.token_file`

---


