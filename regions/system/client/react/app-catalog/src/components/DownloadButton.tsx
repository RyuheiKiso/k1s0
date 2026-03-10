import { useDownload } from '../hooks/useDownload';

interface DownloadButtonProps {
  appId: string;
  versionId: string;
  label?: string;
}

export function DownloadButton({ appId, versionId, label = 'ダウンロード' }: DownloadButtonProps) {
  const { mutate, isPending } = useDownload();

  const handleClick = () => {
    mutate({ appId, versionId });
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
