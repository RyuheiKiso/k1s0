# Testcontainers: 基本

- 対象読者: Docker の基本操作を理解している開発者
- 学習目標: Testcontainers の仕組みを理解し、Rust でインテグレーションテストを書けるようになる
- 所要時間: 約 30 分
- 対象バージョン: testcontainers-rs 0.24
- 最終更新日: 2026-04-12

## 1. このドキュメントで学べること

- Testcontainers が解決する課題と存在意義を説明できる
- テストコードから Docker コンテナを起動・管理する仕組みを理解できる
- Rust（testcontainers-rs）で Redis コンテナを使ったインテグレーションテストを書ける
- WaitFor 戦略やポートマッピングの基本を使い分けられる

## 2. 前提知識

- Docker コンテナの基本操作（[Docker: 基本](../tool/docker_basics.md)）
- Rust のテスト機構（`#[test]`、`#[tokio::test]`）の基礎知識
- 非同期プログラミング（async/await）の基礎概念

## 3. 概要

Testcontainers は、テストコードからプログラム的に Docker コンテナを起動・管理するためのオープンソースライブラリである。データベース、メッセージブローカー、キャッシュなど、本番と同じ実サービスをテスト用に一時的に立ち上げ、テスト終了後に自動で破棄する。

従来のインテグレーションテストでは、モックを使うか、事前に外部サービスを手動で起動しておく必要があった。モックは実装との乖離が生じやすく、手動セットアップは環境差異によるテスト失敗を招く。Testcontainers はこれらの課題を解消し、テストコード内でコンテナのライフサイクルを完結させる。

Java、Go、Python、Rust、.NET、Node.js など主要言語向けの実装が提供されている。本ドキュメントでは Rust 向けの `testcontainers-rs` クレートを対象とする。

## 4. 用語の整理

| 用語 | 説明 |
|------|------|
| GenericImage | 任意の Docker イメージを指定してコンテナを定義する構造体 |
| Container | 起動済みコンテナを表すハンドル。Drop 時に自動でコンテナを停止・削除する |
| WaitFor | コンテナの起動完了を判定する戦略（ログメッセージ検知、ポート疎通など） |
| AsyncRunner / SyncRunner | コンテナを非同期 / 同期で起動するためのトレイト |
| ポートマッピング | コンテナ内部ポートをホスト側のランダムポートに割り当てる仕組み |
| ImageExt | 環境変数やネットワーク設定を追加するための拡張トレイト |

## 5. 仕組み・アーキテクチャ

Testcontainers は Docker Engine の API を通じてコンテナを操作する。テストコードがライブラリの API を呼び出すと、Docker Engine にコンテナの作成・起動を指示し、WaitFor 条件が満たされるまで待機する。

![Testcontainers アーキテクチャ](./img/testcontainers_basics_architecture.svg)

テスト実行のライフサイクルは以下の 5 ステップで構成される。Rust では RAII パターンにより、Container がスコープを抜けると自動的にコンテナが停止・削除される。

![Testcontainers テストライフサイクル](./img/testcontainers_basics_lifecycle.svg)

## 6. 環境構築

### 6.1 必要なもの

- Docker Engine（Docker Desktop または Docker CE）が動作していること
- Rust ツールチェイン（rustup 経由）
- Tokio ランタイム（非同期テストを実行する場合）

### 6.2 セットアップ手順

```bash
# testcontainers クレートを依存関係に追加する
cargo add testcontainers

# 非同期テスト用に tokio を追加する（test feature 付き）
cargo add tokio --features full --dev
```

### 6.3 動作確認

```bash
# Docker Engine が動作していることを確認する
docker info

# テストを実行する（後述のコード例を追加した後に実行する）
cargo test
```

## 7. 基本の使い方

Redis コンテナを起動してインテグレーションテストを行う最小構成の例を示す。

```rust
// Testcontainers を使った Redis インテグレーションテストの最小構成
use testcontainers::{
    // コンテナポートの変換トレイトをインポートする
    core::{IntoContainerPort, WaitFor},
    // 非同期でコンテナを起動するトレイトをインポートする
    runners::AsyncRunner,
    // 汎用イメージ定義をインポートする
    GenericImage,
};

// 非同期テスト関数を定義する
#[tokio::test]
async fn test_with_redis() {
    // Redis 7.2.4 イメージでコンテナを定義する
    let container = GenericImage::new("redis", "7.2.4")
        // コンテナ内部の 6379 番ポートを公開する
        .with_exposed_port(6379.tcp())
        // 標準出力に準備完了メッセージが出るまで待機する
        .with_wait_for(WaitFor::message_on_stdout(
            "Ready to accept connections",
        ))
        // コンテナを起動する
        .start()
        .await
        // 起動失敗時はパニックする
        .expect("Redis コンテナの起動に失敗");

    // ホスト側に割り当てられたポート番号を取得する
    let host_port = container
        .get_host_port_ipv4(6379)
        .await
        .expect("ポート取得に失敗");

    // 取得したポートを使って Redis に接続する
    println!("Redis is available at localhost:{}", host_port);

    // テストロジックをここに記述する

    // container がスコープを抜けると自動でコンテナが停止・削除される
}
```

### 解説

- `GenericImage::new("redis", "7.2.4")` で使用する Docker イメージとタグを指定する
- `.with_exposed_port(6379.tcp())` でコンテナのポートを公開し、ホスト側にランダムなポートが割り当てられる
- `.with_wait_for(...)` でコンテナの起動完了判定の戦略を設定する
- `.start().await` で実際にコンテナを起動する
- `get_host_port_ipv4(6379)` でホスト側のマッピングポートを取得し、テスト対象のクライアントに渡す

## 8. ステップアップ

### 8.1 環境変数とネットワークの設定

`ImageExt` トレイトを使うと、環境変数やネットワークモードを指定できる。

```rust
// ImageExt を使った環境変数とネットワークの設定例
use testcontainers::{
    // 必要なモジュールをインポートする
    core::{IntoContainerPort, WaitFor},
    runners::AsyncRunner,
    // ImageExt トレイトをインポートする
    GenericImage, ImageExt,
};

#[tokio::test]
async fn test_with_env() {
    // PostgreSQL コンテナを環境変数付きで起動する
    let container = GenericImage::new("postgres", "16")
        // PostgreSQL のポートを公開する
        .with_exposed_port(5432.tcp())
        // データベースのパスワードを環境変数で設定する
        .with_env_var("POSTGRES_PASSWORD", "test")
        // データベース名を環境変数で設定する
        .with_env_var("POSTGRES_DB", "testdb")
        // ログメッセージで起動完了を判定する
        .with_wait_for(WaitFor::message_on_stdout(
            "database system is ready to accept connections",
        ))
        // コンテナを起動する
        .start()
        .await
        .expect("PostgreSQL コンテナの起動に失敗");

    // ホスト側のポートを取得する
    let port = container.get_host_port_ipv4(5432).await.unwrap();
    // 接続文字列を構築する
    let url = format!("postgres://postgres:test@localhost:{}/testdb", port);
    println!("PostgreSQL: {}", url);
}
```

### 8.2 コンテナの手動管理

自動クリーンアップではなく、明示的にコンテナを停止・削除することも可能である。

```rust
// コンテナを明示的に停止する
container.stop().await.expect("停止に失敗");
// コンテナを明示的に削除する
container.rm().await.expect("削除に失敗");
```

## 9. よくある落とし穴

- **Docker 未起動**: Docker Engine が停止していると `start()` がエラーになる。CI 環境では Docker サービスの起動を確認すること
- **ポートのハードコード**: ホスト側ポートはランダムに割り当てられるため、固定値を使うとテストが失敗する。必ず `get_host_port_ipv4()` で動的に取得すること
- **WaitFor 未設定**: WaitFor を省略するとコンテナ起動直後にテストが始まり、サービスが未準備でテストが失敗する
- **CI でのイメージ取得時間**: 初回実行時は Docker イメージのプルに時間がかかる。CI のタイムアウト値を十分に設定すること
- **並列テストでのポート競合**: 複数テストが同じコンテナ名を使うと競合する。コンテナ名を指定しない（自動生成に任せる）ことで回避できる

## 10. ベストプラクティス

- テストごとに独立したコンテナを起動し、テスト間の状態共有を避ける
- WaitFor 条件を適切に設定し、サービスの準備完了を確実に待機する
- Docker イメージのタグを固定し（`latest` を避け）、テストの再現性を確保する
- CI パイプラインではイメージのキャッシュを活用してテスト実行時間を短縮する
- 同期テストには `SyncRunner`、非同期テストには `AsyncRunner` を使い分ける

## 11. 演習問題

1. Redis コンテナを起動し、`get_host_port_ipv4()` で取得したポートを標準出力に表示するテストを書け
2. PostgreSQL コンテナを環境変数付きで起動し、接続文字列を組み立てるテストを書け
3. 同じテスト内で Redis と PostgreSQL の 2 つのコンテナを同時に起動するテストを書け

## 12. さらに学ぶには

- 公式ドキュメント: <https://testcontainers.com/>
- testcontainers-rs GitHub: <https://github.com/testcontainers/testcontainers-rs>
- 関連 Knowledge: [Docker: 基本](../tool/docker_basics.md)

## 13. 参考資料

- Testcontainers 公式サイト: <https://testcontainers.com/>
- testcontainers-rs クレートドキュメント: <https://docs.rs/testcontainers/>
- testcontainers-rs GitHub リポジトリ: <https://github.com/testcontainers/testcontainers-rs>
- Testcontainers Getting Started Guide: <https://testcontainers.com/guides/>
