# ADR-0096: Dart Auth ライブラリの flutter_secure_storage 依存抽象化

## ステータス

承認済み

## コンテキスト

外部技術監査（H-010）において、`regions/system/library/dart/auth` の `pubspec.yaml` が `flutter_secure_storage: ^9.2.0` を直接依存（`dependencies`）として宣言していることが指摘された。

この構成には以下の問題がある:

1. **pure Dart 環境での使用不可**: `flutter_secure_storage` は Flutter フレームワーク（iOS/Android/macOS/Windows/Linux プラットフォームプラグイン）に依存しており、pure Dart 環境（CLI ツール、サーバーサイド Dart、テスト環境）でのパッケージ解決が失敗する。
2. **テストの困難**: Flutter プラグインの存在により、純粋な Dart テストランナー（`dart test`）での実行が困難になる。
3. **モック差し替えの不可**: `FlutterSecureStorage` が直接インスタンス化されており、テスト用にモックへ差し替えることができない。

k1s0 プロジェクトでは Flutter（モバイル/デスクトップ）と pure Dart（CLI/サーバー）の両環境をサポートする必要があるため、認証ライブラリは両環境で使用可能でなければならない。

## 決定

`flutter_secure_storage` への直接依存を廃止し、`TokenStorage` 抽象インターフェースを導入する依存注入パターンへ移行する。

1. `lib/src/storage/token_storage.dart` に `TokenStorage` 抽象クラスを新規作成する（`read`/`write`/`delete` の3メソッド）。
2. `lib/src/storage/flutter_token_storage.dart` に `FlutterTokenStorage implements TokenStorage` を作成し、`FlutterSecureStorage` をラップする。
3. `SecureTokenStore` のコンストラクタを `required TokenStorage storage` パラメータを受け取る形に変更する（依存注入パターン）。
4. `pubspec.yaml` の `flutter_secure_storage` を `dependencies` から除去し、参照コメントのみ残す。`dev_dependencies` に移動してテスト用途に限定する。
5. `lib/auth.dart` から `TokenStorage` インターフェースをエクスポートし、ライブラリ利用者が独自実装を提供できるようにする。

## 理由

- **依存注入パターン（DI）**: `SecureTokenStore` が具体的な実装ではなく抽象インターフェースに依存することで、疎結合を実現する。
- **テスト容易性**: テストでは `MemoryTokenStore` またはモック実装の `TokenStorage` を注入することで、Flutter プラグインなしでテストを実行できる。
- **pure Dart 互換性**: `dependencies` から `flutter_secure_storage` を除去することで、pure Dart 環境でのパッケージ解決エラーを回避する。
- **拡張性**: 将来的に `SharedPreferences` や独自暗号化ストレージなど、別のストレージバックエンドへの差し替えが容易になる。

## 影響

**ポジティブな影響**:

- pure Dart 環境（CLI、サーバーサイド Dart）での `k1s0_auth` パッケージ使用が可能になる
- `dart test` によるテスト実行がシンプルになる（Flutter SDK 不要）
- ストレージ実装のモック差し替えが容易になり、テストの信頼性が向上する
- 独自ストレージバックエンドへの拡張が可能になる

**ネガティブな影響・トレードオフ**:

- 既存の Flutter アプリが `SecureTokenStore()` を引数なしで生成している場合、`SecureTokenStore(storage: FlutterTokenStorage())` への書き換えが必要（breaking change）
- `FlutterTokenStorage` を使う場合、Flutter アプリ側の `pubspec.yaml` に `flutter_secure_storage` を追加する必要がある
- コンストラクタシグネチャ変更に伴うマイグレーションコストが発生する

## 代替案

| 案 | 概要 | 採用しなかった理由 |
|----|------|-----------------|
| 案 A | `flutter_secure_storage` を `dev_dependencies` にのみ残しつつ `SecureTokenStore` 内部でオプショナル import を使う | Dart は条件付きインポートの制約があり、実装が複雑になる |
| 案 B | `SecureTokenStore` をライブラリから分離し、別パッケージ（`k1s0_auth_flutter`）として提供する | パッケージ分割はオーバーエンジニアリングであり、現時点の規模には不適切 |
| 案 C | 変更なし（現状維持） | pure Dart 環境での使用不可という監査指摘を解消できない |

## 参考

- [ADR-0045: Vault per-service ロール分離](0045-vault-per-service-role.md)
- [H-010 外部技術監査レポート](../../../tasks/)
- [flutter_secure_storage パッケージ](https://pub.dev/packages/flutter_secure_storage)

## 更新履歴

| 日付 | 変更内容 | 変更者 |
|------|---------|--------|
| 2026-04-04 | 初版作成（H-010 監査対応） | @kiso-ryuhei |
