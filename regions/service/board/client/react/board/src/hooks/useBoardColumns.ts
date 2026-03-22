import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { apiClient } from '../lib/api-client';
import type { BoardColumn, IncrementColumnInput, DecrementColumnInput, UpdateWipLimitInput } from '../types/board';

// クエリキー定数: キャッシュの一貫性を保つために一元管理
const queryKeys = {
  columns: (projectId: string) => ['boards', projectId, 'columns'] as const,
  column: (projectId: string, statusCode: string) => ['boards', projectId, 'columns', statusCode] as const,
};

// プロジェクトの全カラム一覧を取得するフック
export function useBoardColumns(projectId: string) {
  return useQuery({
    queryKey: queryKeys.columns(projectId),
    queryFn: async () => {
      const { data } = await apiClient.get<{ columns: BoardColumn[] }>(
        `/boards/${projectId}/columns`
      );
      return data.columns;
    },
    enabled: !!projectId,
  });
}

// 単一カラムを取得するフック
export function useBoardColumn(projectId: string, statusCode: string) {
  return useQuery({
    queryKey: queryKeys.column(projectId, statusCode),
    queryFn: async () => {
      const { data } = await apiClient.get<BoardColumn>(
        `/boards/${projectId}/columns/${statusCode}`
      );
      return data;
    },
    enabled: !!projectId && !!statusCode,
  });
}

// カラムのタスク数をインクリメントするミューテーション
export function useIncrementColumn() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: async (input: IncrementColumnInput) => {
      const { data } = await apiClient.post<BoardColumn>('/boards/increment', input);
      return data;
    },
    // 成功時に該当プロジェクトのカラム一覧キャッシュを無効化
    onSuccess: (_data, input) => {
      qc.invalidateQueries({ queryKey: queryKeys.columns(input.project_id) });
    },
  });
}

// カラムのタスク数をデクリメントするミューテーション
export function useDecrementColumn() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: async (input: DecrementColumnInput) => {
      const { data } = await apiClient.post<BoardColumn>('/boards/decrement', input);
      return data;
    },
    // 成功時に該当プロジェクトのカラム一覧キャッシュを無効化
    onSuccess: (_data, input) => {
      qc.invalidateQueries({ queryKey: queryKeys.columns(input.project_id) });
    },
  });
}

// カラムのWIP制限を更新するミューテーション
export function useUpdateWipLimit(projectId: string, statusCode: string) {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: async (input: UpdateWipLimitInput) => {
      const { data } = await apiClient.put<BoardColumn>(
        `/boards/${projectId}/columns/${statusCode}/wip-limit`,
        input
      );
      return data;
    },
    // 成功時にカラム一覧と個別カラムのキャッシュを無効化
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: queryKeys.columns(projectId) });
      qc.invalidateQueries({ queryKey: queryKeys.column(projectId, statusCode) });
    },
  });
}
