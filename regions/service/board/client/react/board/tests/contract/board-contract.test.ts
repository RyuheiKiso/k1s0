import { describe, it, expect } from 'vitest';
import {
  boardColumnSchema,
  incrementColumnInputSchema,
  decrementColumnInputSchema,
  updateWipLimitInputSchema,
} from '../../src/types/board';
import { mockColumns } from '../testutil/msw-setup';

// ============================================================
// 契約テスト: クライアント側の型定義がサーバー契約に準拠していることを検証する。
// モックデータ・スキーマの整合性を保証し、サーバー契約の変更時に
// クライアント側の不整合を早期検出する。
// ============================================================

describe('ボード契約テスト', () => {
  // ----------------------------------------------------------
  // モックデータがサーバー契約のスキーマに準拠していることを保証する。
  // MSWモックが正しい形式のレスポンスを返しているかの検証。
  // ----------------------------------------------------------
  describe('モックデータのスキーマ準拠検証', () => {
    // 全件のモックデータを個別にパースし、各レコードがスキーマに適合することを確認
    it.each(mockColumns.map((c, i) => [i, c] as [number, (typeof mockColumns)[number]]))(
      'mockColumns[%i] が boardColumnSchema に準拠する',
      (_index, column) => {
        const result = boardColumnSchema.safeParse(column);
        expect(result.success).toBe(true);
        if (!result.success) {
          // パースエラー時にデバッグ情報を出力する
          console.error('検証失敗:', result.error.format());
        }
      }
    );

    // 全件を一括でパースし、配列全体としてスキーマに準拠することを確認
    it('mockColumns 全件が boardColumnSchema に準拠する', () => {
      for (const column of mockColumns) {
        const result = boardColumnSchema.safeParse(column);
        expect(result.success).toBe(true);
      }
    });
  });

  // ----------------------------------------------------------
  // インクリメント入力スキーマの正常系テスト。
  // ----------------------------------------------------------
  describe('incrementColumnInputSchema 正常系', () => {
    // 有効な入力が受け入れられることを確認
    it('有効な入力が通る', () => {
      const result = incrementColumnInputSchema.safeParse({
        project_id: 'PROJECT-001',
        status_code: 'todo',
      });
      expect(result.success).toBe(true);
    });
  });

  // ----------------------------------------------------------
  // インクリメント入力スキーマの異常系テスト。
  // ----------------------------------------------------------
  describe('incrementColumnInputSchema 異常系', () => {
    // 空のproject_idが拒否されることを確認
    it('空の project_id が拒否される', () => {
      const result = incrementColumnInputSchema.safeParse({
        project_id: '',
        status_code: 'todo',
      });
      expect(result.success).toBe(false);
    });

    // 空のstatus_codeが拒否されることを確認
    it('空の status_code が拒否される', () => {
      const result = incrementColumnInputSchema.safeParse({
        project_id: 'PROJECT-001',
        status_code: '',
      });
      expect(result.success).toBe(false);
    });
  });

  // ----------------------------------------------------------
  // デクリメント入力スキーマの正常系テスト。
  // ----------------------------------------------------------
  describe('decrementColumnInputSchema 正常系', () => {
    it('有効な入力が通る', () => {
      const result = decrementColumnInputSchema.safeParse({
        project_id: 'PROJECT-001',
        status_code: 'in_progress',
      });
      expect(result.success).toBe(true);
    });
  });

  // ----------------------------------------------------------
  // WIP制限更新スキーマの正常系テスト。
  // ----------------------------------------------------------
  describe('updateWipLimitInputSchema 正常系', () => {
    // 0（無制限）が受け入れられることを確認
    it('wip_limit 0（無制限）が受け入れられる', () => {
      const result = updateWipLimitInputSchema.safeParse({ wip_limit: 0 });
      expect(result.success).toBe(true);
    });

    // 正の整数が受け入れられることを確認
    it('正の整数の wip_limit が受け入れられる', () => {
      const result = updateWipLimitInputSchema.safeParse({ wip_limit: 10 });
      expect(result.success).toBe(true);
    });
  });

  // ----------------------------------------------------------
  // WIP制限更新スキーマの異常系テスト。
  // ----------------------------------------------------------
  describe('updateWipLimitInputSchema 異常系', () => {
    // 負のWIP制限が拒否されることを確認
    it('負の wip_limit が拒否される', () => {
      const result = updateWipLimitInputSchema.safeParse({ wip_limit: -1 });
      expect(result.success).toBe(false);
    });

    // wip_limitが未指定の場合に拒否されることを確認
    it('wip_limit が未指定の場合に拒否される', () => {
      const result = updateWipLimitInputSchema.safeParse({});
      expect(result.success).toBe(false);
    });
  });

  // ----------------------------------------------------------
  // boardColumnSchema の必須フィールド検証。
  // ----------------------------------------------------------
  describe('boardColumnSchema 必須フィールド検証', () => {
    // 完全なデータを基準とし、各フィールドの欠落テストに使用する
    const validColumn = {
      id: '550e8400-e29b-41d4-a716-446655440099',
      project_id: 'PROJECT-099',
      status_code: 'todo',
      wip_limit: 5,
      task_count: 2,
      version: 1,
      created_at: '2024-01-20T00:00:00Z',
      updated_at: '2024-01-20T00:00:00Z',
    };

    // 完全なデータがバリデーションを通過することを確認（基準テスト）
    it('完全なデータが boardColumnSchema に準拠する', () => {
      const result = boardColumnSchema.safeParse(validColumn);
      expect(result.success).toBe(true);
    });

    // id が UUID 形式でない場合に拒否されることを確認
    it('id が UUID 形式でない場合に拒否される', () => {
      const result = boardColumnSchema.safeParse({ ...validColumn, id: 'not-a-uuid' });
      expect(result.success).toBe(false);
    });

    // 負のwip_limitが拒否されることを確認
    it('負の wip_limit が拒否される', () => {
      const result = boardColumnSchema.safeParse({ ...validColumn, wip_limit: -1 });
      expect(result.success).toBe(false);
    });

    // 負のtask_countが拒否されることを確認
    it('負の task_count が拒否される', () => {
      const result = boardColumnSchema.safeParse({ ...validColumn, task_count: -1 });
      expect(result.success).toBe(false);
    });

    // version が欠落した場合に拒否されることを確認
    it('version が欠落した場合に拒否される', () => {
      const { version: _, ...withoutVersion } = validColumn;
      const result = boardColumnSchema.safeParse(withoutVersion);
      expect(result.success).toBe(false);
    });
  });
});
