import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { apiClient } from '../lib/api-client';
import type { Order, CreateOrderInput, UpdateOrderStatusInput, OrderStatus } from '../types/order';

// クエリキー定数: キャッシュの一貫性を保つために一元管理
const queryKeys = {
  orders: ['orders'] as const,
  order: (id: string) => ['orders', id] as const,
};

// 注文一覧を取得するフック（顧客IDとステータスでフィルタ可能）
export function useOrders(customerId?: string, status?: OrderStatus) {
  return useQuery({
    queryKey: [...queryKeys.orders, { customerId, status }],
    queryFn: async () => {
      const params: Record<string, string> = {};
      if (customerId) params.customer_id = customerId;
      if (status) params.status = status;
      const { data } = await apiClient.get<{ orders: Order[] }>('/orders', { params });
      return data.orders;
    },
  });
}

// 単一注文を取得するフック
export function useOrder(id: string) {
  return useQuery({
    queryKey: queryKeys.order(id),
    queryFn: async () => {
      const { data } = await apiClient.get<Order>(`/orders/${id}`);
      return data;
    },
    enabled: !!id,
  });
}

// 注文作成ミューテーション
export function useCreateOrder() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: async (input: CreateOrderInput) => {
      const { data } = await apiClient.post<Order>('/orders', input);
      return data;
    },
    // 成功時に注文一覧キャッシュを無効化
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: queryKeys.orders });
    },
  });
}

// 注文ステータス更新ミューテーション
export function useUpdateOrderStatus(id: string) {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: async (input: UpdateOrderStatusInput) => {
      const { data } = await apiClient.put<Order>(`/orders/${id}/status`, input);
      return data;
    },
    // 成功時に注文一覧と個別注文のキャッシュを無効化
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: queryKeys.orders });
      qc.invalidateQueries({ queryKey: queryKeys.order(id) });
    },
  });
}
