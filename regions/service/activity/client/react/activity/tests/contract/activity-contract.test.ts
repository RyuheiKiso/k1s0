import { describe, it, expect } from 'vitest';
import {
  activitySchema,
  activityStatusSchema,
  activityTypeSchema,
  createActivityInputSchema,
  rejectActivityInputSchema,
} from '../../src/types/activity';
import { mockActivities } from '../testutil/msw-setup';

// ============================================================
// 契約テスト: クライアント側の型定義がサーバー契約に準拠していることを検証する。
// モックデータ・スキーマの整合性を保証し、サーバー契約の変更時に
// クライアント側の不整合を早期検出する。
// ============================================================

describe('アクティビティ契約テスト', () => {
  // ----------------------------------------------------------
  // モックデータがサーバー契約のスキーマに準拠していることを保証する。
  // MSWモックが正しい形式のレスポンスを返しているかの検証。
  // ----------------------------------------------------------
  describe('モックデータのスキーマ準拠検証', () => {
    // 全件のモックデータを個別にパースし、各レコードがスキーマに適合することを確認
    it.each(mockActivities.map((a, i) => [i, a] as [number, (typeof mockActivities)[number]]))(
      'mockActivities[%i] が activitySchema に準拠する',
      (_index, activity) => {
        const result = activitySchema.safeParse(activity);
        expect(result.success).toBe(true);
        if (!result.success) {
          // パースエラー時にデバッグ情報を出力する
          console.error('検証失敗:', result.error.format());
        }
      }
    );

    // 全件を一括でパースし、配列全体としてスキーマに準拠することを確認
    it('mockActivities 全件が activitySchema に準拠する', () => {
      for (const activity of mockActivities) {
        const result = activitySchema.safeParse(activity);
        expect(result.success).toBe(true);
      }
    });
  });

  // ----------------------------------------------------------
  // アクティビティ作成リクエストの正常系テスト。
  // 有効な入力がバリデーションを通過することを確認する。
  // ----------------------------------------------------------
  describe('createActivityInputSchema 正常系', () => {
    // 必須フィールドのみの最小限の入力が受け入れられることを確認
    it('必須フィールドのみで有効な入力が通る', () => {
      const validInput = {
        task_id: 'TASK-001',
        actor_id: 'USER-001',
        activity_type: 'comment',
      };
      const result = createActivityInputSchema.safeParse(validInput);
      expect(result.success).toBe(true);
    });

    // 全フィールドを含む入力が受け入れられることを確認
    it('全フィールド指定の入力が通る', () => {
      const validInput = {
        task_id: 'TASK-001',
        actor_id: 'USER-001',
        activity_type: 'time_entry',
        content: '作業内容の説明',
        duration_minutes: 90,
        idempotency_key: 'IDEM-KEY-001',
      };
      const result = createActivityInputSchema.safeParse(validInput);
      expect(result.success).toBe(true);
    });
  });

  // ----------------------------------------------------------
  // アクティビティ作成リクエストの異常系テスト。
  // 不正な入力がバリデーションで拒否されることを確認する。
  // ----------------------------------------------------------
  describe('createActivityInputSchema 異常系', () => {
    // 空の task_id は min(1) 制約により拒否されること
    it('空の task_id が拒否される', () => {
      const invalidInput = {
        task_id: '',
        actor_id: 'USER-001',
        activity_type: 'comment',
      };
      const result = createActivityInputSchema.safeParse(invalidInput);
      expect(result.success).toBe(false);
    });

    // 空の actor_id は min(1) 制約により拒否されること
    it('空の actor_id が拒否される', () => {
      const invalidInput = {
        task_id: 'TASK-001',
        actor_id: '',
        activity_type: 'comment',
      };
      const result = createActivityInputSchema.safeParse(invalidInput);
      expect(result.success).toBe(false);
    });

    // 無効な activity_type は拒否されること
    it('無効な activity_type が拒否される', () => {
      const invalidInput = {
        task_id: 'TASK-001',
        actor_id: 'USER-001',
        activity_type: 'invalid_type',
      };
      const result = createActivityInputSchema.safeParse(invalidInput);
      expect(result.success).toBe(false);
    });

    // 負の duration_minutes は min(0) 制約により拒否されること
    it('負の duration_minutes が拒否される', () => {
      const invalidInput = {
        task_id: 'TASK-001',
        actor_id: 'USER-001',
        activity_type: 'time_entry',
        duration_minutes: -1,
      };
      const result = createActivityInputSchema.safeParse(invalidInput);
      expect(result.success).toBe(false);
    });
  });

  // ----------------------------------------------------------
  // アクティビティステータスの契約検証。
  // ----------------------------------------------------------
  describe('activityStatusSchema の契約検証', () => {
    // サーバー契約で定義された全ての有効なステータス値が受け入れられること
    it.each(['active', 'submitted', 'approved', 'rejected'] as const)(
      '有効なステータス "%s" が受け入れられる',
      (status) => {
        const result = activityStatusSchema.safeParse(status);
        expect(result.success).toBe(true);
      }
    );

    // 定義外のステータス値が拒否されること
    it.each(['pending', 'cancelled', 'draft'])(
      '無効なステータス "%s" が拒否される',
      (invalidStatus) => {
        const result = activityStatusSchema.safeParse(invalidStatus);
        expect(result.success).toBe(false);
      }
    );

    // 空文字列がステータスとして拒否されること
    it('空文字列がステータスとして拒否される', () => {
      const result = activityStatusSchema.safeParse('');
      expect(result.success).toBe(false);
    });
  });

  // ----------------------------------------------------------
  // アクティビティ種別の契約検証。
  // ----------------------------------------------------------
  describe('activityTypeSchema の契約検証', () => {
    // 全ての有効な種別値が受け入れられること
    it.each(['comment', 'time_entry', 'status_change', 'assignment'] as const)(
      '有効な種別 "%s" が受け入れられる',
      (activityType) => {
        const result = activityTypeSchema.safeParse(activityType);
        expect(result.success).toBe(true);
      }
    );

    // 無効な種別値が拒否されること
    it('無効な種別値が拒否される', () => {
      const result = activityTypeSchema.safeParse('invalid_type');
      expect(result.success).toBe(false);
    });
  });

  // ----------------------------------------------------------
  // 却下入力スキーマの契約検証。
  // ----------------------------------------------------------
  describe('rejectActivityInputSchema の契約検証', () => {
    // 空オブジェクト（理由なし）が受け入れられること
    it('空オブジェクトが受け入れられる', () => {
      const result = rejectActivityInputSchema.safeParse({});
      expect(result.success).toBe(true);
    });

    // reason フィールドありの入力が受け入れられること
    it('reason フィールドありの入力が通る', () => {
      const result = rejectActivityInputSchema.safeParse({ reason: '内容が不十分です' });
      expect(result.success).toBe(true);
    });
  });

  // ----------------------------------------------------------
  // activitySchema の必須フィールド検証。
  // ----------------------------------------------------------
  describe('activitySchema 必須フィールド検証', () => {
    // 完全なデータを基準とし、各フィールドの欠落テストに使用する
    const validActivity = {
      id: '550e8400-e29b-41d4-a716-446655440099',
      task_id: 'TASK-099',
      actor_id: 'USER-099',
      activity_type: 'comment',
      content: 'テストコメント',
      duration_minutes: null,
      status: 'active',
      metadata: null,
      idempotency_key: null,
      version: 1,
      created_at: '2024-01-20T00:00:00Z',
      updated_at: '2024-01-20T00:00:00Z',
    };

    // 完全なデータがバリデーションを通過することを確認（基準テスト）
    it('完全なデータが activitySchema に準拠する', () => {
      const result = activitySchema.safeParse(validActivity);
      expect(result.success).toBe(true);
    });

    // id が UUID 形式でない場合に拒否されること
    it('id が UUID 形式でない場合に拒否される', () => {
      const result = activitySchema.safeParse({ ...validActivity, id: 'not-a-uuid' });
      expect(result.success).toBe(false);
    });

    // version フィールドが必須であることを確認（楽観的ロック制御に必要）
    it('version が欠落した場合に拒否される', () => {
      const { version: _, ...withoutVersion } = validActivity;
      const result = activitySchema.safeParse(withoutVersion);
      expect(result.success).toBe(false);
    });

    // 不正なステータス値を持つアクティビティが拒否されること
    it('不正な status を持つアクティビティが拒否される', () => {
      const result = activitySchema.safeParse({ ...validActivity, status: 'unknown_status' });
      expect(result.success).toBe(false);
    });
  });
});
