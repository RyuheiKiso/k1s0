// system_client の ApiClient を使用して BFF の CSRF 契約に準拠する
// 旧 DioClient は x-csrf-token レスポンスヘッダーを読んでいたが、
// BFF は /auth/session の JSON ボディで csrf_token を返すため誤りだった
// 本ファイルは後方互換のために sys.ApiClient.create への委譲を行う
import 'package:dio/dio.dart';
import 'package:system_client/system_client.dart' as sys;

/// ApiClient: system_client の ApiClient.create() に委譲してDioインスタンスを生成する
/// CSRF 契約を正しく実装するためにシステムクライアントを使用する
class ApiClient {
  /// 指定されたベースURLでDioインスタンスを生成する
  /// system_client の ApiClient.create() に委譲し、CSRF 契約を正しく実装する
  static Dio create({
    required String baseUrl,
    sys.CsrfTokenProvider? csrfTokenProvider,
  }) {
    return sys.ApiClient.create(
      baseUrl: baseUrl,
      csrfTokenProvider: csrfTokenProvider,
    );
  }
}
