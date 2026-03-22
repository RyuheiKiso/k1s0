import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { apiClient } from '../lib/api-client';
import type {
  TenantProjectExtension,
  UpdateTenantExtensionInput,
} from '../types/projectMaster';

// クエリキー定数: テナント拡張関連キャッシュの一元管理
const queryKeys = {
  tenantExtensions: (tenantId: string) =>
    ['tenants', tenantId, 'extensions'] as const,
  tenantExtension: (tenantId: string, statusDefinitionId: string) =>
    ['tenants', tenantId, 'status-definitions', statusDefinitionId] as const,
};

// テナントのステータス定義拡張一覧を取得するフック
export function useTenantExtensions(tenantId: string) {
  return useQuery({
    queryKey: queryKeys.tenantExtensions(tenantId),
    queryFn: async () => {
      const { data } = await apiClient.get<{ extensions: TenantProjectExtension[] }>(
        `/taskmanagement/tenant-extensions`,
        { params: { tenant_id: tenantId } }
      );
      return data.extensions;
    },
    enabled: !!tenantId,
  });
}

// 単一テナント拡張を取得するフック
export function useTenantExtension(tenantId: string, statusDefinitionId: string) {
  return useQuery({
    queryKey: queryKeys.tenantExtension(tenantId, statusDefinitionId),
    queryFn: async () => {
      const { data } = await apiClient.get<TenantProjectExtension>(
        `/taskmanagement/tenant-extensions`,
        { params: { tenant_id: tenantId, status_definition_id: statusDefinitionId } }
      );
      return data;
    },
    enabled: !!tenantId && !!statusDefinitionId,
  });
}

// テナント拡張の作成ミューテーション（PUT で upsert）
export function useUpsertTenantExtension(tenantId: string, statusDefinitionId: string) {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: async (input: UpdateTenantExtensionInput) => {
      const { data } = await apiClient.put<TenantProjectExtension>(
        `/taskmanagement/tenant-extensions`,
        { tenant_id: tenantId, status_definition_id: statusDefinitionId, ...input }
      );
      return data;
    },
    // 成功時にテナント拡張関連キャッシュを無効化
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: queryKeys.tenantExtensions(tenantId) });
      qc.invalidateQueries({
        queryKey: queryKeys.tenantExtension(tenantId, statusDefinitionId),
      });
    },
  });
}

// テナント拡張削除ミューテーション
export function useDeleteTenantExtension(tenantId: string, statusDefinitionId: string) {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: async () => {
      await apiClient.delete(`/taskmanagement/tenant-extensions`, {
        params: { tenant_id: tenantId, status_definition_id: statusDefinitionId },
      });
    },
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: queryKeys.tenantExtensions(tenantId) });
      qc.invalidateQueries({
        queryKey: queryKeys.tenantExtension(tenantId, statusDefinitionId),
      });
    },
  });
}
