import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { apiClient } from '../lib/api-client';
import type {
  MasterCategory,
  CreateCategoryInput,
  UpdateCategoryInput,
  MasterItem,
  CreateItemInput,
  UpdateItemInput,
  MasterItemVersion,
  TenantMasterExtension,
  UpdateTenantExtensionInput,
} from '../types/domain-master';

// クエリキー定数: キャッシュの一貫性を保つために一元管理
const queryKeys = {
  categories: ['categories'] as const,
  category: (code: string) => ['categories', code] as const,
  items: (categoryCode: string) => ['categories', categoryCode, 'items'] as const,
  item: (categoryCode: string, itemCode: string) =>
    ['categories', categoryCode, 'items', itemCode] as const,
  versions: (categoryCode: string, itemCode: string) =>
    ['categories', categoryCode, 'items', itemCode, 'versions'] as const,
  tenantExtension: (tenantId: string, itemId: string) =>
    ['tenants', tenantId, 'items', itemId] as const,
  tenantItems: (tenantId: string, categoryCode: string) =>
    ['tenants', tenantId, 'categories', categoryCode, 'items'] as const,
};

// カテゴリ一覧を取得するフック
export function useCategories(activeOnly?: boolean) {
  return useQuery({
    queryKey: [...queryKeys.categories, { activeOnly }],
    queryFn: async () => {
      const params = activeOnly !== undefined ? { active_only: activeOnly } : {};
      const { data } = await apiClient.get<{ categories: MasterCategory[] }>('/categories', {
        params,
      });
      return data.categories;
    },
  });
}

// 単一カテゴリを取得するフック
export function useCategory(code: string) {
  return useQuery({
    queryKey: queryKeys.category(code),
    queryFn: async () => {
      const { data } = await apiClient.get<MasterCategory>(`/categories/${code}`);
      return data;
    },
    enabled: !!code,
  });
}

// カテゴリ作成ミューテーション
export function useCreateCategory() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: async (input: CreateCategoryInput) => {
      const { data } = await apiClient.post<MasterCategory>('/categories', input);
      return data;
    },
    // 成功時にカテゴリ一覧キャッシュを無効化
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: queryKeys.categories });
    },
  });
}

// カテゴリ更新ミューテーション
export function useUpdateCategory(code: string) {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: async (input: UpdateCategoryInput) => {
      const { data } = await apiClient.put<MasterCategory>(`/categories/${code}`, input);
      return data;
    },
    // 成功時にカテゴリ一覧と個別カテゴリのキャッシュを無効化
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: queryKeys.categories });
      qc.invalidateQueries({ queryKey: queryKeys.category(code) });
    },
  });
}

// カテゴリ削除ミューテーション
export function useDeleteCategory() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: async (code: string) => {
      await apiClient.delete(`/categories/${code}`);
    },
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: queryKeys.categories });
    },
  });
}

// カテゴリ配下のアイテム一覧を取得するフック
export function useItems(categoryCode: string, activeOnly?: boolean) {
  return useQuery({
    queryKey: [...queryKeys.items(categoryCode), { activeOnly }],
    queryFn: async () => {
      const params = activeOnly !== undefined ? { active_only: activeOnly } : {};
      const { data } = await apiClient.get<{ items: MasterItem[] }>(
        `/categories/${categoryCode}/items`,
        { params }
      );
      return data.items;
    },
    enabled: !!categoryCode,
  });
}

// 単一アイテムを取得するフック
export function useItem(categoryCode: string, itemCode: string) {
  return useQuery({
    queryKey: queryKeys.item(categoryCode, itemCode),
    queryFn: async () => {
      const { data } = await apiClient.get<MasterItem>(
        `/categories/${categoryCode}/items/${itemCode}`
      );
      return data;
    },
    enabled: !!categoryCode && !!itemCode,
  });
}

// アイテム作成ミューテーション
export function useCreateItem(categoryCode: string) {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: async (input: CreateItemInput) => {
      const { data } = await apiClient.post<MasterItem>(
        `/categories/${categoryCode}/items`,
        input
      );
      return data;
    },
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: queryKeys.items(categoryCode) });
    },
  });
}

// アイテム更新ミューテーション
export function useUpdateItem(categoryCode: string, itemCode: string) {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: async (input: UpdateItemInput) => {
      const { data } = await apiClient.put<MasterItem>(
        `/categories/${categoryCode}/items/${itemCode}`,
        input
      );
      return data;
    },
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: queryKeys.items(categoryCode) });
      qc.invalidateQueries({ queryKey: queryKeys.item(categoryCode, itemCode) });
    },
  });
}

// アイテム削除ミューテーション
export function useDeleteItem(categoryCode: string) {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: async (itemCode: string) => {
      await apiClient.delete(`/categories/${categoryCode}/items/${itemCode}`);
    },
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: queryKeys.items(categoryCode) });
    },
  });
}

// アイテムのバージョン履歴を取得するフック
export function useVersions(categoryCode: string, itemCode: string) {
  return useQuery({
    queryKey: queryKeys.versions(categoryCode, itemCode),
    queryFn: async () => {
      const { data } = await apiClient.get<{ versions: MasterItemVersion[] }>(
        `/categories/${categoryCode}/items/${itemCode}/versions`
      );
      return data.versions;
    },
    enabled: !!categoryCode && !!itemCode,
  });
}

// テナント拡張情報を取得するフック
export function useTenantExtension(tenantId: string, itemId: string) {
  return useQuery({
    queryKey: queryKeys.tenantExtension(tenantId, itemId),
    queryFn: async () => {
      const { data } = await apiClient.get<TenantMasterExtension>(
        `/tenants/${tenantId}/items/${itemId}`
      );
      return data;
    },
    enabled: !!tenantId && !!itemId,
  });
}

// テナント拡張情報の更新ミューテーション
export function useUpdateTenantExtension(tenantId: string, itemId: string) {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: async (input: UpdateTenantExtensionInput) => {
      const { data } = await apiClient.put<TenantMasterExtension>(
        `/tenants/${tenantId}/items/${itemId}`,
        input
      );
      return data;
    },
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: queryKeys.tenantExtension(tenantId, itemId) });
    },
  });
}

// テナント拡張情報の削除ミューテーション
export function useDeleteTenantExtension(tenantId: string, itemId: string) {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: async () => {
      await apiClient.delete(`/tenants/${tenantId}/items/${itemId}`);
    },
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: queryKeys.tenantExtension(tenantId, itemId) });
    },
  });
}

// テナント別カテゴリアイテム一覧を取得するフック
export function useTenantCategoryItems(tenantId: string, categoryCode: string) {
  return useQuery({
    queryKey: queryKeys.tenantItems(tenantId, categoryCode),
    queryFn: async () => {
      const { data } = await apiClient.get<{ items: MasterItem[] }>(
        `/tenants/${tenantId}/categories/${categoryCode}/items`
      );
      return data.items;
    },
    enabled: !!tenantId && !!categoryCode,
  });
}
