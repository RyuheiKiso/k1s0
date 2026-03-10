import { useMutation } from '@tanstack/react-query';
import { fetchDownloadUrl } from '../api/client';

interface DownloadParams {
  appId: string;
  versionId: string;
}

export function useDownload() {
  return useMutation({
    mutationFn: async ({ appId, versionId }: DownloadParams) => {
      const url = await fetchDownloadUrl(appId, versionId);
      window.open(url, '_blank');
      return url;
    },
  });
}
