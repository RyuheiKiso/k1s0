import 'package:dio/dio.dart';

import '../models/project_type.dart';
import '../utils/app_exception.dart';

/// プロジェクトタイプAPIのリポジトリ層
/// サーバーとの通信を担当し、プロジェクトタイプデータの永続化・取得を行う
class ProjectTypeRepository {
  /// APIリクエストに使用するDioインスタンス
  final Dio _dio;

  /// コンストラクタ: Dioインスタンスを外部から注入する（テスタビリティのため）
  ProjectTypeRepository(this._dio);

  /// プロジェクトタイプ一覧を取得する
  /// [activeOnly] がtrueの場合、有効なプロジェクトタイプのみを返す
  Future<List<ProjectType>> listProjectTypes({bool activeOnly = false}) async {
    // DioException をキャッチしてアプリ固有の例外に変換する（情報漏洩防止のため内部エラー詳細は隠蔽する）
    try {
      final response = await _dio.get(
        '/api/v1/taskmanagement/project-types',
        queryParameters: {'active_only': activeOnly},
      );
      final List<dynamic> data =
          (response.data as Map<String, dynamic>)['project_types'] as List<dynamic>;
      return data
          .map((json) => ProjectType.fromJson(json as Map<String, dynamic>))
          .toList();
    } on DioException catch (e) {
      throw AppException.fromDioException(e);
    } catch (e) {
      throw AppException.unknown(e.toString());
    }
  }

  /// 新規プロジェクトタイプを作成する
  /// 作成されたプロジェクトタイプをレスポンスから返す
  Future<ProjectType> createProjectType(CreateProjectTypeInput input) async {
    // DioException をキャッチしてアプリ固有の例外に変換する（情報漏洩防止のため内部エラー詳細は隠蔽する）
    try {
      final response = await _dio.post(
        '/api/v1/taskmanagement/project-types',
        data: input.toJson(),
      );
      return ProjectType.fromJson(response.data as Map<String, dynamic>);
    } on DioException catch (e) {
      throw AppException.fromDioException(e);
    } catch (e) {
      throw AppException.unknown(e.toString());
    }
  }

  /// 指定IDのプロジェクトタイプを取得する
  Future<ProjectType> getProjectType(String id) async {
    // DioException をキャッチしてアプリ固有の例外に変換する（情報漏洩防止のため内部エラー詳細は隠蔽する）
    try {
      final response = await _dio.get('/api/v1/taskmanagement/project-types/$id');
      return ProjectType.fromJson(response.data as Map<String, dynamic>);
    } on DioException catch (e) {
      throw AppException.fromDioException(e);
    } catch (e) {
      throw AppException.unknown(e.toString());
    }
  }

  /// 指定IDのプロジェクトタイプを更新する
  Future<ProjectType> updateProjectType(
    String id,
    UpdateProjectTypeInput input,
  ) async {
    // DioException をキャッチしてアプリ固有の例外に変換する（情報漏洩防止のため内部エラー詳細は隠蔽する）
    try {
      final response = await _dio.put(
        '/api/v1/taskmanagement/project-types/$id',
        data: input.toJson(),
      );
      return ProjectType.fromJson(response.data as Map<String, dynamic>);
    } on DioException catch (e) {
      throw AppException.fromDioException(e);
    } catch (e) {
      throw AppException.unknown(e.toString());
    }
  }

  /// 指定IDのプロジェクトタイプを削除する
  Future<void> deleteProjectType(String id) async {
    // DioException をキャッチしてアプリ固有の例外に変換する（情報漏洩防止のため内部エラー詳細は隠蔽する）
    try {
      await _dio.delete('/api/v1/taskmanagement/project-types/$id');
    } on DioException catch (e) {
      throw AppException.fromDioException(e);
    } catch (e) {
      throw AppException.unknown(e.toString());
    }
  }
}
