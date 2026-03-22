import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { apiClient } from '../lib/api-client';
import type { Task, CreateTaskInput, UpdateTaskStatusInput, TaskStatus } from '../types/task';

// クエリキー定数: キャッシュの一貫性を保つために一元管理
const queryKeys = {
  tasks: ['tasks'] as const,
  task: (id: string) => ['tasks', id] as const,
};

// タスク一覧を取得するフック（プロジェクトID・ステータス・担当者IDでフィルタ可能）
export function useTasks(projectId?: string, status?: TaskStatus, assigneeId?: string) {
  return useQuery({
    queryKey: [...queryKeys.tasks, { projectId, status, assigneeId }],
    queryFn: async () => {
      const params: Record<string, string> = {};
      if (projectId) params.project_id = projectId;
      if (status) params.status = status;
      if (assigneeId) params.assignee_id = assigneeId;
      const { data } = await apiClient.get<{ tasks: Task[] }>('/tasks', { params });
      return data.tasks;
    },
  });
}

// 単一タスクを取得するフック
export function useTask(id: string) {
  return useQuery({
    queryKey: queryKeys.task(id),
    queryFn: async () => {
      const { data } = await apiClient.get<Task>(`/tasks/${id}`);
      return data;
    },
    enabled: !!id,
  });
}

// タスク作成ミューテーション
export function useCreateTask() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: async (input: CreateTaskInput) => {
      const { data } = await apiClient.post<Task>('/tasks', input);
      return data;
    },
    // 成功時にタスク一覧キャッシュを無効化
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: queryKeys.tasks });
    },
  });
}

// タスクステータス更新ミューテーション
export function useUpdateTaskStatus(id: string) {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: async (input: UpdateTaskStatusInput) => {
      const { data } = await apiClient.put<Task>(`/tasks/${id}/status`, input);
      return data;
    },
    // 成功時にタスク一覧と個別タスクのキャッシュを無効化
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: queryKeys.tasks });
      qc.invalidateQueries({ queryKey: queryKeys.task(id) });
    },
  });
}
