/// セキュアストレージの抽象インターフェース
/// Flutter/pure Dart 両環境で使用できるよう依存を注入可能にする
/// H-010 監査対応: flutter_secure_storage への直接依存を抽象化し、
/// pure Dart 環境（CLIやサーバーサイドDart）でも独自実装を注入できるようにする
library;

/// トークン永続化ストレージの抽象インターフェース。
/// 実装は Flutter 環境（FlutterSecureStorage）か、テスト・pure Dart 環境で切り替える。
abstract class TokenStorage {
  /// 指定されたキーの値を読み取る。存在しない場合は null を返す。
  Future<String?> read({required String key});

  /// 指定されたキーに値を書き込む。
  Future<void> write({required String key, required String value});

  /// 指定されたキーを削除する。
  Future<void> delete({required String key});
}
