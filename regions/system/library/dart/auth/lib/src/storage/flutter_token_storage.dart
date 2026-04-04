/// flutter_secure_storage を使った TokenStorage 実装
/// H-010 監査対応: flutter_secure_storage を TokenStorage インターフェース越しに使用する
/// Flutter アプリ（iOS/Android/macOS/Windows/Linux）での利用を想定
library;

import 'package:flutter_secure_storage/flutter_secure_storage.dart';

import 'token_storage.dart';

/// flutter_secure_storage をラップした TokenStorage 実装。
/// Flutter 環境で SecureTokenStore に注入して使用する。
class FlutterTokenStorage implements TokenStorage {
  /// 内部で使用する FlutterSecureStorage インスタンス
  final FlutterSecureStorage _storage;

  /// FlutterSecureStorage を直接受け取るか、デフォルトインスタンスを使用する
  FlutterTokenStorage({FlutterSecureStorage? storage})
      : _storage = storage ?? const FlutterSecureStorage();

  @override
  // 指定されたキーの値を FlutterSecureStorage から読み取る
  Future<String?> read({required String key}) => _storage.read(key: key);

  @override
  // 指定されたキーに値を FlutterSecureStorage へ書き込む
  Future<void> write({required String key, required String value}) =>
      _storage.write(key: key, value: value);

  @override
  // 指定されたキーを FlutterSecureStorage から削除する
  Future<void> delete({required String key}) => _storage.delete(key: key);
}
