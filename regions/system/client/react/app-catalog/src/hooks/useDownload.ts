import { useMutation } from '@tanstack/react-query';
import { fetchDownloadUrl } from '../api/client';

interface DownloadParams {
  appId: string;
  version: string;
  platform?: 'windows' | 'linux' | 'macos';
  arch?: string;
}

export function useDownload() {
  return useMutation({
    mutationFn: async ({ appId, version, platform, arch }: DownloadParams) => {
      const response = await fetchDownloadUrl(appId, version, platform, arch);
      window.open(response.download_url, '_blank', 'noopener,noreferrer');
      return response;
    },
  });
}
