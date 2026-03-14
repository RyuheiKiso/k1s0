/// アプリアップデーターの基底エラークラス。
/// すべてのエラーはこのクラスを継承し、[message] と [code] を持つ。
class AppUpdaterError implements Exception {
  /// エラーの詳細メッセージ。人間が読める形式で記述する。
  final String message;

  /// エラーを識別するコード。プログラム的なエラー判定に使用する。
  final String code;

  /// [AppUpdaterError] を生成する。
  const AppUpdaterError(this.message, this.code);

  @override
  String toString() => 'AppUpdaterError($code): $message';
}

/// サーバーへの接続エラー。
/// ネットワーク障害、タイムアウト、または HTTP エラーレスポンスが原因で発生する。
class ConnectionError extends AppUpdaterError {
  const ConnectionError(String message) : super(message, 'CONNECTION_ERROR');
}

/// 設定値が不正な場合のエラー。
/// serverUrl や appId が空文字の場合にコンストラクタ内でスローされる。
class InvalidConfigError extends AppUpdaterError {
  const InvalidConfigError(String message) : super(message, 'INVALID_CONFIG');
}

/// レスポンスのパースに失敗した場合のエラー。
/// JSONの形式が期待と異なる場合や、必須フィールドが欠落している場合に発生する。
class ParseError extends AppUpdaterError {
  const ParseError(String message) : super(message, 'PARSE_ERROR');
}

/// 認証・認可エラー。
/// HTTP 401 または 403 レスポンスが返された場合にスローされる。
class UnauthorizedError extends AppUpdaterError {
  const UnauthorizedError(String message) : super(message, 'UNAUTHORIZED');
}

/// アプリが見つからない場合のエラー。
/// レジストリサーバーに指定したアプリIDが存在しない場合に発生する。
class AppNotFoundError extends AppUpdaterError {
  const AppNotFoundError(String message) : super(message, 'APP_NOT_FOUND');
}

/// 指定バージョンが見つからない場合のエラー。
/// HTTP 404 レスポンスが返された場合にスローされる。
class VersionNotFoundError extends AppUpdaterError {
  const VersionNotFoundError(String message)
      : super(message, 'VERSION_NOT_FOUND');
}

/// ストアURLが取得できない場合のエラー。
/// 非対応プラットフォームや、ストアURLが設定されていない場合に使用する。
class StoreUrlUnavailableError extends AppUpdaterError {
  const StoreUrlUnavailableError(String message)
      : super(message, 'STORE_URL_UNAVAILABLE');
}

/// チェックサムの検証に失敗した場合のエラー。
/// ダウンロードしたファイルのSHA-256ハッシュが期待値と一致しない場合にスローされる。
class ChecksumError extends AppUpdaterError {
  const ChecksumError(String message) : super(message, 'CHECKSUM_ERROR');
}

/// 署名の検証に失敗した場合のエラー。
/// ダウンロードしたファイルのデジタル署名が正当でない場合にスローされる。
class SignatureVerificationError extends AppUpdaterError {
  const SignatureVerificationError(String message)
      : super(message, 'SIGNATURE_VERIFICATION_FAILED');
}

/// ファイルのダウンロードに失敗した場合のエラー。
/// ネットワーク障害やストレージの問題でダウンロードが完了しなかった場合に発生する。
class DownloadError extends AppUpdaterError {
  const DownloadError(String message) : super(message, 'DOWNLOAD_ERROR');
}
