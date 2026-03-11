import type { AppVersion } from '../api/types';
import { PlatformBadge } from './PlatformBadge';
import { DownloadButton } from './DownloadButton';
import { formatArch, formatBytes } from '../lib/platform';

interface VersionHistoryProps {
  versions: AppVersion[];
  appId: string;
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
            <th>チェックサム</th>
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
              <td>{formatArch(version.arch)}</td>
              <td>{formatBytes(version.size_bytes)}</td>
              <td>
                <code>{version.checksum_sha256}</code>
              </td>
              <td>{new Date(version.published_at).toLocaleDateString('ja-JP')}</td>
              <td>{version.mandatory ? 'はい' : 'いいえ'}</td>
              <td>
                <DownloadButton
                  appId={appId}
                  version={version.version}
                  platform={version.platform}
                  arch={version.arch}
                />
              </td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}
