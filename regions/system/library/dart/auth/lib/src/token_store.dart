/// トークン保存ストア
/// メモリストアと SecureStorage ストアの 2 種類を提供する。
/// H-010 監査対応: flutter_secure_storage への直接依存を廃止し、
/// TokenStorage 抽象インターフェースを通じて依存注入パターンに移行した。
/// pure Dart 環境では MemoryTokenStore を、Flutter 環境では
/// FlutterTokenStorage を注入した SecureTokenStore を使用する。
library;

import 'dart:async';
import 'dart:convert';
// エラーログ出力に使用する（M-30 監査対応）
import 'dart:developer' as developer;

import 'storage/token_storage.dart';
import 'types.dart';

/// トークンストアのインターフェース
abstract class TokenStore {
  /// トークンセットを取得する
  TokenSet? getTokenSet();

  /// トークンセットを保存する
  /// POLY-007 監査対応: Future<void> に変更し、呼び出し元で await 可能にする。
  /// SecureTokenStore は永続化の完了を待機できるようになった。
  /// MemoryTokenStore は同期的に完了するため、await しても即座に返る。
  Future<void> setTokenSet(TokenSet tokenSet);

  /// トークンセットを削除する
  void clearTokenSet();

  /// code_verifier を取得する
  String? getCodeVerifier();

  /// code_verifier を保存する
  /// H-008 監査対応: Future<void> に変更し、呼び出し元で await 可能にする。
  /// fire-and-forget を廃止し、書き込み失敗を上位に伝播させる。
  Future<void> setCodeVerifier(String verifier);

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

/// S-06 対応: TokenStorage 抽象インターフェースを使ったセキュアなトークンストア。
/// H-010 監査対応: flutter_secure_storage の直接依存を廃止し、TokenStorage を注入パターンに移行。
/// Flutter アプリでは FlutterTokenStorage を注入して使用する。
/// pure Dart 環境では MemoryTokenStore を使用することを推奨する。
///
/// 同期インターフェース（TokenStore）に対して内部キャッシュで応答し、
/// 書き込みは非同期で TokenStorage 実装に永続化する。
/// アプリ起動時に [load] を呼び出してキャッシュをストレージから復元すること。
///
/// 使用例（Flutter 環境）:
/// ```dart
/// import 'src/storage/flutter_token_storage.dart';
/// final store = SecureTokenStore(storage: FlutterTokenStorage());
/// await store.load(); // アプリ起動時に一度呼び出す
/// final client = AuthClient(AuthClientOptions(config: config, tokenStore: store));
/// ```
class SecureTokenStore implements TokenStore {
  /// H-010 監査対応: flutter_secure_storage への直接依存を廃止し、
  /// TokenStorage 抽象インターフェースを注入パターンで受け取る
  final TokenStorage _storage;

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
    required TokenStorage storage,
    String prefix = 'k1s0_auth_',
  })  : _storage = storage,
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
  // POLY-007 監査対応: Future<void> に変更し、セキュアストレージへの書き込みを await 可能にする。
  // 以前は fire-and-forget だったため、書き込み完了前にアプリが終了した場合にトークンが
  // 永続化されないリスクがあった。呼び出し元で await することで永続化の確実性が向上する。
  Future<void> setTokenSet(TokenSet tokenSet) async {
    _tokenSet = tokenSet;
    try {
      await _storage.write(
        key: '$_prefix$_kTokenSet',
        value: jsonEncode(tokenSet.toJson()),
      );
    } catch (e, st) {
      developer.log(
        'SecureTokenStore: トークンセットの書き込みに失敗しました',
        error: e,
        stackTrace: st,
        name: 'SecureTokenStore',
      );
      rethrow;
    }
  }

  @override
  void clearTokenSet() {
    _tokenSet = null;
    // fire-and-forget でセキュアストレージから削除する。失敗時はログ出力（M-30 監査対応）
    unawaited(_storage
        .delete(key: '$_prefix$_kTokenSet')
        .catchError((Object e, StackTrace st) {
      developer.log(
        'SecureTokenStore: トークンセットの削除に失敗しました',
        error: e,
        stackTrace: st,
        name: 'SecureTokenStore',
      );
    }));
  }

  @override
  String? getCodeVerifier() => _codeVerifier;

  @override
  // H-008 監査対応: Future<void> に変更し、セキュアストレージへの書き込みを await 可能にする。
  // fire-and-forget を廃止し、書き込み失敗を上位に伝播させることで PKCE フローの信頼性を向上する。
  Future<void> setCodeVerifier(String verifier) async {
    _codeVerifier = verifier;
    try {
      await _storage.write(key: '$_prefix$_kCodeVerifier', value: verifier);
    } catch (e, st) {
      developer.log(
        'SecureTokenStore: code_verifier の書き込みに失敗しました',
        error: e,
        stackTrace: st,
        name: 'SecureTokenStore',
      );
      rethrow;
    }
  }

  @override
  void clearCodeVerifier() {
    _codeVerifier = null;
    // fire-and-forget でセキュアストレージから削除する。失敗時はログ出力（M-30 監査対応）
    unawaited(_storage
        .delete(key: '$_prefix$_kCodeVerifier')
        .catchError((Object e, StackTrace st) {
      developer.log(
        'SecureTokenStore: code_verifier の削除に失敗しました',
        error: e,
        stackTrace: st,
        name: 'SecureTokenStore',
      );
    }));
  }

  @override
  String? getState() => _state;

  @override
  void setState(String state) {
    _state = state;
    // fire-and-forget でセキュアストレージに永続化する。失敗時はログ出力（M-30 監査対応）
    unawaited(_storage
        .write(key: '$_prefix$_kState', value: state)
        .catchError((Object e, StackTrace st) {
      developer.log(
        'SecureTokenStore: state の書き込みに失敗しました',
        error: e,
        stackTrace: st,
        name: 'SecureTokenStore',
      );
    }));
  }

  @override
  void clearState() {
    _state = null;
    // fire-and-forget でセキュアストレージから削除する。失敗時はログ出力（M-30 監査対応）
    unawaited(_storage
        .delete(key: '$_prefix$_kState')
        .catchError((Object e, StackTrace st) {
      developer.log(
        'SecureTokenStore: state の削除に失敗しました',
        error: e,
        stackTrace: st,
        name: 'SecureTokenStore',
      );
    }));
  }

  @override
  void clearAll() {
    _tokenSet = null;
    _codeVerifier = null;
    _state = null;
    // fire-and-forget で全データを一括削除する。失敗時はログ出力（M-30 監査対応）
    // catchError ハンドラは Future<List<void>> の型に合わせて空リストを返す必要がある
    unawaited(Future.wait([
      _storage.delete(key: '$_prefix$_kTokenSet'),
      _storage.delete(key: '$_prefix$_kCodeVerifier'),
      _storage.delete(key: '$_prefix$_kState'),
    ]).catchError((Object e, StackTrace st) {
      developer.log(
        'SecureTokenStore: 全データの削除に失敗しました',
        error: e,
        stackTrace: st,
        name: 'SecureTokenStore',
      );
      return <void>[];
    }));
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
  // POLY-007 監査対応: Future<void> に変更（インターフェースに合わせる）。即座に完了する。
  Future<void> setTokenSet(TokenSet tokenSet) async => _tokenSet = tokenSet;

  @override
  void clearTokenSet() => _tokenSet = null;

  @override
  String? getCodeVerifier() => _codeVerifier;

  @override
  // H-008 監査対応: Future<void> に変更（インターフェースに合わせる）。即座に完了する。
  Future<void> setCodeVerifier(String verifier) async => _codeVerifier = verifier;

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
