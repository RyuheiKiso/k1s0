import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { apiClient } from '../lib/api-client';
import type { Payment, PaymentStatus, InitiatePaymentInput } from '../types/payment';

// クエリキー定数: キャッシュの一貫性を保つために一元管理
const queryKeys = {
  payments: ['payments'] as const,
  payment: (id: string) => ['payments', id] as const,
};

// 決済一覧を取得するフック（注文ID・顧客ID・ステータスでフィルタ可能）
export function usePayments(orderId?: string, customerId?: string, status?: PaymentStatus) {
  return useQuery({
    queryKey: [...queryKeys.payments, { orderId, customerId, status }],
    queryFn: async () => {
      // クエリパラメータの構築（値がある場合のみ追加）
      const params: Record<string, string> = {};
      if (orderId) params.order_id = orderId;
      if (customerId) params.customer_id = customerId;
      if (status) params.status = status;
      const { data } = await apiClient.get<{ payments: Payment[] }>('/list_payments', {
        params,
      });
      return data.payments;
    },
  });
}

// 単一決済を取得するフック
export function usePayment(id: string) {
  return useQuery({
    queryKey: queryKeys.payment(id),
    queryFn: async () => {
      const { data } = await apiClient.get<Payment>(`/get_payment/${id}`);
      return data;
    },
    enabled: !!id,
  });
}

// 決済開始ミューテーション: 新規決済を作成
export function useInitiatePayment() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: async (input: InitiatePaymentInput) => {
      const { data } = await apiClient.post<Payment>('/initiate_payment', input);
      return data;
    },
    // 成功時に決済一覧キャッシュを無効化
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: queryKeys.payments });
    },
  });
}

// 決済完了ミューテーション: 決済を完了状態に変更
export function useCompletePayment() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: async (id: string) => {
      const { data } = await apiClient.post<Payment>(`/complete_payment/${id}`);
      return data;
    },
    // 成功時に決済一覧と個別決済のキャッシュを無効化
    onSuccess: (_data, id) => {
      qc.invalidateQueries({ queryKey: queryKeys.payments });
      qc.invalidateQueries({ queryKey: queryKeys.payment(id) });
    },
  });
}

// 決済失敗ミューテーション: 決済を失敗状態に変更
export function useFailPayment() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: async (id: string) => {
      const { data } = await apiClient.post<Payment>(`/fail_payment/${id}`);
      return data;
    },
    // 成功時に決済一覧と個別決済のキャッシュを無効化
    onSuccess: (_data, id) => {
      qc.invalidateQueries({ queryKey: queryKeys.payments });
      qc.invalidateQueries({ queryKey: queryKeys.payment(id) });
    },
  });
}

// 決済返金ミューテーション: 決済を返金状態に変更
export function useRefundPayment() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: async (id: string) => {
      const { data } = await apiClient.post<Payment>(`/refund_payment/${id}`);
      return data;
    },
    // 成功時に決済一覧と個別決済のキャッシュを無効化
    onSuccess: (_data, id) => {
      qc.invalidateQueries({ queryKey: queryKeys.payments });
      qc.invalidateQueries({ queryKey: queryKeys.payment(id) });
    },
  });
}
