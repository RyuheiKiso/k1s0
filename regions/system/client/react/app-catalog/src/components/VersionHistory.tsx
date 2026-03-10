import type { AppVersion } from '../api/types';
import { PlatformBadge } from './PlatformBadge';
import { DownloadButton } from './DownloadButton';

interface VersionHistoryProps {
  versions: AppVersion[];
  appId: string;
}

function formatBytes(bytes: number | null): string {
  if (bytes === null) return '-';
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}

export function VersionHistory({ versions, appId }: VersionHistoryProps) {
  if (versions.length === 0) {
    return <p className="version-history__empty">バージョンがありません</p>;
  }

  return (
    <div className="version-history">
      <h2 className="version-history__title">バージョン履歴</h2>
      <table className="version-history__table">
        <thead>
          <tr>
            <th>バージョン</th>
            <th>プラットフォーム</th>
            <th>アーキテクチャ</th>
            <th>サイズ</th>
            <th>公開日</th>
            <th>必須</th>
            <th></th>
          </tr>
        </thead>
        <tbody>
          {versions.map((version) => (
            <tr key={version.id}>
              <td>{version.version}</td>
              <td>
                <PlatformBadge platform={version.platform} />
              </td>
              <td>{version.arch}</td>
              <td>{formatBytes(version.size_bytes)}</td>
              <td>{new Date(version.published_at).toLocaleDateString('ja-JP')}</td>
              <td>{version.mandatory ? 'はい' : 'いいえ'}</td>
              <td>
                <DownloadButton appId={appId} versionId={version.id} />
              </td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}
