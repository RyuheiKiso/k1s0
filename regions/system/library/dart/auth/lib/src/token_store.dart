/// トークン保存ストア
/// メモリストアと SecureStorage ストアの 2 種類を提供する。
/// flutter_secure_storage は Pure Dart では使えないため、
/// 抽象インターフェースを通じてオプショナルに対応する。
library;

import 'dart:async';
import 'dart:convert';

import 'package:flutter_secure_storage/flutter_secure_storage.dart';

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

/// S-06 対応: flutter_secure_storage を使ったセキュアなトークンストア。
/// Flutter アプリ（iOS/Android/macOS/Windows/Linux）での長期トークン保管に使用する。
///
/// 同期インターフェース（TokenStore）に対して内部キャッシュで応答し、
/// 書き込みは非同期で flutter_secure_storage に永続化する。
/// アプリ起動時に [load] を呼び出してキャッシュをストレージから復元すること。
///
/// 使用例:
/// ```dart
/// final store = SecureTokenStore();
/// await store.load(); // アプリ起動時に一度呼び出す
/// final client = AuthClient(config: config, store: store);
/// ```
class SecureTokenStore implements TokenStore {
  final FlutterSecureStorage _storage;

  /// セキュアストレージのキープレフィックス（複数インスタンス共存を可能にする）
  final String _prefix;

  // キャッシュキー定数
  static const String _kTokenSet = 'token_set';
  static const String _kCodeVerifier = 'code_verifier';
  static const String _kState = 'state';

  // インメモリキャッシュ（同期アクセス用）
  TokenSet? _tokenSet;
  String? _codeVerifier;
  String? _state;

  SecureTokenStore({
    FlutterSecureStorage? storage,
    String prefix = 'k1s0_auth_',
  })  : _storage = storage ?? const FlutterSecureStorage(),
        _prefix = prefix;

  /// ストレージからキャッシュを初期化する。
  /// アプリ起動時に一度呼び出すことでトークンが復元される。
  Future<void> load() async {
    final tokenSetJson = await _storage.read(key: '$_prefix$_kTokenSet');
    if (tokenSetJson != null) {
      try {
        _tokenSet = TokenSet.fromJson(
            jsonDecode(tokenSetJson) as Map<String, dynamic>);
      } catch (_) {
        // 不正なデータは無視してキャッシュをクリアする
        _tokenSet = null;
      }
    }
    _codeVerifier = await _storage.read(key: '$_prefix$_kCodeVerifier');
    _state = await _storage.read(key: '$_prefix$_kState');
  }

  @override
  TokenSet? getTokenSet() => _tokenSet;

  @override
  void setTokenSet(TokenSet tokenSet) {
    _tokenSet = tokenSet;
    // fire-and-forget でセキュアストレージに永続化する
    unawaited(_storage.write(
      key: '$_prefix$_kTokenSet',
      value: jsonEncode(tokenSet.toJson()),
    ));
  }

  @override
  void clearTokenSet() {
    _tokenSet = null;
    unawaited(_storage.delete(key: '$_prefix$_kTokenSet'));
  }

  @override
  String? getCodeVerifier() => _codeVerifier;

  @override
  void setCodeVerifier(String verifier) {
    _codeVerifier = verifier;
    unawaited(_storage.write(key: '$_prefix$_kCodeVerifier', value: verifier));
  }

  @override
  void clearCodeVerifier() {
    _codeVerifier = null;
    unawaited(_storage.delete(key: '$_prefix$_kCodeVerifier'));
  }

  @override
  String? getState() => _state;

  @override
  void setState(String state) {
    _state = state;
    unawaited(_storage.write(key: '$_prefix$_kState', value: state));
  }

  @override
  void clearState() {
    _state = null;
    unawaited(_storage.delete(key: '$_prefix$_kState'));
  }

  @override
  void clearAll() {
    _tokenSet = null;
    _codeVerifier = null;
    _state = null;
    unawaited(Future.wait([
      _storage.delete(key: '$_prefix$_kTokenSet'),
      _storage.delete(key: '$_prefix$_kCodeVerifier'),
      _storage.delete(key: '$_prefix$_kState'),
    ]));
  }
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
