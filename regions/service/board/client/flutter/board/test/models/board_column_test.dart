/// board_column_test.dart: BoardColumnモデルのユニットテスト。
/// fromJson/toJsonの往復変換、WIPゲージ計算、超過判定を検証する。
import 'package:flutter_test/flutter_test.dart';
import 'package:board/models/board_column.dart';

/// テスト用の基本的なBoardColumn JSONデータ
Map<String, dynamic> _sampleColumnJson({
  String id = '550e8400-e29b-41d4-a716-446655440001',
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

void main() {
  group('BoardColumn.fromJson', () {
    /// 完全なJSONデータが正しくBoardColumnインスタンスに変換されることを確認する
    test('必須フィールドが正しくパースされる', () {
      final column = BoardColumn.fromJson(_sampleColumnJson());

      expect(column.id, '550e8400-e29b-41d4-a716-446655440001');
      expect(column.projectId, 'PROJECT-001');
      expect(column.statusCode, 'todo');
      expect(column.wipLimit, 5);
      expect(column.taskCount, 3);
      expect(column.version, 1);
    });

    /// createdAt と updatedAt が DateTime に変換されることを確認する
    test('日時フィールドが DateTime に変換される', () {
      final column = BoardColumn.fromJson(_sampleColumnJson());
      expect(column.createdAt, isA<DateTime>());
      expect(column.updatedAt, isA<DateTime>());
    });
  });

  group('BoardColumn.toJson', () {
    /// fromJson して toJson した結果が元のデータと一致することを確認する（往復変換）
    test('fromJson → toJson で往復変換が成立する', () {
      final original = _sampleColumnJson();
      final column = BoardColumn.fromJson(original);
      final json = column.toJson();

      expect(json['id'], original['id']);
      expect(json['project_id'], original['project_id']);
      expect(json['status_code'], original['status_code']);
      expect(json['wip_limit'], original['wip_limit']);
      expect(json['task_count'], original['task_count']);
      expect(json['version'], original['version']);
    });
  });

  group('BoardColumn.wipUsageRatio', () {
    /// WIP使用率が正しく計算されることを確認する
    test('WIP使用率が正しく計算される', () {
      final column = BoardColumn.fromJson(_sampleColumnJson(wipLimit: 5, taskCount: 3));
      expect(column.wipUsageRatio, closeTo(0.6, 0.001));
    });

    /// WIP制限が0の場合は0.0を返すことを確認する（無制限）
    test('WIP制限が0の場合は0.0を返す（無制限）', () {
      final column = BoardColumn.fromJson(_sampleColumnJson(wipLimit: 0, taskCount: 10));
      expect(column.wipUsageRatio, 0.0);
    });

    /// WIP使用率が1.0を超えない（クランプ）ことを確認する
    test('WIP使用率は1.0を超えない（超過時は1.0にクランプ）', () {
      final column = BoardColumn.fromJson(_sampleColumnJson(wipLimit: 3, taskCount: 5));
      expect(column.wipUsageRatio, 1.0);
    });
  });

  group('BoardColumn.isOverWipLimit', () {
    /// WIP制限に達した場合にtrueを返すことを確認する
    test('タスク数がWIP制限以上の場合にtrueを返す', () {
      final column = BoardColumn.fromJson(_sampleColumnJson(wipLimit: 3, taskCount: 3));
      expect(column.isOverWipLimit, isTrue);
    });

    /// WIP制限未満の場合にfalseを返すことを確認する
    test('タスク数がWIP制限未満の場合にfalseを返す', () {
      final column = BoardColumn.fromJson(_sampleColumnJson(wipLimit: 5, taskCount: 2));
      expect(column.isOverWipLimit, isFalse);
    });

    /// WIP制限が0（無制限）の場合にfalseを返すことを確認する
    test('WIP制限が0（無制限）の場合にfalseを返す', () {
      final column = BoardColumn.fromJson(_sampleColumnJson(wipLimit: 0, taskCount: 100));
      expect(column.isOverWipLimit, isFalse);
    });
  });

  group('IncrementColumnInput.toJson', () {
    /// 入力データが正しく JSON に変換されることを確認する
    test('入力データが正しく toJson に変換される', () {
      const input = IncrementColumnInput(
        projectId: 'PROJECT-001',
        statusCode: 'todo',
      );
      final json = input.toJson();

      expect(json['project_id'], 'PROJECT-001');
      expect(json['status_code'], 'todo');
    });
  });

  group('DecrementColumnInput.toJson', () {
    /// 入力データが正しく JSON に変換されることを確認する
    test('入力データが正しく toJson に変換される', () {
      const input = DecrementColumnInput(
        projectId: 'PROJECT-001',
        statusCode: 'in_progress',
      );
      final json = input.toJson();

      expect(json['project_id'], 'PROJECT-001');
      expect(json['status_code'], 'in_progress');
    });
  });

  group('UpdateWipLimitInput.toJson', () {
    /// WIP制限更新データが正しく JSON に変換されることを確認する
    test('WIP制限更新データが正しく toJson に変換される', () {
      const input = UpdateWipLimitInput(wipLimit: 10);
      final json = input.toJson();

      expect(json['wip_limit'], 10);
    });

    /// WIP制限0（無制限）が正しく JSON に変換されることを確認する
    test('WIP制限0（無制限）が正しく toJson に変換される', () {
      const input = UpdateWipLimitInput(wipLimit: 0);
      final json = input.toJson();

      expect(json['wip_limit'], 0);
    });
  });
}
