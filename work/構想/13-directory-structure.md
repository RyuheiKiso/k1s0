## 13. ディレクトリ構成（規約）

### 方針

目的

- リポジトリ内の「どこに何を書くべきか」を固定し、読み手/書き手の解釈差をなくす
- 生成（CLI）・共通化（framework）・個別開発（feature）を衝突なく並行させる

### MUST（必須ルール）

- monorepo とし、責務ごとに次の 3 領域へ分離する
	- `framework/`: 開発基盤チームが提供する共通部品（再利用の唯一の置き場）
	- `feature/`: 個別機能チームのサービス実装（機能単位 = マイクロサービス）
	- `CLI/`: 雛形生成・導入・アップグレード

サービス粒度

- 機能単位 = 1 マイクロサービス
- 1 ディレクトリ = 1 サービス（例）
	- バックエンド（Rust）: `feature/backend/rust/{feature_name}/`
	- バックエンド（Go）: `feature/backend/go/{feature_name}/`
	- フロントエンド（React）: `feature/frontend/react/{feature_name}/`
	- フロントエンド（Flutter）: `feature/frontend/flutter/{feature_name}/`
- サービス間の共通化は `feature/` 配下で行わない。共通にしたいものは `framework/` へ移す

個別サービス（feature）の言語別・必須ファイル/ディレクトリ（固定）

- 共通必須: `README.md`
- バックエンド（Rust）必須: `Cargo.toml`, `config/`, `deploy/`, `src/`
- バックエンド（Go）必須: `go.mod`, `config/`, `deploy/`, `src/`
- フロントエンド（React）必須: `package.json`, `config/`, `src/`
	- 必須: `config/{default,dev,stg,prod}.yaml`（機密値直書き禁止。`*_file` 参照のみ許可）
	- 条件付き必須: Kubernetes にデプロイする場合は `deploy/`（`base/` + `overlays/{dev,stg,prod}/`）
- フロントエンド（Flutter）必須: `pubspec.yaml`, `config/`, `lib/`
	- 必須: `config/{default,dev,stg,prod}.yaml`（機密値直書き禁止。`*_file` 参照のみ許可）
	- 条件付き必須: Kubernetes にデプロイする場合は `deploy/`（`base/` + `overlays/{dev,stg,prod}/`）

framework が提供する共通サービス

- framework の `auth-service` / `config-service` / `endpoint-service` は共通マイクロサービスであり、`feature/` 配下のサービスと同じ運用単位として扱う
- 共通サービスの置き場所: `framework/backend/{lang}/services/{service_name}/`
- `{service_name}` は kebab-case かつ `-service` で終える（例: `auth-service`）
- ディレクトリ/ファイル構成は固定
	- 必須: `README.md`, `config/`, `deploy/`, `src/`
	- 言語別必須: Rust は `Cargo.toml`、Go は `go.mod`
	- 必須: `config/{default,dev,stg,prod}.yaml`
	- 必須: `deploy/base/` と `deploy/overlays/{dev,stg,prod}/`
	- 必須: `src/` は Clean Architecture 構成（`application/`, `domain/`, `infrastructure/`, `presentation/`）
	- 条件付き必須: 公開 API を持つ場合
		- REST: `openapi/openapi.yaml`
		- gRPC: `proto/`（`*.proto`）

framework 共通 DB の責務（所有者を固定）

- framework 共通テーブルは所有サービスを固定し、所有サービス以外はスキーマ変更を行わない
	- `auth-service` 所有: `fw_m_user`, `fw_m_role`, `fw_m_permission`, `fw_m_user_role`, `fw_m_role_permission`
	- `config-service` 所有: `fw_m_setting`
	- `endpoint-service` 所有: `fw_m_endpoint`
- DDL の正本は `framework/database/table/*.sql` とする
- 適用用マイグレーションの正本は各サービスの `framework/backend/{lang}/services/{service_name}/migrations/` とする
- テーブル定義を変更する場合は必ず同一 PR で次を同時に更新する
	- `framework/database/table/<table>.sql`
	- 所有サービスの `migrations/`
	- 所有サービスの `README.md`（変更点・影響・ロールバック方針）
- `migrations/` 命名規則: `0001_*.sql`（4 桁連番）
- 互換性ルール: 本番適用後の破壊的変更（列削除・型変更等）は原則禁止。必要なら段階移行とし、ADR で例外承認

サービス `README.md` 規約（全サービス共通）

- 対象: `feature/backend/*/{feature_name}/` / `feature/frontend/*/{feature_name}/` / `framework/backend/*/services/{service_name}/` / BFF（存在する場合）
- `README.md` は必ず次の見出しをこの順で持つ
	- `# {service_name}`
	- `## 概要`
	- `## 責務`
	- `## 公開API`
	- `## 依存`
	- `## 設定`
	- `## DB`
	- `## 認証・認可`（必要な場合）
	- `## 監視`
	- `## 起動方法`
	- `## リリース`
- 公開 API が無いサービスは作らない（例外は ADR に記録）
	- 例外: フロントエンドは「公開 API」を持たない場合があるため、該当する場合は `## 公開API` に「なし」と明記する

命名規則（固定）

- `{feature_name}` は kebab-case
- `{service_name}` は `{feature_name}` と同一（DB 設定スコープ・監視名・権限スコープと一致）
- `{env}` は固定の 4 値: `default` / `dev` / `stg` / `prod`

Clean Architecture（依存方向）

※ 本節はバックエンド（Rust/Go）およびフロントエンド（React/Flutter）を対象とする（層の解釈は各プラットフォームで統一する）

- `presentation` → `application` → `domain`
- `infrastructure` は外部 I/O（DB/HTTP/Cache/Queue/OTel 等）の実装置き場
- `domain` は原則として外部フレームワーク/DB/HTTP に依存しない

フロントエンドでの解釈（固定）

- `presentation`: 画面/UI（pages, widgets, components 等）と UI からの入出力（フォーム/バリデーション/DTO）
- `application`: 状態管理・ユースケース・画面ロジック（ViewModel/Controller/Usecase 等）
- `domain`: 業務ドメインのモデル（Entity/ValueObject/DomainError）と port（Repository interface 等）
- `infrastructure`: 外部 I/O（HTTP API client, storage, cache 等）の実装（repository 実装）

層ごとの置くもの（固定）

- `src/domain/`: Entity / ValueObject / DomainError / Repository traits（ports）
- `src/application/`: Usecase と Application Service
- `src/infrastructure/`: Repository 実装、DB/Cache/Queue/Config/Logging/Telemetry
- `src/presentation/`: HTTP/gRPC/WebSocket のルーティング、DTO、バリデーション、エラー変換

フロントエンド（React）の置き場（固定）

- `src/presentation/`: pages/components、ルーティング、フォーム/バリデーション、UI 用 DTO
- `src/application/`: usecases、state（store）、ViewModel/Controller
- `src/domain/`: entities/value_objects、domain errors、repository interfaces
- `src/infrastructure/`: api client、repository implementations、storage（local persistence）

補足（React の配線の扱い）

- `src/presentation/` でルーティングを直接組むのではなく、framework が提供する AppShell（Header/Footer/Menu）と設定駆動ナビゲーションに寄せる
- 個別機能チームは「画面（Screen）実装」と「画面IDの登録」までを担当し、ルート/メニュー/遷移は `config/{env}.yaml` の変更を既定とする

フロントエンド（Flutter）の置き場（固定）

- `lib/src/presentation/`: pages/widgets、ルーティング、フォーム/バリデーション、UI 用 DTO
- `lib/src/application/`: usecases、state（provider/bloc等の選定は別途固定）、ViewModel/Controller
- `lib/src/domain/`: entities/value_objects、domain errors、repository interfaces
- `lib/src/infrastructure/`: api client、repository implementations、storage（local persistence）

型の流出ルール（固定）

- domain 型（Entity/VO）を presentation のレスポンス DTO として直接返さない
- infrastructure の DB 行モデル（Row/Record）を domain に漏らさない
- DTO ↔ Usecase 入出力 ↔ Domain の変換は presentation 側（I/O 境界）で行う

設定ルール（固定）

フロントエンド（React/Flutter）の設定（固定）

- フロントエンドも `config/{env}.yaml` を正本とする。環境変数による設定注入は禁止
	- 機密値そのものは YAML に置かず、参照（`*_file`）のみを保持する
- Kubernetes では次を標準とする
	- `config/{env}.yaml` を `/etc/k1s0/config/{env}.yaml` に mount
	- 機密は Kubernetes Secret 等から volume mount し、`/var/run/secrets/k1s0/` 配下のファイルとして配布
	- アプリは起動時/初期化時に設定ファイルと `*_file` を読み込む（読み込み方式の実装はテンプレで統一する）

- framework 設定: `config/{env}.yaml`
- feature 固有設定: DB（`fw_m_setting`）
- 起動時に `--env {env}` を明示し、`config/{env}.yaml` を選択する（暗黙選択しない）
- `fw_m_setting.setting_key` 命名: `category.name`（ドット区切り、小文字+数字+アンダースコア）
	- 例: `http.timeout_ms` / `db.pool_size` / `auth.jwt_ttl_sec` / `feature.flag_x`

フロントエンド（React）の UI / ナビゲーション設定（固定）

- 画面遷移/メニュー/表示制御は `config/{env}.yaml` の `ui.navigation`（例）で定義し、framework が解釈して反映する
	- routes（URL と画面IDの対応）
	- menu（メニュー表示・並び・グルーピング）
	- flows（ウィザード等の画面遷移グラフ。許可遷移と条件）
- 権限/feature flag による表示・遷移制御も設定で表現できる
	- 例: `requires.permissions` / `requires.flags` を用意し、満たさない場合は非表示/遷移不可を既定とする

`ui.navigation` の例（routes / menu / flows まで）

```yaml
ui:
	navigation:
		# このバージョンは、設定スキーマの互換性管理に使う（将来のupgradeで重要）
		version: 1

		# URL と「画面ID（screen_id）」の対応
		# screen_id はコード側で登録される（例: screen registry）。設定は配線のみを担う。
		routes:
			- path: /
				redirect_to: /home

			- path: /home
				screen_id: home
				title: Home
				requires:
					permissions: []
					flags: []

			- path: /users
				screen_id: users.list
				title: Users
				requires:
					permissions: ["user:read"]

			- path: /users/:userId
				screen_id: users.detail
				title: User Detail
				requires:
					permissions: ["user:read"]

			- path: /settings
				screen_id: settings
				title: Settings
				requires:
					permissions: ["settings:read"]
					flags: ["settings_enabled"]

		# メニュー（Header/SideMenu等）に出す情報
		# framework はこの定義からメニューUIを生成し、認可/flag条件により自動で表示制御する
		menu:
			- id: primary
				label: Main
				items:
					- label: Home
						to: /home
						icon: home
					- label: Users
						to: /users
						icon: users
						requires:
							permissions: ["user:read"]
					- label: Settings
						to: /settings
						icon: settings
						requires:
							permissions: ["settings:read"]
							flags: ["settings_enabled"]

		# ウィザード等の「許可される遷移」と「条件」を、設定として表現する
		# 目的:
		# - “画面の配線（どこへ進めるか）” をコードから外し、ノーコードで変更可能にする
		# - 不正遷移（手順スキップ/逆流）を framework 側のガードで防ぐ
		flows:
			- id: user_onboarding
				title: User Onboarding

				# flow の入口（開始画面）
				start:
					screen_id: users.onboarding.start

				# 任意: flow 実行に必要な条件（満たさない場合は開始不可）
				requires:
					permissions: ["user:write"]

				# flow を構成するノード（各ノードは screen_id を持つ）
				# node_id は遷移定義の参照名であり、URLではない
				nodes:
					- node_id: start
						screen_id: users.onboarding.start

					- node_id: profile
						screen_id: users.onboarding.profile

					- node_id: confirm
						screen_id: users.onboarding.confirm

					- node_id: done
						screen_id: users.onboarding.done

				# 許可遷移（ガード付き）
				# event は UI 操作（Next/Back/Cancel 等）を抽象化したもの
				transitions:
					- from: start
						event: next
						to: profile

					- from: profile
						event: back
						to: start

					- from: profile
						event: next
						to: confirm
						when:
							# 例: 入力が揃っていること、フラグが有効であること等
							required_form_keys: ["profile"]
							flags: ["onboarding_enabled"]

					- from: confirm
						event: back
						to: profile

					- from: confirm
						event: submit
						to: done

				# 例外遷移（強制遷移）
				# - 条件を満たさない/未登録screen等の場合のフォールバック
				on_error:
					redirect_to: /home
```

補足（解釈ルールの既定）

- `requires.permissions` / `requires.flags` を満たさない場合
	- menu: 自動的に非表示（既定）
	- route: 遷移を拒否し、`redirect_to`（なければ `/`）へ戻す（既定）
	- flow: `start` および `transitions` を拒否し、`on_error.redirect_to` へ遷移（既定）
- `screen_id` が未登録の場合は起動時に検知して失敗する（設定のミスを早期に落とす）

API ドキュメント/デプロイ配置（固定）

- API ドキュメント: バックエンドは `openapi/`（REST）および `proto/`（gRPC）
- デプロイ: Kubernetes にデプロイするサービスは `deploy/`（Kustomize: `base/` + `overlays/{env}/`）

### SHOULD（推奨）

- `docs/` には確定した規約/運用/ADR を置き、検討中の草案は `work/` に置く
- `openapi/` は手書きと生成物を分ける（手書き `openapi/openapi.yaml`、生成 `openapi/gen/`）
- `migrations/` は連番 SQL で管理し、適用ツールは各言語標準に合わせる

### 禁止事項

- `feature/` 配下に `common/` を作るなど、共通化を各チーム裁量にしない
- `domain` から `infrastructure` を直接 import しない
- 設定の読み替えや上書きに環境変数を利用する実装を入れない（必要なら ADR で例外化）

---


