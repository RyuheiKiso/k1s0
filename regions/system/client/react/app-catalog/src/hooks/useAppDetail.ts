import { useQuery } from '@tanstack/react-query';
import { fetchAppDetail } from '../api/client';

export function useAppDetail(appId: string) {
  return useQuery({
    queryKey: ['app', appId],
    queryFn: () => fetchAppDetail(appId),
    enabled: !!appId,
  });
}
