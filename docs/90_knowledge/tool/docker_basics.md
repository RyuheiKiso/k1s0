# Docker: 概要

- 対象読者: Linux の基本操作ができる開発者
- 学習目標: Docker の仕組みを理解し、コンテナの作成・実行・管理ができるようになる
- 所要時間: 約 40 分
- 対象バージョン: Docker Engine 27.x / Docker Desktop 4.x
- 最終更新日: 2026-04-12

## 1. このドキュメントで学べること

- Docker が解決する課題とコンテナ技術の意義を説明できる
- イメージ・コンテナ・レジストリの関係を理解できる
- Dockerfile を記述してイメージをビルドできる
- docker コマンドでコンテナの起動・停止・削除ができる

## 2. 前提知識

- Linux コマンドラインの基本操作（cd, ls, cat 等）
- アプリケーションの実行環境（OS、ライブラリ、依存関係）の概念
- テキストエディタの基本操作

## 3. 概要

Docker は、アプリケーションとその実行環境をまとめて「コンテナ」というパッケージに閉じ込め、どの環境でも同じように動作させるためのプラットフォームである。Docker 社（旧 dotCloud 社）が 2013 年にオープンソースとして公開した。

従来の開発では「開発マシンでは動くが本番サーバーでは動かない」という問題が頻発していた。OS の違い、ライブラリのバージョン差異、設定の食い違いが原因である。Docker はアプリケーションに必要なすべてをイメージとして固め、そのイメージからコンテナを起動する。コンテナはホスト OS のカーネルを共有しつつ、プロセスやファイルシステムを隔離するため、仮想マシンより軽量で高速に起動する。

## 4. 用語の整理

| 用語 | 説明 |
|------|------|
| イメージ（Image） | コンテナの実行に必要なファイルシステムと設定をまとめた読み取り専用のテンプレート |
| コンテナ（Container） | イメージから起動された実行中のインスタンス。プロセスが隔離された環境で動作する |
| Dockerfile | イメージのビルド手順を記述したテキストファイル |
| レジストリ（Registry） | イメージを保管・配布するサーバー。Docker Hub が代表的 |
| レイヤー（Layer） | イメージを構成する差分の層。Dockerfile の各命令が 1 レイヤーを生成する |
| ボリューム（Volume） | コンテナのライフサイクルとは独立してデータを永続化する仕組み |
| Docker Daemon | コンテナ・イメージ・ネットワーク等を管理するバックグラウンドプロセス |
| Docker Compose | 複数コンテナのアプリケーションを YAML で定義・管理するツール |

## 5. 仕組み・アーキテクチャ

Docker はクライアント・サーバーアーキテクチャを採用している。ユーザーが Docker CLI（クライアント）でコマンドを実行すると、Docker Daemon（サーバー）がイメージのビルド、コンテナの実行、レジストリとの通信を行う。

![Docker アーキテクチャ](./img/docker_basics_architecture.svg)

イメージの作成から実行までのワークフローは以下のとおりである。Dockerfile からイメージをビルドし、イメージからコンテナを起動する。イメージはレジストリを介して共有できる。

![Docker ワークフロー](./img/docker_basics_workflow.svg)

## 6. 環境構築

### 6.1 必要なもの

- Docker Desktop（Windows / macOS）または Docker Engine（Linux）
- ターミナル（コマンドプロンプト / PowerShell / bash）

### 6.2 セットアップ手順

```bash
# Linux の場合: 公式スクリプトでインストールする
curl -fsSL https://get.docker.com | sh

# Docker サービスを起動する
sudo systemctl start docker

# 現在のユーザーを docker グループに追加する（sudo なしで実行可能にする）
sudo usermod -aG docker $USER
```

Windows / macOS の場合は Docker Desktop をインストーラからインストールする。

### 6.3 動作確認

```bash
# バージョンを確認する
docker --version

# テスト用コンテナを実行する
docker run hello-world
```

`Hello from Docker!` と表示されればセットアップ完了である。

## 7. 基本の使い方

以下は Python の Web アプリケーションをコンテナ化する最小構成の例である。

```dockerfile
# Python Web アプリケーションのコンテナイメージ定義

# ベースイメージとして Python 3.12 のスリム版を使用する
FROM python:3.12-slim

# コンテナ内の作業ディレクトリを設定する
WORKDIR /app

# 依存パッケージ定義ファイルをコピーする
COPY requirements.txt .

# 依存パッケージをインストールする
RUN pip install --no-cache-dir -r requirements.txt

# アプリケーションコードをコピーする
COPY app.py .

# コンテナ起動時に実行するコマンドを指定する
CMD ["python", "app.py"]
```

### 解説

- `FROM`: ベースとなるイメージを指定する。公式イメージを使うことで必要な環境が整った状態から始められる
- `WORKDIR`: 以降の命令が実行されるディレクトリを設定する
- `COPY`: ホストのファイルをコンテナ内にコピーする
- `RUN`: ビルド時にコマンドを実行する。ここではパッケージのインストールに使用する
- `CMD`: コンテナ起動時に実行されるデフォルトコマンドを定義する

```bash
# イメージをビルドする（-t でタグ名を指定する）
docker build -t my-app:1.0 .

# コンテナをバックグラウンドで起動する（-d: デタッチモード、-p: ポートマッピング）
docker run -d -p 8080:8080 --name my-app my-app:1.0

# 実行中のコンテナを一覧表示する
docker ps

# コンテナのログを表示する
docker logs my-app

# コンテナを停止する
docker stop my-app

# コンテナを削除する
docker rm my-app
```

## 8. ステップアップ

### 8.1 Docker Compose による複数コンテナ管理

実際のアプリケーションでは Web サーバーとデータベースなど複数のコンテナを組み合わせる。Docker Compose を使うと、これらを 1 つの YAML ファイルで定義し、一括で管理できる。

```yaml
# Docker Compose による Web + DB 構成の定義

services:
  # Web アプリケーションサービスを定義する
  web:
    # カレントディレクトリの Dockerfile からビルドする
    build: .
    # ホストのポート 8080 をコンテナのポート 8080 にマッピングする
    ports:
      - "8080:8080"
    # db サービスへの依存を宣言する
    depends_on:
      - db
  # データベースサービスを定義する
  db:
    # PostgreSQL 16 の公式イメージを使用する
    image: postgres:16
    # 環境変数でパスワードを設定する
    environment:
      POSTGRES_PASSWORD: example
    # 名前付きボリュームでデータを永続化する
    volumes:
      - db-data:/var/lib/postgresql/data

# 名前付きボリュームを定義する
volumes:
  db-data:
```

```bash
# すべてのサービスをバックグラウンドで起動する
docker compose up -d

# サービスの状態を確認する
docker compose ps

# すべてのサービスを停止・削除する
docker compose down
```

### 8.2 ボリュームによるデータ永続化

コンテナは停止・削除するとコンテナ内のデータも消失する。永続化が必要なデータにはボリュームを使用する。

```bash
# 名前付きボリュームを作成する
docker volume create my-data

# ボリュームをマウントしてコンテナを起動する
docker run -d -v my-data:/app/data --name my-app my-app:1.0

# ボリュームの一覧を表示する
docker volume ls
```

## 9. よくある落とし穴

- **コンテナを停止しても削除されない**: `docker stop` はコンテナを停止するだけである。ディスクを解放するには `docker rm` で削除する必要がある
- **イメージサイズの肥大化**: `RUN` 命令ごとにレイヤーが作成される。`apt-get install` と `apt-get clean` は同一の `RUN` で実行し、不要ファイルを残さない
- **ビルドキャッシュの無効化**: `COPY . .` を Dockerfile の上部に書くと、ソースコードが変わるたびに以降のすべてのレイヤーが再ビルドされる。変更頻度の低いファイル（依存定義）を先にコピーする
- **root ユーザーでの実行**: デフォルトではコンテナ内のプロセスは root で動作する。セキュリティ上、`USER` 命令で非 root ユーザーに切り替える

## 10. ベストプラクティス

- ベースイメージは `-slim` や `-alpine` などの軽量版を選択する
- `.dockerignore` ファイルで不要なファイルをビルドコンテキストから除外する
- マルチステージビルドを活用し、最終イメージにビルドツールを含めない
- 1 コンテナ = 1 プロセスの原則に従い、関心事を分離する
- コンテナイメージには具体的なバージョンタグを指定し、`latest` の使用を避ける

## 11. 演習問題

1. `nginx:1.27` イメージを使ってコンテナを起動し、ブラウザから `http://localhost:8080` でアクセスできることを確認せよ
2. 静的な HTML ファイルを含む Dockerfile を作成し、カスタムイメージをビルドして起動せよ
3. Docker Compose を使って nginx と PostgreSQL の 2 コンテナ構成を定義し、`docker compose up` で起動せよ

## 12. さらに学ぶには

- 公式ドキュメント: https://docs.docker.com/
- Docker Getting Started: https://docs.docker.com/get-started/
- Dockerfile リファレンス: https://docs.docker.com/reference/dockerfile/
- 関連 Knowledge: Kubernetes の基本は `../infra/kubernetes_basics.md` を参照

## 13. 参考資料

- Docker Documentation: https://docs.docker.com/
- Docker Hub: https://hub.docker.com/
- Dockerfile Best Practices: https://docs.docker.com/build/building/best-practices/
