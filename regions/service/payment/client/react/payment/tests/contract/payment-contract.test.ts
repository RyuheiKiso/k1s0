import { describe, it, expect } from 'vitest';
import {
  paymentSchema,
  paymentStatusSchema,
  initiatePaymentSchema,
} from '../../src/types/payment';
import { mockPayments } from '../testutil/msw-setup';

// ============================================================
// 契約テスト: クライアント側の型定義がサーバー契約に準拠していることを検証する。
// モックデータ・スキーマの整合性を保証し、サーバー契約の変更時に
// クライアント側の不整合を早期検出する。
// ============================================================

describe('決済契約テスト', () => {
  // ----------------------------------------------------------
  // モックデータがサーバー契約のスキーマに準拠していることを保証する。
  // MSWモックが正しい形式のレスポンスを返しているかの検証。
  // ----------------------------------------------------------
  describe('モックデータのスキーマ準拠検証', () => {
    // 全件のモックデータを個別にパースし、各レコードがスキーマに適合することを確認
    it.each(mockPayments.map((p, i) => [i, p]))(
      'mockPayments[%i] が paymentSchema に準拠する',
      (_index, payment) => {
        const result = paymentSchema.safeParse(payment);
        expect(result.success).toBe(true);
      }
    );

    // 全件を一括でパースし、配列全体としてスキーマに準拠することを確認
    it('mockPayments 全件が paymentSchema に準拠する', () => {
      for (const payment of mockPayments) {
        const result = paymentSchema.safeParse(payment);
        expect(result.success).toBe(true);
        if (!result.success) {
          // パースエラー時にデバッグ情報を出力する
          console.error('検証失敗:', result.error.format());
        }
      }
    });
  });

  // ----------------------------------------------------------
  // 決済開始リクエストの正常系テスト。
  // 有効な入力がバリデーションを通過することを確認する。
  // ----------------------------------------------------------
  describe('initiatePaymentSchema 正常系', () => {
    // 必須フィールドのみの最小限の入力が受け入れられることを確認
    it('必須フィールドのみで有効な入力が通る', () => {
      const validInput = {
        order_id: 'ORD-100',
        customer_id: 'CUS-100',
        amount: 5000,
      };
      const result = initiatePaymentSchema.safeParse(validInput);
      expect(result.success).toBe(true);
    });

    // すべてのフィールドを指定した完全な入力が受け入れられることを確認
    it('全フィールド指定の入力が通る', () => {
      const validInput = {
        order_id: 'ORD-200',
        customer_id: 'CUS-200',
        amount: 15000,
        currency: 'USD',
        payment_method: 'credit_card',
      };
      const result = initiatePaymentSchema.safeParse(validInput);
      expect(result.success).toBe(true);
    });

    // amount の最小値（1）が受け入れられることを確認（境界値テスト）
    it('amount が 1 の場合に通る（最小有効値）', () => {
      const validInput = {
        order_id: 'ORD-300',
        customer_id: 'CUS-300',
        amount: 1,
      };
      const result = initiatePaymentSchema.safeParse(validInput);
      expect(result.success).toBe(true);
    });
  });

  // ----------------------------------------------------------
  // 決済開始リクエストの異常系テスト。
  // 不正な入力がバリデーションで拒否されることを確認する。
  // ----------------------------------------------------------
  describe('initiatePaymentSchema 異常系', () => {
    // 空の order_id は min(1) 制約により拒否されること
    it('空の order_id が拒否される', () => {
      const invalidInput = {
        order_id: '',
        customer_id: 'CUS-100',
        amount: 5000,
      };
      const result = initiatePaymentSchema.safeParse(invalidInput);
      expect(result.success).toBe(false);
    });

    // 空の customer_id は min(1) 制約により拒否されること
    it('空の customer_id が拒否される', () => {
      const invalidInput = {
        order_id: 'ORD-100',
        customer_id: '',
        amount: 5000,
      };
      const result = initiatePaymentSchema.safeParse(invalidInput);
      expect(result.success).toBe(false);
    });

    // 負の amount は min(1) 制約により拒否されること
    it('負の amount が拒否される', () => {
      const invalidInput = {
        order_id: 'ORD-100',
        customer_id: 'CUS-100',
        amount: -1000,
      };
      const result = initiatePaymentSchema.safeParse(invalidInput);
      expect(result.success).toBe(false);
    });

    // amount が 0 は min(1) 制約により拒否されること（境界値テスト）
    it('amount が 0 の場合に拒否される', () => {
      const invalidInput = {
        order_id: 'ORD-100',
        customer_id: 'CUS-100',
        amount: 0,
      };
      const result = initiatePaymentSchema.safeParse(invalidInput);
      expect(result.success).toBe(false);
    });

    // order_id が欠落している場合に拒否されること
    it('order_id が未指定の場合に拒否される', () => {
      const invalidInput = {
        customer_id: 'CUS-100',
        amount: 5000,
      };
      const result = initiatePaymentSchema.safeParse(invalidInput);
      expect(result.success).toBe(false);
    });

    // amount に文字列を指定した場合に型エラーで拒否されること
    it('amount が文字列の場合に拒否される', () => {
      const invalidInput = {
        order_id: 'ORD-100',
        customer_id: 'CUS-100',
        amount: '5000',
      };
      const result = initiatePaymentSchema.safeParse(invalidInput);
      expect(result.success).toBe(false);
    });
  });

  // ----------------------------------------------------------
  // 決済ステータスの契約検証。
  // サーバー側で定義された有効なステータス値のみが受け入れられ、
  // 旧ステータス値（リネーム前）は拒否されることを確認する。
  // ----------------------------------------------------------
  describe('paymentStatusSchema の契約検証', () => {
    // サーバー契約で定義された有効なステータス値が受け入れられること
    it.each(['initiated', 'completed', 'failed', 'refunded'])(
      '有効なステータス "%s" が受け入れられる',
      (status) => {
        const result = paymentStatusSchema.safeParse(status);
        expect(result.success).toBe(true);
      }
    );

    // 旧ステータス値（サーバー契約変更前の値）が拒否されること
    // pending→initiated、processing→削除 に変更済み
    it.each(['pending', 'processing', 'cancelled', 'authorized'])(
      '旧ステータス "%s" が拒否される',
      (oldStatus) => {
        const result = paymentStatusSchema.safeParse(oldStatus);
        expect(result.success).toBe(false);
      }
    );

    // 空文字列がステータスとして拒否されること
    it('空文字列がステータスとして拒否される', () => {
      const result = paymentStatusSchema.safeParse('');
      expect(result.success).toBe(false);
    });

    // null がステータスとして拒否されること（ステータスは必須フィールド）
    it('null がステータスとして拒否される', () => {
      const result = paymentStatusSchema.safeParse(null);
      expect(result.success).toBe(false);
    });
  });

  // ----------------------------------------------------------
  // paymentSchema の必須フィールド検証。
  // サーバーレスポンスに含まれるべき必須フィールドが欠落した場合に
  // バリデーションエラーとなることを確認する。
  // ----------------------------------------------------------
  describe('paymentSchema 必須フィールド検証', () => {
    // 有効な完全データを基準とし、各フィールドの欠落テストに使用する
    const validPayment = {
      id: '550e8400-e29b-41d4-a716-446655440099',
      order_id: 'ORD-099',
      customer_id: 'CUS-099',
      amount: 10000,
      currency: 'JPY',
      status: 'initiated',
      payment_method: null,
      transaction_id: null,
      error_code: null,
      error_message: null,
      version: 1,
      created_at: '2024-01-20T00:00:00Z',
      updated_at: '2024-01-20T00:00:00Z',
    };

    // 完全なデータがバリデーションを通過することを確認（基準テスト）
    it('完全なデータが paymentSchema に準拠する', () => {
      const result = paymentSchema.safeParse(validPayment);
      expect(result.success).toBe(true);
    });

    // version フィールドが必須であることを確認（楽観的ロック制御に必要）
    it('version が欠落した場合に拒否される', () => {
      const { version: _, ...withoutVersion } = validPayment;
      const result = paymentSchema.safeParse(withoutVersion);
      expect(result.success).toBe(false);
    });

    // id フィールドが必須であることを確認
    it('id が欠落した場合に拒否される', () => {
      const { id: _, ...withoutId } = validPayment;
      const result = paymentSchema.safeParse(withoutId);
      expect(result.success).toBe(false);
    });

    // status フィールドが必須であることを確認
    it('status が欠落した場合に拒否される', () => {
      const { status: _, ...withoutStatus } = validPayment;
      const result = paymentSchema.safeParse(withoutStatus);
      expect(result.success).toBe(false);
    });

    // id が UUID 形式でない場合に拒否されること
    it('id が UUID 形式でない場合に拒否される', () => {
      const result = paymentSchema.safeParse({
        ...validPayment,
        id: 'not-a-uuid',
      });
      expect(result.success).toBe(false);
    });

    // created_at が欠落した場合に拒否されること
    it('created_at が欠落した場合に拒否される', () => {
      const { created_at: _, ...withoutCreatedAt } = validPayment;
      const result = paymentSchema.safeParse(withoutCreatedAt);
      expect(result.success).toBe(false);
    });

    // updated_at が欠落した場合に拒否されること
    it('updated_at が欠落した場合に拒否される', () => {
      const { updated_at: _, ...withoutUpdatedAt } = validPayment;
      const result = paymentSchema.safeParse(withoutUpdatedAt);
      expect(result.success).toBe(false);
    });

    // amount が負の値の場合に拒否されること（min(0) 制約）
    it('amount が負の値の場合に拒否される', () => {
      const result = paymentSchema.safeParse({
        ...validPayment,
        amount: -1,
      });
      expect(result.success).toBe(false);
    });
  });
});
