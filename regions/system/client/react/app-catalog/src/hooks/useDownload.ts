import { useMutation } from '@tanstack/react-query';
import { validateURL } from '@k1s0/validation';
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

      // MED-19 監査対応: ローカルの isValidUrl を @k1s0/validation の validateURL に統一する。
      // validateURL は http/https スキームのみ許可し、ValidationError をスローする。
      validateURL(response.download_url);

      window.open(response.download_url, '_blank', 'noopener,noreferrer');
      return response;
    },
  });
}
