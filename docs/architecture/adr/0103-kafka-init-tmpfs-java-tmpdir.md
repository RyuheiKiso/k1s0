# ADR-0103: kafka-init の /tmp:noexec 維持と JAVA_TOOL_OPTIONS による JVM 展開先変更

## ステータス

承認済み

## コンテキスト

MED-012 監査対応として kafka-init コンテナの tmpfs に `:noexec` オプションを追加した（`/tmp:noexec`）。
目的は `/tmp` への任意バイナリ展開・実行を防ぐセキュリティ強化であった。

しかし外部技術監査（2026-04-05、CRIT-001）により、この変更が以下の副作用を引き起こすことが判明した：

- Apache Kafka の Java クライアント（kafka-topics.sh が呼び出す JVM）は起動時に `libzstd-jni` ネイティブライブラリを `/tmp` に展開してロードする
- `/tmp:noexec` マウントではネイティブライブラリの実行が拒否され、`UnsatisfiedLinkError` が発生する
- 結果として全 58 件の Kafka トピック作成に失敗し、Kafka を使用する全サービス（event-store, notification, quota 等）が機能しない

MED-012 コメントには「kafka-topics.sh はシェルスクリプトなので `/tmp` でバイナリ実行は不要」と記載していたが、
kafka-topics.sh が呼び出す JVM が libzstd-jni を `/tmp` に展開するという事実を見落としていた。

## 決定

`/tmp:noexec` オプションは維持し、JVM のネイティブライブラリ展開先を `JAVA_TOOL_OPTIONS` で変更する。

```yaml
# docker-compose.yaml の kafka-init サービス
tmpfs:
  - /tmp:noexec       # MED-012 対応のセキュリティ設定を維持
  - /var/tmp          # CRIT-001 対応: JVM の libzstd-jni 展開先（exec 許可）

environment:
  # CRIT-001 対応: /tmp:noexec を維持しつつ JVM の libzstd-jni 展開先を /var/tmp に変更
  JAVA_TOOL_OPTIONS: "-Djava.io.tmpdir=/var/tmp"
```

## 理由

JVM は `java.io.tmpdir` システムプロパティで指定されたディレクトリにネイティブライブラリを展開する。
`JAVA_TOOL_OPTIONS` 環境変数は全 JVM プロセス起動時に自動適用されるため、
kafka-topics.sh が起動する全 JVM プロセスの展開先を `/var/tmp` に変更できる。

`/var/tmp` は tmpfs でマウントするため（exec 許可）、ネイティブライブラリの実行が可能になる。
`/tmp:noexec` は引き続きそのままとなり、シェルスクリプト用の一時ファイル領域のセキュリティは維持される。

## 影響

**ポジティブな影響**:

- `/tmp:noexec` によるセキュリティが維持される（任意バイナリの `/tmp` 経由での実行防止）
- JVM の libzstd-jni ロードが成功し、全 58 件の Kafka トピックが正常に作成される
- event-store, notification, quota など Kafka に依存する全サービスが正常動作する
- `JAVA_TOOL_OPTIONS` は kafka-init の全 JVM プロセスに自動適用されるため、個別設定が不要

**ネガティブな影響・トレードオフ**:

- `/var/tmp` tmpfs の追加によりコンテナのメモリ使用量がわずかに増加する（libzstd-jni ファイルは数 MB 程度）
- `JAVA_TOOL_OPTIONS` は環境変数として可視であるため、他の JVM オプションとの競合に注意が必要

## 代替案

| 案 | 概要 | 採用しなかった理由 |
|----|------|-----------------|
| 案 A: `/tmp:noexec` を削除 | MED-012 対応を元に戻す | `/tmp` への任意バイナリ展開・実行が可能になりセキュリティが低下する |
| 案 B: kafka-topics.sh をラッパースクリプトで包み tmpdir を指定 | コマンド実行時のみ tmpdir を変更 | 全 JVM プロセスへの適用が困難で保守性が低い |
| 案 C: `/tmp` をコンテナ通常ファイルシステムに変更 | tmpfs 自体をやめる | コンテナ再起動後も `/tmp` に一時ファイルが残るリスクがある |

## 参考

- [MED-012 監査対応](../../../docker-compose.yaml) — kafka-init の `/tmp:noexec` 設定（修正元）
- [CRIT-001 外部技術監査指摘](../../../報告書.md) — `/tmp:noexec` による Kafka トピック作成全失敗
- [ADR-0103 実装](../../../docker-compose.yaml) — kafka-init サービスの tmpfs と environment 設定

## 更新履歴

| 日付 | 変更内容 | 変更者 |
|------|---------|--------|
| 2026-04-05 | 初版作成（CRIT-001 外部監査対応） | kiso ryuhei |
