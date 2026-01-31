# ADR-0009: Kotlin バックエンドサポートの追加

## ステータス

承認済み

## コンテキスト

k1s0 は v0.2.1 で Python サポートを追加し、Rust・Go・C#・Python の 4 言語バックエンドをサポートしている。しかし、JVM エコシステムを活用したいチームや、Spring Boot / Ktor での開発経験があるチームからの需要がある。特に以下の要因がある:

- Android バックエンドとの技術スタック統一（Kotlin Multiplatform の可能性）
- JVM エコシステムの豊富なライブラリ群の活用
- 既存の Java/Kotlin 資産を持つチームの k1s0 プラットフォームへの移行需要
- Spring Boot と比較して軽量な Ktor フレームワークの採用によるマイクロサービス適性

## 決定

k1s0 において、`backend-kotlin` テンプレートタイプを追加し、Ktor ベースの Kotlin バックエンドサポートを導入する。

### 具体的な変更内容

1. **CLI 拡張**: `new-feature` および `new-domain` コマンドで `--type backend-kotlin` を選択可能にする
2. **テンプレート追加**: `CLI/templates/backend-kotlin/` に feature テンプレートと domain テンプレートを作成する
3. **フレームワークパッケージ**: `framework/backend/kotlin/` に共通パッケージを提供する
   - Tier 1: k1s0-error, k1s0-config, k1s0-validation
   - Tier 2: k1s0-observability, k1s0-grpc-server, k1s0-grpc-client, k1s0-health, k1s0-db, k1s0-domain-event, k1s0-resilience, k1s0-cache
   - Tier 3: k1s0-auth
4. **Lint 対応**: K020/K022/K029/K050/K053 の既存 lint ルールを Kotlin コードに対応させる
5. **CI/CD**: `kotlin.yml` ワークフローを追加する
6. **Clean Architecture 準拠**: 他言語と同じ 4 層構造を Kotlin パッケージ構成で実現する

### 技術選定

| 項目 | 選定 | 理由 |
|------|------|------|
| フレームワーク | Ktor 3.x | 軽量、Kotlin ネイティブ、マイクロサービスに適した非同期設計 |
| ビルドツール | Gradle Kotlin DSL | Kotlin プロジェクト標準、型安全なビルドスクリプト |
| DI | Koin | 軽量、Kotlin DSL ベース、コンパイル時依存なし |
| DB | Exposed + HikariCP | Kotlin ネイティブ ORM、型安全な SQL 構築 |
| gRPC | grpc-kotlin | Kotlin コルーチン対応の gRPC 実装 |
| テスト | JUnit 5 + kotest | Kotlin 標準テストフレームワーク |
| Lint | ktlint + detekt | Kotlin 標準の静的解析ツール |

## 理由

- **JVM エコシステム需要**: Java/Kotlin は企業でのシェアが高く、既存資産の活用や JVM ライブラリへのアクセスが重要
- **Ktor の選定**: Spring Boot と比較して軽量で、Kotlin コルーチンとの統合が優れており、マイクロサービスに適している
- **Koin の選定**: Kotlin DSL ベースで学習コストが低く、アノテーション処理が不要なためビルドが高速
- **Exposed の選定**: Kotlin ネイティブの型安全な SQL フレームワークで、JPA/Hibernate より軽量

## 結果

### ポジティブ

- k1s0 がサポートするバックエンド言語が 5 つに増加する
- JVM エコシステムを活用したいチームが k1s0 プラットフォームを採用可能になる
- backend-kotlin と frontend-android で Kotlin を共通言語として使用可能になる

### ネガティブ

- 5 つ目のバックエンド言語のメンテナンスコストが発生する
- lint ルールの Kotlin 対応パターンの保守コストが増加する
- Gradle ビルドの CI 時間が比較的長くなる可能性がある

### 関連 ADR

- [ADR-0001](ADR-0001-scope-and-prerequisites.md): スコープと前提条件
- [ADR-0006](ADR-0006-three-layer-architecture.md): 三層アーキテクチャ
- [ADR-0008](ADR-0008-python-backend-support.md): Python バックエンドサポート
