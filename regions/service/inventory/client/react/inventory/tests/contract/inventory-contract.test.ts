import { describe, it, expect } from 'vitest';
import {
  inventoryItemSchema,
  stockOperationSchema,
  updateStockSchema,
} from '../../src/types/inventory';
import { mockInventoryItems } from '../testutil/msw-setup';

// ============================================================
// 契約テスト: クライアント側の型定義がサーバー契約に準拠していることを検証する。
// モックデータ・スキーマの整合性を保証し、サーバー契約の変更時に
// クライアント側の不整合を早期検出する。
// ============================================================

describe('在庫契約テスト', () => {
  // ----------------------------------------------------------
  // モックデータがサーバー契約のスキーマに準拠していることを保証する。
  // MSWモックが正しい形式のレスポンスを返しているかの検証。
  // ----------------------------------------------------------
  describe('モックデータのスキーマ準拠検証', () => {
    // 全件のモックデータを個別にパースし、各レコードがスキーマに適合することを確認
    it.each(
      mockInventoryItems.map((item, i) => [i, item] as [number, (typeof mockInventoryItems)[number]])
    )('mockInventoryItems[%i] が inventoryItemSchema に準拠する', (_index, item) => {
      const result = inventoryItemSchema.safeParse(item);
      expect(result.success).toBe(true);
      if (!result.success) {
        // パースエラー時にデバッグ情報を出力する
        console.error('検証失敗:', result.error.format());
      }
    });

    // 全件を一括でパースし、配列全体としてスキーマに準拠することを確認
    it('mockInventoryItems 全件が inventoryItemSchema に準拠する', () => {
      for (const item of mockInventoryItems) {
        const result = inventoryItemSchema.safeParse(item);
        expect(result.success).toBe(true);
      }
    });

    // 全ての在庫ステータス値が含まれていることを確認（テストデータ網羅性検証）
    it('mockInventoryItems が全ステータスパターンを含む', () => {
      const statuses = new Set(mockInventoryItems.map((i) => i.status));
      expect(statuses.has('in_stock')).toBe(true);
      expect(statuses.has('low_stock')).toBe(true);
      expect(statuses.has('out_of_stock')).toBe(true);
    });
  });

  // ----------------------------------------------------------
  // 在庫操作リクエストの正常系テスト（予約・解放共通）。
  // ----------------------------------------------------------
  describe('stockOperationSchema 正常系', () => {
    // 最小有効量（1）の在庫操作が受け入れられることを確認（境界値テスト）
    it('quantity=1 の最小有効量で通る', () => {
      const validInput = {
        product_id: 'PROD-001',
        warehouse_id: 'WH-001',
        quantity: 1,
      };
      const result = stockOperationSchema.safeParse(validInput);
      expect(result.success).toBe(true);
    });

    // 大量数の在庫操作が受け入れられることを確認
    it('大量数量の在庫操作が通る', () => {
      const validInput = {
        product_id: 'PROD-001',
        warehouse_id: 'WH-001',
        quantity: 9999,
      };
      const result = stockOperationSchema.safeParse(validInput);
      expect(result.success).toBe(true);
    });
  });

  // ----------------------------------------------------------
  // 在庫操作リクエストの異常系テスト。
  // ----------------------------------------------------------
  describe('stockOperationSchema 異常系', () => {
    // 数量0は min(1) 制約により拒否されること（境界値テスト）
    it('quantity=0 が拒否される', () => {
      const invalidInput = {
        product_id: 'PROD-001',
        warehouse_id: 'WH-001',
        quantity: 0,
      };
      const result = stockOperationSchema.safeParse(invalidInput);
      expect(result.success).toBe(false);
    });

    // 負の数量は min(1) 制約により拒否されること
    it('負の数量が拒否される', () => {
      const invalidInput = {
        product_id: 'PROD-001',
        warehouse_id: 'WH-001',
        quantity: -10,
      };
      const result = stockOperationSchema.safeParse(invalidInput);
      expect(result.success).toBe(false);
    });

    // 空の product_id は min(1) 制約により拒否されること
    it('空の product_id が拒否される', () => {
      const invalidInput = {
        product_id: '',
        warehouse_id: 'WH-001',
        quantity: 10,
      };
      const result = stockOperationSchema.safeParse(invalidInput);
      expect(result.success).toBe(false);
    });

    // 空の warehouse_id は min(1) 制約により拒否されること
    it('空の warehouse_id が拒否される', () => {
      const invalidInput = {
        product_id: 'PROD-001',
        warehouse_id: '',
        quantity: 10,
      };
      const result = stockOperationSchema.safeParse(invalidInput);
      expect(result.success).toBe(false);
    });

    // product_id が欠落している場合に拒否されること
    it('product_id が欠落している場合に拒否される', () => {
      const invalidInput = {
        warehouse_id: 'WH-001',
        quantity: 10,
      };
      const result = stockOperationSchema.safeParse(invalidInput);
      expect(result.success).toBe(false);
    });
  });

  // ----------------------------------------------------------
  // 在庫更新リクエストの正常系テスト。
  // ----------------------------------------------------------
  describe('updateStockSchema 正常系', () => {
    // 必須フィールドのみで更新リクエストが通ることを確認
    it('quantity_available のみで通る', () => {
      const result = updateStockSchema.safeParse({ quantity_available: 100 });
      expect(result.success).toBe(true);
    });

    // 全フィールド指定で更新リクエストが通ることを確認
    it('全フィールド指定で通る', () => {
      const result = updateStockSchema.safeParse({
        quantity_available: 100,
        reorder_point: 20,
      });
      expect(result.success).toBe(true);
    });

    // quantity_available=0 が受け入れられることを確認（在庫切れの正常状態）
    it('quantity_available=0 が受け入れられる', () => {
      const result = updateStockSchema.safeParse({ quantity_available: 0 });
      expect(result.success).toBe(true);
    });
  });

  // ----------------------------------------------------------
  // 在庫更新リクエストの異常系テスト。
  // ----------------------------------------------------------
  describe('updateStockSchema 異常系', () => {
    // 負の在庫数は min(0) 制約により拒否されること
    it('負の quantity_available が拒否される', () => {
      const result = updateStockSchema.safeParse({ quantity_available: -1 });
      expect(result.success).toBe(false);
    });

    // 負の再注文点は min(0) 制約により拒否されること
    it('負の reorder_point が拒否される', () => {
      const result = updateStockSchema.safeParse({
        quantity_available: 10,
        reorder_point: -1,
      });
      expect(result.success).toBe(false);
    });

    // quantity_available が欠落している場合に拒否されること
    it('quantity_available が欠落している場合に拒否される', () => {
      const result = updateStockSchema.safeParse({ reorder_point: 10 });
      expect(result.success).toBe(false);
    });
  });

  // ----------------------------------------------------------
  // inventoryItemSchema の必須フィールド検証。
  // ----------------------------------------------------------
  describe('inventoryItemSchema 必須フィールド検証', () => {
    // 完全なデータを基準とし、各フィールドの欠落テストに使用する
    const validItem = {
      id: '550e8400-e29b-41d4-a716-446655440099',
      product_id: 'PROD-099',
      product_name: 'テスト商品',
      warehouse_id: 'WH-099',
      warehouse_name: 'テスト倉庫',
      quantity_available: 100,
      quantity_reserved: 10,
      reorder_point: 20,
      status: 'in_stock',
      version: 1,
      created_at: '2024-01-20T00:00:00Z',
      updated_at: '2024-01-20T00:00:00Z',
    };

    // 完全なデータがバリデーションを通過することを確認（基準テスト）
    it('完全なデータが inventoryItemSchema に準拠する', () => {
      const result = inventoryItemSchema.safeParse(validItem);
      expect(result.success).toBe(true);
    });

    // id が UUID 形式でない場合に拒否されること
    it('id が UUID 形式でない場合に拒否される', () => {
      const result = inventoryItemSchema.safeParse({ ...validItem, id: 'not-a-uuid' });
      expect(result.success).toBe(false);
    });

    // 不正なステータス値を持つアイテムが拒否されること
    it('不正な status を持つアイテムが拒否される', () => {
      const result = inventoryItemSchema.safeParse({ ...validItem, status: 'unknown' });
      expect(result.success).toBe(false);
    });

    // 負の利用可能数は min(0) 制約により拒否されること
    it('負の quantity_available が拒否される', () => {
      const result = inventoryItemSchema.safeParse({ ...validItem, quantity_available: -1 });
      expect(result.success).toBe(false);
    });

    // version フィールドが必須であることを確認（楽観的ロック制御に必要）
    it('version が欠落した場合に拒否される', () => {
      const { version: _, ...withoutVersion } = validItem;
      const result = inventoryItemSchema.safeParse(withoutVersion);
      expect(result.success).toBe(false);
    });
  });
});
