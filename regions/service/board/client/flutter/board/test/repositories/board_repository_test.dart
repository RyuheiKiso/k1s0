/// board_repository_test.dart: BoardRepositoryのユニットテスト。
/// MockHttpClientAdapterを使用してAPI通信をモックし、
/// 各メソッドが正しくリクエストを送信・レスポンスを処理することを検証する。
library;
import 'dart:convert';

import 'package:dio/dio.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:board/models/board_column.dart';
import 'package:board/repositories/board_repository.dart';

/// テスト用のHTTPクライアントアダプター
class _MockHttpClientAdapter implements HttpClientAdapter {
  final String Function(RequestOptions) responseBodyFn;
  final int statusCode;

  _MockHttpClientAdapter({
    required this.responseBodyFn,
  });

  @override
  Future<ResponseBody> fetch(
    RequestOptions options,
    Stream<List<int>>? requestStream,
    Future<void>? cancelFuture,
  ) async {
    return ResponseBody.fromString(
      responseBodyFn(options),
      statusCode,
      headers: {
        'content-type': ['application/json'],
      },
    );
  }

  @override
  void close({bool force = false}) {}
}

/// テスト用のサンプルBoardColumn JSONデータを生成する
Map<String, dynamic> _sampleColumnData({
  String id = 'COL-001',
  String projectId = 'PROJECT-001',
  String statusCode = 'todo',
  int wipLimit = 5,
  int taskCount = 3,
}) =>
    {
      'id': id,
      'project_id': projectId,
      'status_code': statusCode,
      'wip_limit': wipLimit,
      'task_count': taskCount,
      'version': 1,
      'created_at': '2024-01-20T00:00:00.000Z',
      'updated_at': '2024-01-20T00:00:00.000Z',
    };

/// テスト用のDioインスタンスを生成する
Dio _createTestDio(String Function(RequestOptions) responseBodyFn) {
  final dio = Dio(BaseOptions(baseUrl: 'http://localhost:8080'));
  dio.httpClientAdapter = _MockHttpClientAdapter(responseBodyFn: responseBodyFn);
  return dio;
}

void main() {
  group('BoardRepository.listColumns', () {
    /// カラム一覧が正しく取得されることを確認する
    test('カラム一覧が正しく取得される', () async {
      final dio = _createTestDio((_) => jsonEncode({
            'columns': [
              _sampleColumnData(id: 'COL-001', statusCode: 'todo'),
              _sampleColumnData(id: 'COL-002', statusCode: 'in_progress'),
            ]
          }));
      final repo = BoardRepository(dio);

      final columns = await repo.listColumns('PROJECT-001');

      expect(columns, hasLength(2));
      expect(columns[0].id, 'COL-001');
      expect(columns[1].id, 'COL-002');
      expect(columns[1].statusCode, 'in_progress');
    });

    /// 空のカラム一覧が返されたときに空リストになることを確認する
    test('空のカラム一覧が返される', () async {
      final dio = _createTestDio((_) => jsonEncode({'columns': []}));
      final repo = BoardRepository(dio);

      final columns = await repo.listColumns('PROJECT-001');

      expect(columns, isEmpty);
    });
  });

  group('BoardRepository.getColumn', () {
    /// 指定ステータスコードのカラムが正しく取得されることを確認する
    test('指定ステータスコードのカラムが正しく取得される', () async {
      final dio = _createTestDio(
        (_) => jsonEncode(_sampleColumnData(id: 'COL-001', statusCode: 'todo')),
      );
      final repo = BoardRepository(dio);

      final column = await repo.getColumn('PROJECT-001', 'todo');

      expect(column.id, 'COL-001');
      expect(column.statusCode, 'todo');
    });
  });

  group('BoardRepository.incrementColumn', () {
    /// タスク数インクリメントが正しく動作することを確認する
    test('タスク数がインクリメントされる', () async {
      final dio = _createTestDio(
        (_) => jsonEncode(_sampleColumnData(taskCount: 4)),
      );
      final repo = BoardRepository(dio);

      const input = IncrementColumnInput(
        projectId: 'PROJECT-001',
        statusCode: 'todo',
      );
      final column = await repo.incrementColumn(input);

      expect(column.taskCount, 4);
    });
  });

  group('BoardRepository.decrementColumn', () {
    /// タスク数デクリメントが正しく動作することを確認する
    test('タスク数がデクリメントされる', () async {
      final dio = _createTestDio(
        (_) => jsonEncode(_sampleColumnData(taskCount: 2)),
      );
      final repo = BoardRepository(dio);

      const input = DecrementColumnInput(
        projectId: 'PROJECT-001',
        statusCode: 'todo',
      );
      final column = await repo.decrementColumn(input);

      expect(column.taskCount, 2);
    });
  });

  group('BoardRepository.updateWipLimit', () {
    /// WIP制限更新が正しく動作することを確認する
    test('WIP制限が正しく更新される', () async {
      final dio = _createTestDio(
        (_) => jsonEncode(_sampleColumnData(wipLimit: 10)),
      );
      final repo = BoardRepository(dio);

      const input = UpdateWipLimitInput(wipLimit: 10);
      final column = await repo.updateWipLimit('PROJECT-001', 'todo', input);

      expect(column.wipLimit, 10);
    });

    /// WIP制限0（無制限）への更新が正しく動作することを確認する
    test('WIP制限0（無制限）への更新が正しく動作する', () async {
      final dio = _createTestDio(
        (_) => jsonEncode(_sampleColumnData(wipLimit: 0)),
      );
      final repo = BoardRepository(dio);

      const input = UpdateWipLimitInput(wipLimit: 0);
      final column = await repo.updateWipLimit('PROJECT-001', 'todo', input);

      expect(column.wipLimit, 0);
    });
  });
}
