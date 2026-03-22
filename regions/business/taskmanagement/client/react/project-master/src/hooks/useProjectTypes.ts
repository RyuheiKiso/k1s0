import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { apiClient } from '../lib/api-client';
import type {
  ProjectType,
  CreateProjectTypeInput,
  UpdateProjectTypeInput,
} from '../types/projectMaster';

// クエリキー定数: プロジェクトタイプ関連キャッシュの一元管理
const queryKeys = {
  projectTypes: ['project-types'] as const,
  projectType: (id: string) => ['project-types', id] as const,
};

// プロジェクトタイプ一覧を取得するフック
export function useProjectTypes(activeOnly?: boolean) {
  return useQuery({
    queryKey: [...queryKeys.projectTypes, { activeOnly }],
    queryFn: async () => {
      const params = activeOnly !== undefined ? { active_only: activeOnly } : {};
      const { data } = await apiClient.get<{ project_types: ProjectType[] }>(
        '/taskmanagement/project-types',
        { params }
      );
      return data.project_types;
    },
  });
}

// 単一プロジェクトタイプを取得するフック
export function useProjectType(id: string) {
  return useQuery({
    queryKey: queryKeys.projectType(id),
    queryFn: async () => {
      const { data } = await apiClient.get<ProjectType>(
        `/taskmanagement/project-types/${id}`
      );
      return data;
    },
    enabled: !!id,
  });
}

// プロジェクトタイプ作成ミューテーション
export function useCreateProjectType() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: async (input: CreateProjectTypeInput) => {
      const { data } = await apiClient.post<ProjectType>(
        '/taskmanagement/project-types',
        input
      );
      return data;
    },
    // 成功時にプロジェクトタイプ一覧キャッシュを無効化
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: queryKeys.projectTypes });
    },
  });
}

// プロジェクトタイプ更新ミューテーション
export function useUpdateProjectType(id: string) {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: async (input: UpdateProjectTypeInput) => {
      const { data } = await apiClient.put<ProjectType>(
        `/taskmanagement/project-types/${id}`,
        input
      );
      return data;
    },
    // 成功時にプロジェクトタイプ一覧と個別キャッシュを無効化
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: queryKeys.projectTypes });
      qc.invalidateQueries({ queryKey: queryKeys.projectType(id) });
    },
  });
}

// プロジェクトタイプ削除ミューテーション
export function useDeleteProjectType() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: async (id: string) => {
      await apiClient.delete(`/taskmanagement/project-types/${id}`);
    },
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: queryKeys.projectTypes });
    },
  });
}
