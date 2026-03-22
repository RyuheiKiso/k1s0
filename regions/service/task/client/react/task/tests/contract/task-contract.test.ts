import { describe, it, expect } from 'vitest';
import {
  taskSchema,
  taskStatusSchema,
  taskPrioritySchema,
  createTaskInputSchema,
  updateTaskStatusInputSchema,
} from '../../src/types/task';
import { mockTasks } from '../testutil/msw-setup';

// ============================================================
// 契約テスト: クライアント側の型定義がサーバー契約に準拠していることを検証する。
// モックデータ・スキーマの整合性を保証し、サーバー契約の変更時に
// クライアント側の不整合を早期検出する。
// ============================================================

describe('タスク契約テスト', () => {
  // ----------------------------------------------------------
  // モックデータがサーバー契約のスキーマに準拠していることを保証する。
  // MSWモックが正しい形式のレスポンスを返しているかの検証。
  // ----------------------------------------------------------
  describe('モックデータのスキーマ準拠検証', () => {
    // 全件のモックデータを個別にパースし、各レコードがスキーマに適合することを確認
    it.each(mockTasks.map((t, i) => [i, t] as [number, (typeof mockTasks)[number]]))(
      'mockTasks[%i] が taskSchema に準拠する',
      (_index, task) => {
        const result = taskSchema.safeParse(task);
        expect(result.success).toBe(true);
        if (!result.success) {
          // パースエラー時にデバッグ情報を出力する
          console.error('検証失敗:', result.error.format());
        }
      }
    );

    // 全件を一括でパースし、配列全体としてスキーマに準拠することを確認
    it('mockTasks 全件が taskSchema に準拠する', () => {
      for (const task of mockTasks) {
        const result = taskSchema.safeParse(task);
        expect(result.success).toBe(true);
      }
    });
  });

  // ----------------------------------------------------------
  // タスク作成リクエストの正常系テスト。
  // 有効な入力がバリデーションを通過することを確認する。
  // ----------------------------------------------------------
  describe('createTaskInputSchema 正常系', () => {
    // 必須フィールドのみの最小限の入力が受け入れられることを確認
    it('必須フィールドのみで有効な入力が通る', () => {
      const validInput = {
        project_id: 'PROJ-001',
        title: 'テストタスク',
      };
      const result = createTaskInputSchema.safeParse(validInput);
      expect(result.success).toBe(true);
    });

    // オプションフィールドを含む完全な入力が受け入れられることを確認
    it('全フィールド指定の入力が通る', () => {
      const validInput = {
        project_id: 'PROJ-001',
        title: 'テストタスク',
        description: '詳細説明',
        priority: 'high',
        assignee_id: 'USER-001',
        due_date: '2026-04-01',
        labels: ['bug', 'frontend'],
      };
      const result = createTaskInputSchema.safeParse(validInput);
      expect(result.success).toBe(true);
    });
  });

  // ----------------------------------------------------------
  // タスク作成リクエストの異常系テスト。
  // 不正な入力がバリデーションで拒否されることを確認する。
  // ----------------------------------------------------------
  describe('createTaskInputSchema 異常系', () => {
    // 空の project_id は min(1) 制約により拒否されること
    it('空の project_id が拒否される', () => {
      const invalidInput = {
        project_id: '',
        title: 'テストタスク',
      };
      const result = createTaskInputSchema.safeParse(invalidInput);
      expect(result.success).toBe(false);
    });

    // 空のタイトルは min(1) 制約により拒否されること
    it('空の title が拒否される', () => {
      const invalidInput = {
        project_id: 'PROJ-001',
        title: '',
      };
      const result = createTaskInputSchema.safeParse(invalidInput);
      expect(result.success).toBe(false);
    });

    // project_id が未指定の場合に拒否されること
    it('project_id が未指定の場合に拒否される', () => {
      const invalidInput = {
        title: 'テストタスク',
      };
      const result = createTaskInputSchema.safeParse(invalidInput);
      expect(result.success).toBe(false);
    });
  });

  // ----------------------------------------------------------
  // タスクステータス更新リクエストの正常系テスト。
  // ----------------------------------------------------------
  describe('updateTaskStatusInputSchema 正常系', () => {
    // 全ての有効なステータス値で更新リクエストが通ることを確認
    it.each(['open', 'in_progress', 'review', 'done', 'cancelled'] as const)(
      '有効なステータス "%s" で更新リクエストが通る',
      (status) => {
        const result = updateTaskStatusInputSchema.safeParse({ status });
        expect(result.success).toBe(true);
      }
    );
  });

  // ----------------------------------------------------------
  // タスクステータス更新リクエストの異常系テスト。
  // ----------------------------------------------------------
  describe('updateTaskStatusInputSchema 異常系', () => {
    // 無効なステータス値は拒否されること
    it('無効なステータス値が拒否される', () => {
      const result = updateTaskStatusInputSchema.safeParse({ status: 'invalid_status' });
      expect(result.success).toBe(false);
    });

    // statusが欠落している場合に拒否されること
    it('status が欠落している場合に拒否される', () => {
      const result = updateTaskStatusInputSchema.safeParse({});
      expect(result.success).toBe(false);
    });
  });

  // ----------------------------------------------------------
  // タスクステータスの契約検証。
  // サーバー側で定義された有効なステータス値のみが受け入れられることを確認する。
  // ----------------------------------------------------------
  describe('taskStatusSchema の契約検証', () => {
    // サーバー契約で定義された全ての有効なステータス値が受け入れられること
    it.each(['open', 'in_progress', 'review', 'done', 'cancelled'] as const)(
      '有効なステータス "%s" が受け入れられる',
      (status) => {
        const result = taskStatusSchema.safeParse(status);
        expect(result.success).toBe(true);
      }
    );

    // 定義外のステータス値が拒否されること
    it.each(['pending', 'confirmed', 'archived'])(
      '無効なステータス "%s" が拒否される',
      (invalidStatus) => {
        const result = taskStatusSchema.safeParse(invalidStatus);
        expect(result.success).toBe(false);
      }
    );

    // 空文字列がステータスとして拒否されること
    it('空文字列がステータスとして拒否される', () => {
      const result = taskStatusSchema.safeParse('');
      expect(result.success).toBe(false);
    });
  });

  // ----------------------------------------------------------
  // タスク優先度の契約検証。
  // ----------------------------------------------------------
  describe('taskPrioritySchema の契約検証', () => {
    // 全ての有効な優先度値が受け入れられること
    it.each(['low', 'medium', 'high', 'critical'] as const)(
      '有効な優先度 "%s" が受け入れられる',
      (priority) => {
        const result = taskPrioritySchema.safeParse(priority);
        expect(result.success).toBe(true);
      }
    );

    // 定義外の優先度値が拒否されること
    it('無効な優先度値が拒否される', () => {
      const result = taskPrioritySchema.safeParse('urgent');
      expect(result.success).toBe(false);
    });
  });

  // ----------------------------------------------------------
  // taskSchema の必須フィールド検証。
  // ----------------------------------------------------------
  describe('taskSchema 必須フィールド検証', () => {
    // 完全なデータを基準とし、各フィールドの欠落テストに使用する
    const validTask = {
      id: '550e8400-e29b-41d4-a716-446655440099',
      project_id: 'PROJ-099',
      title: 'テストタスク',
      description: null,
      status: 'open',
      priority: 'medium',
      assignee_id: null,
      reporter_id: 'USER-001',
      due_date: null,
      labels: [],
      created_by: 'USER-001',
      updated_by: 'USER-001',
      version: 1,
      created_at: '2026-01-20T00:00:00Z',
      updated_at: '2026-01-20T00:00:00Z',
    };

    // 完全なデータがバリデーションを通過することを確認（基準テスト）
    it('完全なデータが taskSchema に準拠する', () => {
      const result = taskSchema.safeParse(validTask);
      expect(result.success).toBe(true);
    });

    // id が UUID 形式でない場合に拒否されること
    it('id が UUID 形式でない場合に拒否される', () => {
      const result = taskSchema.safeParse({ ...validTask, id: 'not-a-uuid' });
      expect(result.success).toBe(false);
    });

    // version フィールドが必須であることを確認（楽観的ロック制御に必要）
    it('version が欠落した場合に拒否される', () => {
      const { version: _, ...withoutVersion } = validTask;
      const result = taskSchema.safeParse(withoutVersion);
      expect(result.success).toBe(false);
    });

    // 不正なステータス値を持つタスクが拒否されること
    it('不正な status を持つタスクが拒否される', () => {
      const result = taskSchema.safeParse({ ...validTask, status: 'unknown_status' });
      expect(result.success).toBe(false);
    });

    // 不正な優先度値を持つタスクが拒否されること
    it('不正な priority を持つタスクが拒否される', () => {
      const result = taskSchema.safeParse({ ...validTask, priority: 'unknown_priority' });
      expect(result.success).toBe(false);
    });
  });
});
