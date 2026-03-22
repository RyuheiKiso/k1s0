# 開発環境トラブルシューティング

k1s0 開発環境でよく発生するトラブルとその対処法をまとめたガイド。

---

## 1. CRLF 問題（最も多い罠）

### 症状

- シェルスクリプトが実行できない
- `\r: command not found` エラーが発生する
- `bash: scripts/setup-wsl.sh: cannot execute: required file not found` エラーが発生する

### 原因

Git for Windows はデフォルトで `core.autocrlf=true` が設定されており、クローン時に LF が CRLF に変換される。本リポジトリの `.gitattributes` は `eol=lf` を強制しているため、CRLF が混入したスクリプトは Linux / WSL2 上で実行できない。

### 対処

**クローン前（推奨）**:

```powershell
# PowerShell または Git Bash で実行
git config --global core.autocrlf input
git config --global core.longpaths true
```

**クローン後に発生した場合**（インデックスをリセットして .gitattributes を再適用する）:

```bash
# 未コミットの変更がある場合は先に退避する
git stash

# インデックスをクリアして .gitattributes を再適用する
git rm -rf --cached .
git reset --hard HEAD

# 退避した変更を戻す
git stash pop
```

**現在の設定を確認する**:

```bash
git config core.autocrlf
# "input" であれば正常（LF をそのまま保持する設定）
```

---

## 2. Docker リソース不足

### 症状

- コンテナが OOM Killed（メモリ不足で強制終了）になる
- `docker compose up` 後にコンテナが繰り返し再起動する（`Restarting` 状態）
- コンテナの起動が異常に遅い

### 原因

インフラサービス（PostgreSQL, MySQL, Redis x2, Kafka, Schema Registry, Keycloak, Vault, Kafka UI）を全て起動すると合計で 5 GB 以上のメモリを消費する。Docker Desktop / WSL2 のデフォルトメモリ割り当てが不足している。

### 対処

**Docker Desktop の場合**（Settings → Resources → Memory を 8 GB 以上に設定）:

```
Docker Desktop → Settings → Resources → Memory: 8 GB 以上
```

**WSL2 の場合**（`%USERPROFILE%\.wslconfig` を作成・編集する）:

```ini
# Windows ホスト側: %USERPROFILE%\.wslconfig
[wsl2]
memory=8GB
processors=4
swap=4GB
```

変更後は `wsl --shutdown` で WSL2 を再起動して設定を反映させる。

**段階的起動**（必要なサービスのみを起動してリソース消費を抑える）:

```bash
# インフラのみ起動（最小構成）
docker compose --profile infra up -d

# 可観測性スタックも必要な場合に追加する
docker compose --profile infra --profile observability up -d
```

---

## 3. WSL2 のファイルシステム（I/O 遅延）

### 症状

- `cargo build` や `npm install` が異常に遅い（数分以上かかる）
- ファイル監視（ホットリロード）が効かない
- `inotify` 関連のエラーが発生する

### 原因

リポジトリを `/mnt/c/` 配下（Windows ファイルシステム）にクローンしている場合、WSL2 は Plan 9 ファイルシステムプロトコル（9P）を経由してファイルにアクセスするため I/O が大幅に低下する。また `inotify` によるファイル変更監視も動作しない。

### 対処

WSL2 のホームディレクトリ（`~/`）にリポジトリをクローンし直す:

```bash
# リポジトリの場所を確認する
pwd
# NG: /mnt/c/Users/... の場合は以下の手順で移動する

# WSL2 のホームディレクトリにクローンし直す
cd ~
mkdir -p work && cd work
git clone <repository-url> k1s0
cd k1s0
```

> **注意**: Windows 側の VS Code からも WSL2 ファイルシステムのリポジトリを開ける。`Remote - WSL` 拡張機能を使用して接続すること。

---

## 4. ポート競合

### 症状

- `docker compose up` で以下のようなエラーが発生する:
  ```
  Error response from daemon: Ports are not available: exposing port TCP 0.0.0.0:5432 -> 0.0.0.0:0: listen tcp 0.0.0.0:5432: bind: address already in use
  ```

### 原因

ホスト側で既に同じポートを使用しているプロセスがある。Windows ではポート予約（Hyper-V のポート予約など）によってポートが使用できないケースもある。

### 使用ポート一覧

| サービス | デフォルトポート | 環境変数 |
|---------|----------------|---------|
| PostgreSQL | 5432 | `PG_HOST_PORT` |
| MySQL | 3306 | `MYSQL_HOST_PORT` |
| Redis | 6379 | `REDIS_HOST_PORT` |
| Redis（セッション） | 6380 | `REDIS_SESSION_HOST_PORT` |
| Kafka | 9092 | `KAFKA_HOST_PORT` |
| Kafka UI | 8090 | `KAFKA_UI_HOST_PORT` |
| Schema Registry | 8081 | `SCHEMA_REGISTRY_HOST_PORT` |
| Keycloak | 8180 | `KEYCLOAK_HOST_PORT` |
| Keycloak Management | 9000 | `KEYCLOAK_MGMT_HOST_PORT` |
| Vault | 8200 | `VAULT_HOST_PORT` |
| Jaeger UI | 16686 | `JAEGER_UI_HOST_PORT` |
| Prometheus | 9090 | `PROMETHEUS_HOST_PORT` |
| Grafana | 3200 | `GRAFANA_HOST_PORT` |

### 対処

**競合ポートを確認する（Linux / WSL2）**:

```bash
# 対象ポートを使用しているプロセスを確認する（例: 5432）
netstat -tulnp | grep 5432

# または ss コマンドを使用する
ss -tulnp | grep 5432
```

**競合ポートを確認する（Windows PowerShell）**:

```powershell
netstat -ano | findstr :5432
```

**`.env` でポートを変更する**（`.env.example` をコピーして編集する）:

```bash
cp .env.example .env
# .env を編集して競合するポートを変更する（例: PostgreSQL を 5433 に変更）
# PG_HOST_PORT=5433
```

---

## 5. Docker グループ権限

### 症状

WSL2 で Docker コマンドを実行すると以下のエラーが発生する:

```
permission denied while trying to connect to the Docker daemon socket at unix:///var/run/docker.sock
```

または:

```
docker: Got permission denied while trying to connect to the Docker daemon socket
```

### 原因

`usermod -aG docker $USER` でユーザーを `docker` グループに追加した後、WSL2 を再起動せずにいるためグループ変更が反映されていない。

### 対処

**WSL2 を完全に再起動する**（Windows PowerShell で実行）:

```powershell
# WSL2 を完全シャットダウンする
wsl --shutdown

# WSL2 を再起動する（Ubuntu を開き直す）
```

再起動後、グループが反映されているか確認する:

```bash
# docker グループに所属しているか確認する
groups
# 出力に "docker" が含まれていれば正常

# 動作確認
docker run --rm hello-world
```

グループに追加されていない場合は改めて追加する:

```bash
sudo usermod -aG docker $USER
# その後 wsl --shutdown → 再起動
```

---

## 6. Windows ネイティブでのビルドエラー

### 症状

Rust のサーバービルド（`cargo build`）で以下のようなエラーが発生する:

```
error: failed to run custom build command for `rdkafka-sys v...`
  cmake not found
  ...
```

または:

```
error[E0425]: cannot find function `...` in module `zen_engine`
```

### 原因

以下のクレートは Windows ネイティブビルドに非対応:

- **`rdkafka`**: librdkafka の C ライブラリをビルドするため、Linux 向けの依存関係（`libsasl2`, `libssl` 等）が必要
- **`zen-engine`**: master-maintenance サーバーで使用するビジネスルールエンジン。Windows 向けビルドに制約がある

### 対処

**devcontainer を使用する（推奨）**:

```
VS Code → コマンドパレット → "Dev Containers: Reopen in Container"
```

**WSL2 ネイティブビルドを使用する**:

```bash
# WSL2 内でビルドする
cd ~/work/k1s0
cargo build
```

**Windows ネイティブで実施可能な作業の範囲**（参考）:

| 作業 | Windows ネイティブ |
|------|------------------|
| CLI ビルド・テスト | 可（`just cli-build / cli-test`） |
| TypeScript リント・テスト | 可（`pnpm run lint / test`） |
| Dart リント・テスト | 可（`dart analyze / dart test`） |
| Rust サーバービルド | **不可**（rdkafka/zen-engine の制約） |
| `just local-up` | **不可**（WSL2/devcontainer が必要） |

---

## 7. Docker デーモンが起動しない（WSL2）

### 症状

```
Cannot connect to the Docker daemon at unix:///var/run/docker.sock. Is the docker daemon running?
```

### 原因

WSL2 で systemd が有効でない場合、Docker デーモンが自動起動しない。

### 対処

**systemd を有効にする**（`/etc/wsl.conf` を作成または編集する）:

```ini
# WSL2 内: /etc/wsl.conf
[boot]
systemd=true
```

設定後は `wsl --shutdown` で再起動してから Docker を起動する:

```bash
sudo systemctl start docker
sudo systemctl enable docker

# 動作確認
docker run --rm hello-world
```

**systemd が使えない場合（手動起動）**:

```bash
# Docker デーモンをバックグラウンドで起動する
sudo dockerd &

# ログを確認しながら起動する
sudo dockerd --debug 2>&1 | head -50
```

---

## 8. VS Code が WSL2 の Docker を認識しない

### 症状

Dev Containers 拡張が Docker を検出できず、「Reopen in Container」が表示されない。

### 対処

VS Code の `settings.json` に Docker ソケットのパスを明示する:

```json
{
  "dev.containers.dockerSocketPath": "/var/run/docker.sock"
}
```

---

## 9. Redis dump.rdb パーミッションエラー（M-06 監査対応）

### 症状

`docker compose up` 時に Redis が起動失敗し、以下のエラーが発生する:

```
redis | Fatal error, can't open config file '/data/dump.rdb': Permission denied
```

または:

```
redis | FATAL CONFIG FILE ERROR (Redis 7.x.x)
redis | Can't open the log file: /data/dump.rdb: Permission denied
```

### 原因

`docker compose down`（`-v` なし）後に再起動すると、前回のコンテナが残した `dump.rdb` ファイルのオーナーが現ユーザーと異なる場合がある。具体的には以下の状況で発生する:

- コンテナ内の Redis プロセスが `redis` ユーザー（UID 999 等）で `dump.rdb` を作成した
- `docker compose down` 時に `-v` フラグを付けなかったため、Docker volume（またはバインドマウントディレクトリ）が残留した
- 再起動時にホスト側ユーザーと `dump.rdb` のオーナーが一致せず、Permission denied が発生する

### 対処方法

**方法 1: ボリュームを含めて完全削除してから再起動する（推奨）**

Redis の永続化データが不要な場合（開発環境では通常不要）は、ボリュームごと削除して起動し直す。

```bash
# ボリュームを含めて全コンテナを削除してから再起動する
docker compose down -v
docker compose up -d
```

**方法 2: Redis ボリュームのみを削除して再起動する**

他のサービスのボリュームを維持したまま Redis のみリセットする場合:

```bash
# Redis コンテナを停止する
docker compose stop redis

# Redis ボリュームのみを削除する
docker volume rm $(docker volume ls -q | grep redis)

# Redis コンテナを再起動する
docker compose up -d redis
```

**方法 3: パーミッションを手動で修正する（データを保持したい場合）**

Redis のデータを残したまま修正する場合（本番データのバックアップ等）:

```bash
# dump.rdb のオーナーを確認する
ls -la $(docker volume inspect --format '{{ .Mountpoint }}' $(docker volume ls -q | grep redis))

# オーナーを現ユーザーに変更する（UID 999 は Redis コンテナ内のユーザー）
sudo chown -R 999:999 $(docker volume inspect --format '{{ .Mountpoint }}' $(docker volume ls -q | grep redis))

# Redis を再起動する
docker compose up -d redis
```

### 再発防止

開発環境ではデータの永続化が不要な場合が多いため、作業終了時は常に `-v` フラグ付きで停止することを推奨する:

```bash
# 作業終了時の停止コマンド（ボリューム含む）
docker compose down -v
```

---

## 関連ドキュメント

- [Windows クイックスタート](./windows-quickstart.md) — 3 つのセットアップ方法と手順
- [WSL2 開発環境セットアップ](./WSL2開発環境セットアップ.md) — WSL2 詳細セットアップ手順
- [docker-compose 設計](../docker/docker-compose設計.md) — サービス構成・プロファイル
- [ポート割り当て](../docker/ポート割り当て.md) — 全サービスのポート一覧
