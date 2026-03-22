import 'package:dio/dio.dart';

import '../models/board_column.dart';

/// ボードAPIのリポジトリ層
/// サーバーとの通信を担当し、ボードカラムデータの永続化・取得を行う
class BoardRepository {
  /// APIリクエストに使用するDioインスタンス
  final Dio _dio;

  /// コンストラクタ: Dioインスタンスを外部から注入する（テスタビリティのため）
  BoardRepository(this._dio);

  // ========================================
  // カラム操作
  // ========================================

  /// 指定プロジェクトのカラム一覧を取得する
  Future<List<BoardColumn>> listColumns(String projectId) async {
    final response = await _dio.get('/api/v1/boards/$projectId/columns');
    final List<dynamic> data =
        (response.data as Map<String, dynamic>)['columns'] as List<dynamic>;
    return data
        .map((json) => BoardColumn.fromJson(json as Map<String, dynamic>))
        .toList();
  }

  /// 指定プロジェクト・ステータスコードのカラムを取得する
  Future<BoardColumn> getColumn(String projectId, String statusCode) async {
    final response = await _dio.get(
      '/api/v1/boards/$projectId/columns/$statusCode',
    );
    return BoardColumn.fromJson(response.data as Map<String, dynamic>);
  }

  /// カラムのタスク数をインクリメントする
  Future<BoardColumn> incrementColumn(IncrementColumnInput input) async {
    final response = await _dio.post(
      '/api/v1/boards/increment',
      data: input.toJson(),
    );
    return BoardColumn.fromJson(response.data as Map<String, dynamic>);
  }

  /// カラムのタスク数をデクリメントする
  Future<BoardColumn> decrementColumn(DecrementColumnInput input) async {
    final response = await _dio.post(
      '/api/v1/boards/decrement',
      data: input.toJson(),
    );
    return BoardColumn.fromJson(response.data as Map<String, dynamic>);
  }

  /// カラムのWIP制限を更新する
  Future<BoardColumn> updateWipLimit(
    String projectId,
    String statusCode,
    UpdateWipLimitInput input,
  ) async {
    final response = await _dio.put(
      '/api/v1/boards/$projectId/columns/$statusCode/wip-limit',
      data: input.toJson(),
    );
    return BoardColumn.fromJson(response.data as Map<String, dynamic>);
  }
}
