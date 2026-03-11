import { useDownload } from '../hooks/useDownload';

interface DownloadButtonProps {
  appId: string;
  version: string;
  platform?: 'windows' | 'linux' | 'macos';
  arch?: string;
  label?: string;
}

export function DownloadButton({
  appId,
  version,
  platform,
  arch,
  label = 'ダウンロード',
}: DownloadButtonProps) {
  const { mutate, isPending } = useDownload();

  const handleClick = () => {
    mutate({ appId, version, platform, arch });
  };

  return (
    <button
      type="button"
      className="download-button"
      onClick={handleClick}
      disabled={isPending}
      data-testid="download-button"
    >
      {isPending ? 'ダウンロード中...' : label}
    </button>
  );
}
