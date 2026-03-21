import { describe, it, expect } from 'vitest';
import {
  orderSchema,
  orderStatusSchema,
  createOrderInputSchema,
  updateOrderStatusInputSchema,
} from '../../src/types/order';
import { mockOrders } from '../testutil/msw-setup';

// ============================================================
// 契約テスト: クライアント側の型定義がサーバー契約に準拠していることを検証する。
// モックデータ・スキーマの整合性を保証し、サーバー契約の変更時に
// クライアント側の不整合を早期検出する。
// ============================================================

describe('注文契約テスト', () => {
  // ----------------------------------------------------------
  // モックデータがサーバー契約のスキーマに準拠していることを保証する。
  // MSWモックが正しい形式のレスポンスを返しているかの検証。
  // ----------------------------------------------------------
  describe('モックデータのスキーマ準拠検証', () => {
    // 全件のモックデータを個別にパースし、各レコードがスキーマに適合することを確認
    it.each(mockOrders.map((o, i) => [i, o] as [number, (typeof mockOrders)[number]]))(
      'mockOrders[%i] が orderSchema に準拠する',
      (_index, order) => {
        const result = orderSchema.safeParse(order);
        expect(result.success).toBe(true);
        if (!result.success) {
          // パースエラー時にデバッグ情報を出力する
          console.error('検証失敗:', result.error.format());
        }
      }
    );

    // 全件を一括でパースし、配列全体としてスキーマに準拠することを確認
    it('mockOrders 全件が orderSchema に準拠する', () => {
      for (const order of mockOrders) {
        const result = orderSchema.safeParse(order);
        expect(result.success).toBe(true);
      }
    });
  });

  // ----------------------------------------------------------
  // 注文作成リクエストの正常系テスト。
  // 有効な入力がバリデーションを通過することを確認する。
  // ----------------------------------------------------------
  describe('createOrderInputSchema 正常系', () => {
    // 必須フィールドのみの最小限の入力が受け入れられることを確認
    it('必須フィールドのみで有効な入力が通る', () => {
      const validInput = {
        customer_id: 'CUST-001',
        items: [
          {
            product_id: 'PROD-001',
            product_name: 'テスト商品',
            quantity: 1,
            unit_price: 1000,
          },
        ],
      };
      const result = createOrderInputSchema.safeParse(validInput);
      expect(result.success).toBe(true);
    });

    // オプションフィールドを含む完全な入力が受け入れられることを確認
    it('全フィールド指定の入力が通る', () => {
      const validInput = {
        customer_id: 'CUST-001',
        currency: 'USD',
        items: [
          {
            product_id: 'PROD-001',
            product_name: 'テスト商品',
            quantity: 3,
            unit_price: 500,
          },
        ],
        notes: '配送メモ',
      };
      const result = createOrderInputSchema.safeParse(validInput);
      expect(result.success).toBe(true);
    });

    // 複数アイテムのある注文が受け入れられることを確認
    it('複数アイテムの入力が通る', () => {
      const validInput = {
        customer_id: 'CUST-001',
        items: [
          { product_id: 'P1', product_name: '商品A', quantity: 1, unit_price: 100 },
          { product_id: 'P2', product_name: '商品B', quantity: 2, unit_price: 200 },
        ],
      };
      const result = createOrderInputSchema.safeParse(validInput);
      expect(result.success).toBe(true);
    });
  });

  // ----------------------------------------------------------
  // 注文作成リクエストの異常系テスト。
  // 不正な入力がバリデーションで拒否されることを確認する。
  // ----------------------------------------------------------
  describe('createOrderInputSchema 異常系', () => {
    // 空の customer_id は min(1) 制約により拒否されること
    it('空の customer_id が拒否される', () => {
      const invalidInput = {
        customer_id: '',
        items: [{ product_id: 'P1', product_name: '商品A', quantity: 1, unit_price: 100 }],
      };
      const result = createOrderInputSchema.safeParse(invalidInput);
      expect(result.success).toBe(false);
    });

    // 空のアイテム配列は min(1) 制約により拒否されること
    it('空のアイテム配列が拒否される', () => {
      const invalidInput = {
        customer_id: 'CUST-001',
        items: [],
      };
      const result = createOrderInputSchema.safeParse(invalidInput);
      expect(result.success).toBe(false);
    });

    // 負の数量は min(1) 制約により拒否されること
    it('負の数量が拒否される', () => {
      const invalidInput = {
        customer_id: 'CUST-001',
        items: [{ product_id: 'P1', product_name: '商品A', quantity: -1, unit_price: 100 }],
      };
      const result = createOrderInputSchema.safeParse(invalidInput);
      expect(result.success).toBe(false);
    });

    // 負の単価は min(0) 制約により拒否されること
    it('負の単価が拒否される', () => {
      const invalidInput = {
        customer_id: 'CUST-001',
        items: [{ product_id: 'P1', product_name: '商品A', quantity: 1, unit_price: -100 }],
      };
      const result = createOrderInputSchema.safeParse(invalidInput);
      expect(result.success).toBe(false);
    });

    // customer_id が未指定の場合に拒否されること
    it('customer_id が未指定の場合に拒否される', () => {
      const invalidInput = {
        items: [{ product_id: 'P1', product_name: '商品A', quantity: 1, unit_price: 100 }],
      };
      const result = createOrderInputSchema.safeParse(invalidInput);
      expect(result.success).toBe(false);
    });
  });

  // ----------------------------------------------------------
  // 注文ステータス更新リクエストの正常系テスト。
  // ----------------------------------------------------------
  describe('updateOrderStatusInputSchema 正常系', () => {
    // 全ての有効なステータス値で更新リクエストが通ることを確認
    it.each(['pending', 'confirmed', 'processing', 'shipped', 'delivered', 'cancelled'] as const)(
      '有効なステータス "%s" で更新リクエストが通る',
      (status) => {
        const result = updateOrderStatusInputSchema.safeParse({ status });
        expect(result.success).toBe(true);
      }
    );
  });

  // ----------------------------------------------------------
  // 注文ステータス更新リクエストの異常系テスト。
  // ----------------------------------------------------------
  describe('updateOrderStatusInputSchema 異常系', () => {
    // 無効なステータス値は拒否されること
    it('無効なステータス値が拒否される', () => {
      const result = updateOrderStatusInputSchema.safeParse({ status: 'invalid_status' });
      expect(result.success).toBe(false);
    });

    // statusが欠落している場合に拒否されること
    it('status が欠落している場合に拒否される', () => {
      const result = updateOrderStatusInputSchema.safeParse({});
      expect(result.success).toBe(false);
    });
  });

  // ----------------------------------------------------------
  // 注文ステータスの契約検証。
  // サーバー側で定義された有効なステータス値のみが受け入れられることを確認する。
  // ----------------------------------------------------------
  describe('orderStatusSchema の契約検証', () => {
    // サーバー契約で定義された全ての有効なステータス値が受け入れられること
    it.each(['pending', 'confirmed', 'processing', 'shipped', 'delivered', 'cancelled'] as const)(
      '有効なステータス "%s" が受け入れられる',
      (status) => {
        const result = orderStatusSchema.safeParse(status);
        expect(result.success).toBe(true);
      }
    );

    // 定義外のステータス値が拒否されること
    it.each(['draft', 'paid', 'returned', 'archived'])(
      '無効なステータス "%s" が拒否される',
      (invalidStatus) => {
        const result = orderStatusSchema.safeParse(invalidStatus);
        expect(result.success).toBe(false);
      }
    );

    // 空文字列がステータスとして拒否されること
    it('空文字列がステータスとして拒否される', () => {
      const result = orderStatusSchema.safeParse('');
      expect(result.success).toBe(false);
    });
  });

  // ----------------------------------------------------------
  // orderSchema の必須フィールド検証。
  // ----------------------------------------------------------
  describe('orderSchema 必須フィールド検証', () => {
    // 完全なデータを基準とし、各フィールドの欠落テストに使用する
    const validOrder = {
      id: '550e8400-e29b-41d4-a716-446655440099',
      customer_id: 'CUST-099',
      status: 'pending',
      total_amount: 10000,
      currency: 'JPY',
      items: [
        {
          product_id: 'PROD-099',
          product_name: 'テスト商品',
          quantity: 2,
          unit_price: 5000,
          subtotal: 10000,
        },
      ],
      notes: null,
      version: 1,
      created_at: '2024-01-20T00:00:00Z',
      updated_at: '2024-01-20T00:00:00Z',
    };

    // 完全なデータがバリデーションを通過することを確認（基準テスト）
    it('完全なデータが orderSchema に準拠する', () => {
      const result = orderSchema.safeParse(validOrder);
      expect(result.success).toBe(true);
    });

    // id が UUID 形式でない場合に拒否されること
    it('id が UUID 形式でない場合に拒否される', () => {
      const result = orderSchema.safeParse({ ...validOrder, id: 'not-a-uuid' });
      expect(result.success).toBe(false);
    });

    // version フィールドが必須であることを確認（楽観的ロック制御に必要）
    it('version が欠落した場合に拒否される', () => {
      const { version: _, ...withoutVersion } = validOrder;
      const result = orderSchema.safeParse(withoutVersion);
      expect(result.success).toBe(false);
    });

    // 不正なステータス値を持つ注文が拒否されること
    it('不正な status を持つ注文が拒否される', () => {
      const result = orderSchema.safeParse({ ...validOrder, status: 'unknown_status' });
      expect(result.success).toBe(false);
    });

    // total_amount が負の値の場合に拒否されること
    it('total_amount が負の場合に拒否される', () => {
      const result = orderSchema.safeParse({ ...validOrder, total_amount: -1 });
      expect(result.success).toBe(false);
    });
  });
});
