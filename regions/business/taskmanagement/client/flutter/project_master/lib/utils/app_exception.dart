// アプリ固有の例外クラス: DioException などの外部例外をアプリ内部の例外に変換する
// 情報漏洩防止のため、外部通信エラーの詳細はログに記録せずに汎用メッセージに変換する
import 'package:dio/dio.dart';

/// アプリ固有の例外クラス
/// 外部 API 通信エラーや予期しない例外を統一的に扱うための例外型
class AppException implements Exception {
  /// ユーザー向けのエラーメッセージ（内部エラー詳細を含まない汎用メッセージ）
  final String message;

  /// エラーコード（HTTP ステータスコードや内部エラーコードを格納する）
  final int? code;

  const AppException(this.message, {this.code});

  /// DioException をアプリ固有の例外に変換するファクトリコンストラクタ
  /// 情報漏洩防止のため、レスポンスボディの詳細は隠蔽して汎用メッセージを返す
  factory AppException.fromDioException(DioException e) {
    switch (e.type) {
      case DioExceptionType.connectionTimeout:
      case DioExceptionType.sendTimeout:
      case DioExceptionType.receiveTimeout:
        // タイムアウト発生時は接続エラーとして汎用メッセージを返す
        return const AppException('接続がタイムアウトしました。しばらく経ってから再試行してください。', code: 408);
      case DioExceptionType.badResponse:
        // HTTP エラーレスポンス: ステータスコードのみを保持し、ボディ詳細は隠蔽する
        final statusCode = e.response?.statusCode;
        if (statusCode == 401) {
          return AppException('認証エラーが発生しました。再ログインしてください。', code: statusCode);
        } else if (statusCode == 403) {
          return AppException('この操作を実行する権限がありません。', code: statusCode);
        } else if (statusCode == 404) {
          return AppException('リソースが見つかりませんでした。', code: statusCode);
        } else if (statusCode != null && statusCode >= 500) {
          return AppException('サーバーエラーが発生しました。しばらく経ってから再試行してください。', code: statusCode);
        }
        return AppException('リクエストが失敗しました。', code: statusCode);
      case DioExceptionType.cancel:
        // リクエストキャンセル時は汎用メッセージを返す
        return const AppException('リクエストがキャンセルされました。');
      case DioExceptionType.connectionError:
        // ネットワーク接続エラー時は汎用メッセージを返す
        return const AppException('ネットワーク接続エラーが発生しました。接続を確認してください。');
      default:
        return const AppException('通信エラーが発生しました。しばらく経ってから再試行してください。');
    }
  }

  /// 予期しない例外を汎用メッセージに変換するファクトリコンストラクタ
  /// 内部エラー詳細は隠蔽して汎用メッセージを返す
  factory AppException.unknown(String detail) {
    // 詳細ログは上位で記録するため、ユーザー向けには汎用メッセージのみを返す
    return const AppException('予期しないエラーが発生しました。');
  }

  @override
  String toString() => 'AppException: $message (code: $code)';
}
