import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { apiClient } from '../lib/api-client';
import type {
  StatusDefinition,
  CreateStatusDefinitionInput,
  UpdateStatusDefinitionInput,
  StatusDefinitionVersion,
} from '../types/projectMaster';

// クエリキー定数: ステータス定義関連キャッシュの一元管理
const queryKeys = {
  statusDefinitions: (projectTypeId: string) =>
    ['project-types', projectTypeId, 'status-definitions'] as const,
  statusDefinition: (id: string) => ['status-definitions', id] as const,
  versions: (statusDefinitionId: string) =>
    ['status-definitions', statusDefinitionId, 'versions'] as const,
};

// プロジェクトタイプ配下のステータス定義一覧を取得するフック
export function useStatusDefinitions(projectTypeId: string) {
  return useQuery({
    queryKey: queryKeys.statusDefinitions(projectTypeId),
    queryFn: async () => {
      const { data } = await apiClient.get<{ status_definitions: StatusDefinition[] }>(
        `/taskmanagement/project-types/${projectTypeId}/status-definitions`
      );
      return data.status_definitions;
    },
    enabled: !!projectTypeId,
  });
}

// ステータス定義作成ミューテーション
export function useCreateStatusDefinition(projectTypeId: string) {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: async (input: CreateStatusDefinitionInput) => {
      const { data } = await apiClient.post<StatusDefinition>(
        `/taskmanagement/project-types/${projectTypeId}/status-definitions`,
        input
      );
      return data;
    },
    // 成功時にステータス定義一覧キャッシュを無効化
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: queryKeys.statusDefinitions(projectTypeId) });
    },
  });
}

// ステータス定義更新ミューテーション
export function useUpdateStatusDefinition(projectTypeId: string, id: string) {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: async (input: UpdateStatusDefinitionInput) => {
      const { data } = await apiClient.put<StatusDefinition>(
        `/taskmanagement/project-types/${projectTypeId}/status-definitions/${id}`,
        input
      );
      return data;
    },
    // 成功時に関連キャッシュを無効化
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: queryKeys.statusDefinitions(projectTypeId) });
      qc.invalidateQueries({ queryKey: queryKeys.statusDefinition(id) });
    },
  });
}

// ステータス定義削除ミューテーション
export function useDeleteStatusDefinition(projectTypeId: string) {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: async (id: string) => {
      await apiClient.delete(
        `/taskmanagement/project-types/${projectTypeId}/status-definitions/${id}`
      );
    },
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: queryKeys.statusDefinitions(projectTypeId) });
    },
  });
}

// ステータス定義のバージョン履歴を取得するフック
export function useStatusDefinitionVersions(statusDefinitionId: string) {
  return useQuery({
    queryKey: queryKeys.versions(statusDefinitionId),
    queryFn: async () => {
      const { data } = await apiClient.get<{ versions: StatusDefinitionVersion[] }>(
        `/taskmanagement/status-definitions/${statusDefinitionId}/versions`
      );
      return data.versions;
    },
    enabled: !!statusDefinitionId,
  });
}
