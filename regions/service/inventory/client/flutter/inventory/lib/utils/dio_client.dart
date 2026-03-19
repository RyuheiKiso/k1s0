import 'package:dio/dio.dart';

/// CSRFトークンを管理するインターセプター
/// サーバーから取得したCSRFトークンをリクエストヘッダーに自動付与する
class CsrfTokenInterceptor extends Interceptor {
  /// 現在保持しているCSRFトークン
  String? _csrfToken;

  /// レスポンスからCSRFトークンを抽出して保持する
  @override
  void onResponse(Response response, ResponseInterceptorHandler handler) {
    final token = response.headers.value('x-csrf-token');
    if (token != null) {
      _csrfToken = token;
    }
    handler.next(response);
  }

  /// リクエスト送信前にCSRFトークンをヘッダーに付与する
  @override
  void onRequest(RequestOptions options, RequestInterceptorHandler handler) {
    if (_csrfToken != null) {
      options.headers['x-csrf-token'] = _csrfToken;
    }
    handler.next(options);
  }
}

/// エラーレスポンスを統一的に処理するインターセプター
/// サーバーエラーをアプリケーション固有の例外に変換する
class ErrorInterceptor extends Interceptor {
  @override
  void onError(DioException err, ErrorInterceptorHandler handler) {
    final statusCode = err.response?.statusCode;
    final message = (err.response?.data as Map<String, dynamic>?)?['error'] ?? err.message;

    /// ステータスコードに応じたエラーメッセージを生成する
    final errorMessage = switch (statusCode) {
      400 => 'リクエストが不正です: $message',
      401 => '認証が必要です',
      403 => 'アクセス権限がありません',
      404 => 'リソースが見つかりません',
      409 => '競合が発生しました: $message',
      422 => 'バリデーションエラー: $message',
      _ => 'サーバーエラーが発生しました: $message',
    };

    handler.next(DioException(
      requestOptions: err.requestOptions,
      response: err.response,
      type: err.type,
      message: errorMessage,
    ));
  }
}

/// Dioクライアントのファクトリクラス
/// 共通設定を適用したDioインスタンスを生成する
class DioClient {
  /// 指定されたベースURLでDioインスタンスを生成する
  /// Cookie認証とCSRFトークン管理を自動設定する
  static Dio create({required String baseUrl}) {
    final dio = Dio(BaseOptions(
      baseUrl: baseUrl,
      connectTimeout: const Duration(seconds: 30),
      receiveTimeout: const Duration(seconds: 30),
      headers: {
        'Content-Type': 'application/json',
        'Accept': 'application/json',
      },
      /// Cookie認証を使用するためcredentialsを有効化する
      extra: {'withCredentials': true},
    ));

    /// CSRFトークン管理とエラーハンドリングのインターセプターを追加する
    dio.interceptors.addAll([
      CsrfTokenInterceptor(),
      ErrorInterceptor(),
    ]);

    return dio;
  }
}
