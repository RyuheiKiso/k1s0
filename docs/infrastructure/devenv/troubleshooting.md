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

## 5. Windows Hyper-V によるポート排除（gRPC ポート競合）

### 症状

- `docker compose up` で gRPC サービス（event-monitor/master-maintenance/navigation/policy/rule-engine/session）のポートが `bind: address already in use` になる
- Windows の Hyper-V が特定ポート帯を動的に予約しているため、50174-50273 や 50279-50378 のポートが使用できない

### 原因

Windows 上で Hyper-V または WSL2 が動作していると、Hyper-V の動的ポート予約メカニズムがランダムなポート帯を排除リストに登録する。この排除リストに k1s0 の gRPC ポートが含まれると `bind` に失敗する。

CRIT-002 監査対応: 元々 50300-50305 を使用していたが、Hyper-V の排除範囲 50279-50378 と重複するため、50400-50405 に移動した。

### 確認方法

```powershell
# Hyper-V の動的ポート排除範囲を確認する（PowerShell で実行）
netsh int ipv4 show excludedportrange protocol=tcp
```

出力例（排除範囲が 50279-50378 を含む場合）:
```
プロトコル tcp の除外ポート範囲

開始ポート    終了ポート
----------    ----------
...
50174         50273
50279         50378        ← このため 50300-50305 は使用不可
...
```

### 対処

k1s0 はすでに CRIT-002 対応として gRPC ポートを 50400-50405 に移動済み。
排除範囲が 50400 以上まで拡張した場合は `.env.dev` の以下の変数を変更すること:

```bash
EVENT_MONITOR_GRPC_HOST_PORT=50400    # 変更が必要な場合は別ポートへ
MASTER_MAINTENANCE_GRPC_HOST_PORT=50401
NAVIGATION_GRPC_HOST_PORT=50402
POLICY_GRPC_HOST_PORT=50403
RULE_ENGINE_GRPC_HOST_PORT=50404
SESSION_GRPC_HOST_PORT=50405
```

### CRIT-003: K8s API サーバーポート競合（Windows Hyper-V）

Hyper-V の排除範囲が kubectl 通信ポート（6443）や K8s API サーバーポートと重複することがある。

```powershell
# K8s 関連ポートが排除されているか確認する
netsh int ipv4 show excludedportrange protocol=tcp | findstr /r "6443\|6440\|6450"
```

排除されている場合は以下を試みる:

```powershell
# Hyper-V を一時停止してポート排除をリセットする（管理者権限で実行）
net stop winnat
net start winnat
```

---

## 6. Docker グループ権限

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

## 10. Docker Desktop でのイメージビルド並列 OOM（M-03/HIGH-2 監査対応）

### 症状

`docker compose build` または `docker buildx bake` を実行すると以下のいずれかが発生する:

- ビルド途中で OOM Killer により Docker Desktop が強制再起動する
- `docker buildx bake` が `exit code 137` で失敗する
- `docker compose build --parallel` でメモリ使用量が急増してホスト全体が不安定になる
- OOM クラッシュ後にサービスを再起動してもホストポートフォワーディングが機能しない（HIGH-7）

### 原因

k1s0 は 30 以上のサービスを含むモノリポであり、全サービスを並列ビルドすると Docker BuildKit が同時に多数の Rust / Go コンパイラプロセスを起動する。Rust のコンパイルは特にメモリを消費するため、並列度が高いと Docker Desktop に割り当てたメモリを超過する。

> **重要**: `.env.dev` の `COMPOSE_PARALLEL_LIMIT=4` は `docker compose up` の依存解決並列数にのみ有効であり、`docker compose build` の並列ビルド数には効果がない（HIGH-2 監査対応）。

### 対処

**方法 1: `just docker-build-safe`（推奨）**

```bash
# 並列数 2 に制限した安全なビルド（OOM 防止）
just docker-build-safe
```

このコマンドは `docker compose build --parallel 2` を内部的に実行する。通常の `just docker-build` でOOMが発生する場合はこちらを使用する。

**方法 2: `docker compose build --parallel` で直接制限する**

```bash
# 並列ビルド数を 2 に制限（--parallel フラグは build 専用）
docker compose --env-file .env.dev -f docker-compose.yaml -f docker-compose.dev.yaml \
  build --parallel 2
```

**方法 3: WSL2 の場合は `.wslconfig` でメモリ上限を設定する**

```ini
# C:\Users\<username>\.wslconfig
[wsl2]
memory=8GB   # 環境に応じて調整（推奨: 実装メモリの50%以内）
swap=2GB
```

設定後は WSL2 を再起動: `wsl --shutdown`（Docker Desktop も再起動が必要）

**OOM クラッシュ後のポートフォワーディング復旧（HIGH-7）**

OOM で Docker Desktop がクラッシュ・自動復旧した後、ホストポートが機能しない場合:

```
Docker Desktop タスクバーアイコン → Restart
または
wsl --shutdown  → Docker Desktop を手動で起動
```

---

## 11. Windows Hyper-V 動的ポート除外による gRPC 起動失敗（HIGH-1 / CRIT-002 監査対応）

> **最新情報**: 「5. Windows Hyper-V によるポート排除（gRPC ポート競合）」も参照すること。
> CRIT-002 監査対応で gRPC ポートは 50300 帯から **50400-50405** 帯に移動済み（2026-04-06）。

### 症状

以下のエラーが発生し、gRPC ポートを持つサービスが起動しない:

```
Error response from daemon: driver failed programming external connectivity
on endpoint: Bind for 0.0.0.0:50060 failed: port is already allocated
```

または:

```
Error starting userland proxy: listen tcp4 0.0.0.0:50060: bind:
An attempt was made to access a socket in a way forbidden by its access permissions.
```

### 原因

Windows の Hyper-V は起動時に TCP ポートを動的に予約・除外する。確認された排除範囲: 50174-50273 / 50279-50378。

除外範囲を確認するには:

```powershell
netsh int ipv4 show excludedportrange protocol=tcp
```

### 対処

**現行バージョン（ADR-0040 + CRIT-002 対応済み）**: デフォルト gRPC ポートは 50400-50405 に変更済みのため、通常は発生しない。

古い `.env` ファイルや環境変数オーバーライドで旧ポート（50200-50205 または 50300-50305）を指定している場合は、`.env.dev` を最新版に更新すること:

```bash
# .env.dev の現行設定（CRIT-002 対応後）
EVENT_MONITOR_GRPC_HOST_PORT=50400
MASTER_MAINTENANCE_GRPC_HOST_PORT=50401
NAVIGATION_GRPC_HOST_PORT=50402
POLICY_GRPC_HOST_PORT=50403
RULE_ENGINE_GRPC_HOST_PORT=50404
SESSION_GRPC_HOST_PORT=50405
```

---

## 12. Docker Desktop on Windows: ヘルスチェックタイムアウト問題

### 症状

`just local-up-dev` 実行後、以下のようなエラーが発生してサービスが起動しない:

```
Container k1s0-jaeger-1 Error dependency jaeger failed to start
Container k1s0-kafka-1 Error dependency kafka failed to start
dependency failed to start: container k1s0-jaeger-1 is unhealthy
```

`docker inspect` でコンテナのヘルス状態を確認すると `unhealthy` になっている:

```bash
docker inspect k1s0-jaeger-1 --format '{{json .State.Health}}' | jq
```

### 原因

Docker Desktop on Windows では、`CMD-SHELL` 形式のヘルスチェックが特定の条件下でタイムアウトになる問題がある。
PostgreSQL・Redis・Vault・Kafka・Jaeger が `unhealthy` 状態になり、これらに `service_healthy` 条件で依存するサービスが起動を待ち続ける。

### 対策 1: ヘルスチェックタイムアウトを延長する（推奨）

`docker-compose.override.yaml` を作成してタイムアウト値を延長する:

```yaml
# docker-compose.override.yaml（.gitignore 対象、ローカル専用）
services:
  postgres:
    healthcheck:
      timeout: 10s
      retries: 10
      start_period: 60s
  kafka:
    healthcheck:
      timeout: 30s
      retries: 10
      start_period: 120s
  jaeger:
    healthcheck:
      timeout: 20s
      retries: 10
      start_period: 60s
  vault:
    healthcheck:
      timeout: 10s
      retries: 10
      start_period: 60s
```

### 対策 2: WSL2 バックエンドを使用する（推奨）

Docker Desktop の設定で「Use the WSL 2 based engine」を有効にすると、Linux ネイティブのヘルスチェックが使われてタイムアウト問題が解消されることが多い。

### 対策 3: 手動で健全性を確認してから起動する

インフラサービスを先に起動し、状態を手動確認してからアプリを起動する:

```bash
# Step 1: インフラ起動
docker compose --env-file .env.dev -f docker-compose.yaml -f docker-compose.dev.yaml \
  --profile infra up -d

# Step 2: PostgreSQL が接続できるまで待機
until docker exec k1s0-postgres-1 pg_isready -U k1s0 2>/dev/null; do
  echo "Waiting for postgres..."; sleep 3
done

# Step 3: その後アプリ起動
docker compose --env-file .env.dev -f docker-compose.yaml -f docker-compose.dev.yaml \
  --profile system up -d
```

### 参考

- [ADR-0040](../../../architecture/adr/0040-grpc-port-range-hyper-v-avoidance.md): Hyper-V ポート予約回避
- Docker Desktop 公式: WSL 2 バックエンドの利用推奨

---

## 13. K8s desktop-worker ノードが NotReady 状態（LOW-012 監査対応）

### 症状

```bash
kubectl get nodes
# NAME              STATUS     ROLES           AGE
# docker-desktop    Ready      control-plane   ...
# desktop-worker    NotReady   <none>          ...
```

または `kubectl describe node desktop-worker` で以下のメッセージが確認される:

```
Kubelet stopped posting node status.
```

### 原因

Docker Desktop の Kubernetes 機能を使用している場合、`desktop-worker` はワーカーノードとして自動作成される仮想ノードである。以下の状況で `NotReady` になる:

- Docker Desktop の再起動後に kubelet が自動復旧しなかった
- Docker Desktop のリソース不足（メモリ・CPU）でノードエージェントが停止した
- Kubernetes クラスターの内部状態の破損

### 対処

**方法 1: Docker Desktop ごと再起動する（最も簡単）**

```
Docker Desktop タスクバーアイコン → Restart
```

再起動後、`kubectl get nodes` で `desktop-worker` が `Ready` になることを確認する。

**方法 2: Kubernetes クラスターをリセットする**

```
Docker Desktop → Settings → Kubernetes → Reset Kubernetes Cluster
```

> **注意**: クラスターのリセットは全デプロイ・ConfigMap・Secret を削除する。ローカルで適用した K8s リソースは再適用が必要。

**方法 3: ノードを強制削除して再参加させる**

```bash
# ノードを削除する（kubelet 再起動後に自動再参加する）
kubectl delete node desktop-worker

# Docker Desktop の Kubernetes を再起動する
# （Restart ではなく Reset Kubernetes Cluster）
```

### CI/CD への影響

この問題はローカル Docker Desktop 固有であり、CI/CD 環境には影響しない。

CI/CD の K8s 統合テストには独立した環境（GitHub Actions の K8s サービスコンテナ、または専用クラスター）を使用すること。ローカルの `desktop-worker` 状態に依存したテストは書かないこと。

### 参考

- [kubernetes 設計](../kubernetes/kubernetes設計.md) — K8s 全体設計
- [デプロイ手順書](../kubernetes/デプロイ手順書.md) — K8s デプロイ手順

---

---

## Docker Compose 環境変数変更後のコンテナ再起動（HIGH-001 対応）

### 症状

- `docker-compose.dev.yaml` や `.env.dev` に環境変数を追加・変更したが、稼働中サービスに反映されない
- サービスの readyz が `degraded` を返す（例: rule-engine の Kafka 未接続）

### 原因

Docker Compose はコンテナ起動時に環境変数を読み込むため、設定変更後も **既存コンテナには反映されない**。特に `docker compose up -d` は変更を検知しても依存するコンテナが既に起動中の場合は再起動しない。

### 対処

**特定サービスのみ強制再起動する**:

```bash
# rule-engine を再起動する例（Kafka 接続設定変更後など）
docker compose --env-file .env.dev \
  -f docker-compose.yaml -f docker-compose.dev.yaml \
  up -d --force-recreate rule-engine-rust

# featureflag を再起動する例
docker compose --env-file .env.dev \
  -f docker-compose.yaml -f docker-compose.dev.yaml \
  up -d --force-recreate featureflag-rust
```

**全サービスをリビルドして再起動する**:

```bash
docker compose --env-file .env.dev \
  -f docker-compose.yaml -f docker-compose.dev.yaml \
  build && \
docker compose --env-file .env.dev \
  -f docker-compose.yaml -f docker-compose.dev.yaml \
  up -d --force-recreate
```

### 確認

```bash
# readyz で全サービスの状態確認
for port in 8083 8084 8085 8086 8087 8088 8089 8091 8092 8093 8094 \
            8095 8096 8097 8098 8099 8101 8102 8103 8104 8105 8106 \
            8107 8108 8082 8122 8211 8311 8321 8331; do
  echo -n "port $port: "
  curl -sf http://127.0.0.1:${port}/readyz | python3 -c "import sys,json; d=json.load(sys.stdin); print(d.get('status','?'))" 2>/dev/null || echo "unreachable"
done
```

---

## stale Docker イメージ問題（CRIT-002 / CRIT-003 / HIGH-002 / HIGH-003 / HIGH-006 対応）

### 症状

- ソースコードを変更したのに稼働中コンテナに反映されない
- readyz が 404 を返す（例: featureflag の `/readyz` エンドポイント）
- healthz が旧バージョンのレスポンスを返す
- `config-rust` が `Error: auth configuration is required` で起動ループする（`Restarting` 状態）
- `featureflag-rust` が `503 Service Unavailable` を返し続け `unhealthy` になる

### 原因

稼働中の Docker コンテナが古いイメージを使用している。`docker ps` で `CREATED` 列を確認し、数時間〜数日前になっていれば stale image の可能性が高い。

`docker compose up`（just を使わず直接）を実行した場合、`--build` オプションが省略されるため新しいコードが反映されない。

**具体的な症状別原因**:

| 症状 | 根本原因 |
|------|---------|
| config-rust が起動ループ（CRIT-002） | `dev-auth-bypass` feature フラグを含まないイメージが起動している |
| featureflag-rust が unhealthy（CRIT-003） | migration 006（UUID→TEXT）対応前の古いイメージが動作している |
| マイグレーション未実行でサービス unhealthy（HIGH-003） | `just local-up` を使わず `docker compose up` を直接実行した |

### 対処

**最も確実な方法（推奨）**: `just local-up` を使用する

```bash
# just local-up は --build と migrate-all が自動適用される
just local-up
```

`just local-up` は `just local-up-dev` のエイリアスであり、以下を自動実行する:
1. **Phase 1**: インフラサービスを `--build` 付きで起動
2. **Phase 1.5**: `just migrate-all`（DB マイグレーション）を自動実行
3. **Phase 2**: 全サービスを `--build` 付きで起動

**特定サービスのみ再ビルドする場合**:

```bash
# config-rust と featureflag-rust のみ再ビルド・再起動する
docker compose --env-file .env.dev \
  -f docker-compose.yaml -f docker-compose.dev.yaml \
  build config-rust featureflag-rust && \
docker compose --env-file .env.dev \
  -f docker-compose.yaml -f docker-compose.dev.yaml \
  up -d --force-recreate config-rust featureflag-rust
```

**注意**: `docker compose up` を直接実行することは推奨しない。`--build` と `--env-file .env.dev` と `-f docker-compose.dev.yaml` の全てを手動で指定しなければならず、漏れると上記の CRIT-002/003 症状が発生する。

### stale image の検出

```bash
# 長時間起動しているコンテナを確認する（24 時間以上が目安）
docker ps --format "table {{.Names}}\t{{.Status}}\t{{.CreatedAt}}" | sort
```

---

## ghcr.io レジストリアクセス拒否（HIGH-002 対応）

### 症状

```
Error response from daemon: Head "https://ghcr.io/v2/prometheus/jmx-exporter/manifests/1.0.1":
denied: denied
```

または

```
Error response from daemon: pull access denied for ghcr.io/prometheus/jmx-exporter,
repository does not exist or may require 'docker login'
```

### 原因

GitHub Container Registry（ghcr.io）への認証が未設定。ghcr.io はパブリックイメージでも Docker ログインが必要な場合がある。

### 対処

```bash
# GitHub Personal Access Token（read:packages スコープ）でログインする
# GITHUB_TOKEN 環境変数が設定済みの場合
echo $GITHUB_TOKEN | docker login ghcr.io -u <your-github-username> --password-stdin

# 対話的にログインする場合（パスワード欄に Personal Access Token を入力）
docker login ghcr.io
```

**Personal Access Token の作成方法**:
1. GitHub → Settings → Developer settings → Personal access tokens → Tokens (classic)
2. `read:packages` スコープを選択して生成
3. 上記コマンドのパスワード欄に貼り付け

ログイン後、`docker compose pull` または `just local-up` を再実行する。

---

## Windows Hyper-V ポートフォワーディング問題（HIGH-005 / LOW-003 対応）

### 症状

- `curl http://127.0.0.1:8084/healthz` がホスト（Windows）から応答しない（`Empty reply from server` / exit 52）
- Docker 内部ネットワーク（`curl http://config-rust:8080/healthz`）では正常応答する
- Docker の healthcheck（コンテナ内から実行）は成功し、コンテナは `healthy` 表示になる

### 原因

Windows の Hyper-V / Docker Desktop における動的ポート除外範囲（Ephemeral Port Range）に特定ポートが含まれており、ポートフォワーディングが機能しない。

**除外範囲の確認コマンド（PowerShell）**:

```powershell
netsh interface ipv4 show excludedportrange protocol=tcp
```

出力に当該ポート（例: 8084）が含まれている場合、そのポートは使用できない。

### 対処

**環境変数でポートを変更する**（推奨）:

`.env.dev` または環境変数に以下を追加し、コンテナを再起動する:

```bash
# 例: config サービスを 8184 に変更する
CONFIG_REST_HOST_PORT=8184
```

**Docker 内部ネットワーク経由でアクセスする**:

```bash
# Docker ネットワーク経由でコンテナに curl する
docker run --rm --network k1s0-network curlimages/curl \
  curl http://config-rust:8080/healthz
```

**別ポートでの代替確認**:

```bash
# Wireshark / Process Monitor で除外されていないポートを特定して使用する
# 一般的に 8200〜8299 や 9000〜9099 帯は除外されないことが多い
```

---

## Git 設定推奨値（LOW-001 対応）

`k1s0 doctor` コマンドで以下の警告が表示される場合の対処:

```
[WARN] Git: core.autocrlf が未設定です
[WARN] Git: core.longpaths が未設定または false です
```

### 対処（PowerShell または Git Bash）

```bash
# Windows 環境での推奨設定
git config --global core.autocrlf input   # LF を維持（CRLF 変換防止）
git config --global core.longpaths true   # 260 文字超のパスを許可
```

---

## k1s0 CLI が PATH 未登録（LOW-002 対応）

`k1s0 doctor` コマンドで以下の警告が表示される場合の対処:

```
[WARN] k1s0 CLI: k1s0 が見つかりません
```

### 対処

```bash
# CLI を cargo でインストールする
cargo install --path CLI/crates/k1s0-cli

# インストール確認
k1s0 --version
```

---

## 関連ドキュメント

- [Windows クイックスタート](./windows-quickstart.md) — 3 つのセットアップ方法と手順
- [WSL2 開発環境セットアップ](./WSL2開発環境セットアップ.md) — WSL2 詳細セットアップ手順
- [docker-compose 設計](../docker/docker-compose設計.md) — サービス構成・プロファイル
- [ポート割り当て](../docker/ポート割り当て.md) — 全サービスのポート一覧
