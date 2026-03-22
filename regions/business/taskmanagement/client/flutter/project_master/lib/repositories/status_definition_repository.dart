import 'package:dio/dio.dart';

import '../models/status_definition.dart';

/// ステータス定義APIのリポジトリ層
/// サーバーとの通信を担当し、ステータス定義データの永続化・取得を行う
class StatusDefinitionRepository {
  /// APIリクエストに使用するDioインスタンス
  final Dio _dio;

  /// コンストラクタ: Dioインスタンスを外部から注入する（テスタビリティのため）
  StatusDefinitionRepository(this._dio);

  /// 指定プロジェクトタイプのステータス定義一覧を取得する
  Future<List<StatusDefinition>> listStatusDefinitions(String projectTypeId) async {
    final response = await _dio.get(
      '/api/v1/taskmanagement/project-types/$projectTypeId/status-definitions',
    );
    final List<dynamic> data =
        (response.data as Map<String, dynamic>)['status_definitions'] as List<dynamic>;
    return data
        .map((json) => StatusDefinition.fromJson(json as Map<String, dynamic>))
        .toList();
  }

  /// 指定プロジェクトタイプに新規ステータス定義を作成する
  Future<StatusDefinition> createStatusDefinition(
    String projectTypeId,
    CreateStatusDefinitionInput input,
  ) async {
    final response = await _dio.post(
      '/api/v1/taskmanagement/project-types/$projectTypeId/status-definitions',
      data: input.toJson(),
    );
    return StatusDefinition.fromJson(response.data as Map<String, dynamic>);
  }

  /// 指定ステータス定義を更新する
  Future<StatusDefinition> updateStatusDefinition(
    String projectTypeId,
    String id,
    UpdateStatusDefinitionInput input,
  ) async {
    final response = await _dio.put(
      '/api/v1/taskmanagement/project-types/$projectTypeId/status-definitions/$id',
      data: input.toJson(),
    );
    return StatusDefinition.fromJson(response.data as Map<String, dynamic>);
  }

  /// 指定ステータス定義を削除する
  Future<void> deleteStatusDefinition(String projectTypeId, String id) async {
    await _dio.delete(
      '/api/v1/taskmanagement/project-types/$projectTypeId/status-definitions/$id',
    );
  }

  /// 指定ステータス定義のバージョン履歴を取得する
  /// 変更の監査証跡として使用する
  Future<List<StatusDefinitionVersion>> listVersions(String statusDefinitionId) async {
    final response = await _dio.get(
      '/api/v1/taskmanagement/status-definitions/$statusDefinitionId/versions',
    );
    final List<dynamic> data =
        (response.data as Map<String, dynamic>)['versions'] as List<dynamic>;
    return data
        .map((json) =>
            StatusDefinitionVersion.fromJson(json as Map<String, dynamic>))
        .toList();
  }
}
