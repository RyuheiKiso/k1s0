import { useMutation } from '@tanstack/react-query';
import { fetchDownloadUrl } from '../api/client';

interface DownloadParams {
  appId: string;
  version: string;
  platform?: 'windows' | 'linux' | 'macos';
  arch?: string;
}

// FE-04 対応: URL スキーム検証（http/https のみ許可）
// javascript: や data: スキームによる XSS・open redirect 攻撃を防ぐ
const isValidUrl = (url: string): boolean => {
  try {
    const parsed = new URL(url);
    return parsed.protocol === 'http:' || parsed.protocol === 'https:';
  } catch {
    return false;
  }
};

export function useDownload() {
  return useMutation({
    mutationFn: async ({ appId, version, platform, arch }: DownloadParams) => {
      const response = await fetchDownloadUrl(appId, version, platform, arch);

      // ダウンロードURLのスキームを検証し、http/https 以外のスキームへの遷移を拒否する
      if (!isValidUrl(response.download_url)) {
        throw new Error(`無効なダウンロードURL: ${response.download_url}`);
      }

      window.open(response.download_url, '_blank', 'noopener,noreferrer');
      return response;
    },
  });
}
