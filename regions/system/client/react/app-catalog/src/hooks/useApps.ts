import { useQuery } from '@tanstack/react-query';
import { fetchApps } from '../api/client';
import type { AppListParams } from '../api/types';

export function useApps(params?: AppListParams) {
  return useQuery({
    queryKey: ['apps', params],
    queryFn: () => fetchApps(params),
  });
}
