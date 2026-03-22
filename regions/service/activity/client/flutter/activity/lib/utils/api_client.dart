// system_client の ApiClient を使用して BFF の CSRF 契約に準拠する
// 旧 DioClient は x-csrf-token レスポンスヘッダーを読んでいたが、
// BFF は /auth/session の JSON ボディで csrf_token を返すため誤りだった
// 本ファイルは後方互換のために ApiClient.create へ委譲する
import 'package:dio/dio.dart';
import 'package:system_client/system_client.dart';

/// ApiClient のファクトリメソッドラッパー
/// system_client の ApiClient.create() に委譲し、CSRF 契約を正しく実装する
class ActivityApiClient {
  /// 指定されたベースURLでDioインスタンスを生成する
  /// system_client の ApiClient.create() に委譲する
  static Dio create({required String baseUrl, CsrfTokenProvider? csrfTokenProvider}) {
    return ApiClient.create(
      baseUrl: baseUrl,
      csrfTokenProvider: csrfTokenProvider,
    );
  }
}
