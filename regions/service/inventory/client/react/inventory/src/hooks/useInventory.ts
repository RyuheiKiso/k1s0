import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { apiClient } from '../lib/api-client';
import type { InventoryItem, StockOperation, UpdateStockInput } from '../types/inventory';

// クエリキー定数: キャッシュの一貫性を保つために一元管理
const queryKeys = {
  inventory: ['inventory'] as const,
  inventoryItem: (id: string) => ['inventory', id] as const,
};

// 在庫一覧を取得するフック
export function useInventoryList() {
  return useQuery({
    queryKey: queryKeys.inventory,
    queryFn: async () => {
      const { data } = await apiClient.get<{ items: InventoryItem[] }>('/inventory');
      return data.items;
    },
  });
}

// 単一の在庫アイテムを取得するフック
export function useInventoryItem(id: string) {
  return useQuery({
    queryKey: queryKeys.inventoryItem(id),
    queryFn: async () => {
      const { data } = await apiClient.get<InventoryItem>(`/inventory/${id}`);
      return data;
    },
    enabled: !!id,
  });
}

// 在庫予約ミューテーション: 指定数量を予約状態にする
export function useReserveStock() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: async (input: StockOperation) => {
      const { data } = await apiClient.post<InventoryItem>('/inventory/reserve', input);
      return data;
    },
    // 成功時に在庫一覧キャッシュを無効化
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: queryKeys.inventory });
    },
  });
}

// 在庫予約解放ミューテーション: 予約済み数量を解放する
export function useReleaseStock() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: async (input: StockOperation) => {
      const { data } = await apiClient.post<InventoryItem>('/inventory/release', input);
      return data;
    },
    // 成功時に在庫一覧キャッシュを無効化
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: queryKeys.inventory });
    },
  });
}

// 在庫数更新ミューテーション: 利用可能数・再注文点を更新する
export function useUpdateStock(id: string) {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: async (input: UpdateStockInput) => {
      const { data } = await apiClient.put<InventoryItem>(`/inventory/${id}/stock`, input);
      return data;
    },
    // 成功時に在庫一覧と個別在庫のキャッシュを無効化
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: queryKeys.inventory });
      qc.invalidateQueries({ queryKey: queryKeys.inventoryItem(id) });
    },
  });
}
