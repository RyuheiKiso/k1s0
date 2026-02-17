/// トークン保存ストア
/// メモリストアと SecureStorage ストアの 2 種類を提供する。
/// flutter_secure_storage は Pure Dart では使えないため、
/// 抽象インターフェースを通じてオプショナルに対応する。

import 'types.dart';

/// トークンストアのインターフェース
abstract class TokenStore {
  /// トークンセットを取得する
  TokenSet? getTokenSet();

  /// トークンセットを保存する
  void setTokenSet(TokenSet tokenSet);

  /// トークンセットを削除する
  void clearTokenSet();

  /// code_verifier を取得する
  String? getCodeVerifier();

  /// code_verifier を保存する
  void setCodeVerifier(String verifier);

  /// code_verifier を削除する
  void clearCodeVerifier();

  /// state を取得する
  String? getState();

  /// state を保存する
  void setState(String state);

  /// state を削除する
  void clearState();

  /// すべてのデータを削除する
  void clearAll();
}

/// メモリベースのトークンストア。
/// テスト用、または Pure Dart 環境で使用する。
class MemoryTokenStore implements TokenStore {
  TokenSet? _tokenSet;
  String? _codeVerifier;
  String? _state;

  @override
  TokenSet? getTokenSet() => _tokenSet;

  @override
  void setTokenSet(TokenSet tokenSet) => _tokenSet = tokenSet;

  @override
  void clearTokenSet() => _tokenSet = null;

  @override
  String? getCodeVerifier() => _codeVerifier;

  @override
  void setCodeVerifier(String verifier) => _codeVerifier = verifier;

  @override
  void clearCodeVerifier() => _codeVerifier = null;

  @override
  String? getState() => _state;

  @override
  void setState(String state) => _state = state;

  @override
  void clearState() => _state = null;

  @override
  void clearAll() {
    _tokenSet = null;
    _codeVerifier = null;
    _state = null;
  }
}
