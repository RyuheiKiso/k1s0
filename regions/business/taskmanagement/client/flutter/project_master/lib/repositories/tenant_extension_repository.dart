import 'package:dio/dio.dart';

import '../models/tenant_extension.dart';
import '../utils/app_exception.dart';

/// テナント拡張APIのリポジトリ層
/// サーバーとの通信を担当し、テナント固有の拡張データの永続化・取得を行う
class TenantExtensionRepository {
  /// APIリクエストに使用するDioインスタンス
  final Dio _dio;

  /// コンストラクタ: Dioインスタンスを外部から注入する（テスタビリティのため）
  TenantExtensionRepository(this._dio);

  /// テナントの拡張設定一覧を取得する
  Future<List<TenantProjectExtension>> listTenantExtensions(String tenantId) async {
    // DioException をキャッチしてアプリ固有の例外に変換する（情報漏洩防止のため内部エラー詳細は隠蔽する）
    try {
      final response = await _dio.get(
        '/api/v1/taskmanagement/tenant-extensions',
        queryParameters: {'tenant_id': tenantId},
      );
      final List<dynamic> data =
          (response.data as Map<String, dynamic>)['extensions'] as List<dynamic>;
      return data
          .map((json) =>
              TenantProjectExtension.fromJson(json as Map<String, dynamic>))
          .toList();
    } on DioException catch (e) {
      throw AppException.fromDioException(e);
    } catch (e) {
      throw AppException.unknown(e.toString());
    }
  }

  /// テナント固有のステータス定義拡張情報を取得する
  Future<TenantProjectExtension> getTenantExtension(
    String tenantId,
    String statusDefinitionId,
  ) async {
    // DioException をキャッチしてアプリ固有の例外に変換する（情報漏洩防止のため内部エラー詳細は隠蔽する）
    try {
      final response = await _dio.get(
        '/api/v1/taskmanagement/tenant-extensions',
        queryParameters: {
          'tenant_id': tenantId,
          'status_definition_id': statusDefinitionId,
        },
      );
      return TenantProjectExtension.fromJson(
          response.data as Map<String, dynamic>);
    } on DioException catch (e) {
      throw AppException.fromDioException(e);
    } catch (e) {
      throw AppException.unknown(e.toString());
    }
  }

  /// テナント固有のステータス定義拡張情報を更新する（upsert）
  Future<TenantProjectExtension> upsertTenantExtension(
    String tenantId,
    String statusDefinitionId,
    UpdateTenantExtensionInput input,
  ) async {
    // DioException をキャッチしてアプリ固有の例外に変換する（情報漏洩防止のため内部エラー詳細は隠蔽する）
    try {
      final body = {
        'tenant_id': tenantId,
        'status_definition_id': statusDefinitionId,
        ...input.toJson(),
      };
      final response = await _dio.put(
        '/api/v1/taskmanagement/tenant-extensions',
        data: body,
      );
      return TenantProjectExtension.fromJson(
          response.data as Map<String, dynamic>);
    } on DioException catch (e) {
      throw AppException.fromDioException(e);
    } catch (e) {
      throw AppException.unknown(e.toString());
    }
  }

  /// テナント固有のステータス定義拡張情報を削除する
  Future<void> deleteTenantExtension(
    String tenantId,
    String statusDefinitionId,
  ) async {
    // DioException をキャッチしてアプリ固有の例外に変換する（情報漏洩防止のため内部エラー詳細は隠蔽する）
    try {
      await _dio.delete(
        '/api/v1/taskmanagement/tenant-extensions',
        queryParameters: {
          'tenant_id': tenantId,
          'status_definition_id': statusDefinitionId,
        },
      );
    } on DioException catch (e) {
      throw AppException.fromDioException(e);
    } catch (e) {
      throw AppException.unknown(e.toString());
    }
  }
}
