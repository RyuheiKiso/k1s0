import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { apiClient } from '../lib/api-client';
import type {
  Activity,
  ActivityType,
  CreateActivityInput,
  RejectActivityInput,
} from '../types/activity';

// クエリキー定数: キャッシュの一貫性を保つために一元管理
const queryKeys = {
  activities: ['activities'] as const,
  activity: (id: string) => ['activities', id] as const,
};

// アクティビティ一覧を取得するフック（タスクID・アクターID・種別でフィルタ可能）
export function useActivities(taskId?: string, actorId?: string, activityType?: ActivityType) {
  return useQuery({
    queryKey: [...queryKeys.activities, { taskId, actorId, activityType }],
    queryFn: async () => {
      const params: Record<string, string> = {};
      if (taskId) params.task_id = taskId;
      if (actorId) params.actor_id = actorId;
      if (activityType) params.activity_type = activityType;
      const { data } = await apiClient.get<{ activities: Activity[] }>('/activities', { params });
      return data.activities;
    },
  });
}

// 単一アクティビティを取得するフック
export function useActivity(id: string) {
  return useQuery({
    queryKey: queryKeys.activity(id),
    queryFn: async () => {
      const { data } = await apiClient.get<Activity>(`/activities/${id}`);
      return data;
    },
    enabled: !!id,
  });
}

// アクティビティ作成ミューテーション
export function useCreateActivity() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: async (input: CreateActivityInput) => {
      const { data } = await apiClient.post<Activity>('/activities', input);
      return data;
    },
    // 成功時にアクティビティ一覧キャッシュを無効化
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: queryKeys.activities });
    },
  });
}

// アクティビティ承認申請ミューテーション
export function useSubmitActivity(id: string) {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: async () => {
      const { data } = await apiClient.post<Activity>(`/activities/${id}/submit`, {});
      return data;
    },
    // 成功時に一覧と個別アクティビティのキャッシュを無効化
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: queryKeys.activities });
      qc.invalidateQueries({ queryKey: queryKeys.activity(id) });
    },
  });
}

// アクティビティ承認ミューテーション
export function useApproveActivity(id: string) {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: async () => {
      const { data } = await apiClient.post<Activity>(`/activities/${id}/approve`, {});
      return data;
    },
    // 成功時に一覧と個別アクティビティのキャッシュを無効化
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: queryKeys.activities });
      qc.invalidateQueries({ queryKey: queryKeys.activity(id) });
    },
  });
}

// アクティビティ却下ミューテーション
export function useRejectActivity(id: string) {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: async (input: RejectActivityInput) => {
      const { data } = await apiClient.post<Activity>(`/activities/${id}/reject`, input);
      return data;
    },
    // 成功時に一覧と個別アクティビティのキャッシュを無効化
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: queryKeys.activities });
      qc.invalidateQueries({ queryKey: queryKeys.activity(id) });
    },
  });
}
