import { useParams } from 'react-router-dom';
import { useAppDetail } from '../hooks/useAppDetail';
import { VersionHistory } from '../components/VersionHistory';
import { DownloadButton } from '../components/DownloadButton';

export function AppDetailPage() {
  const { appId } = useParams<{ appId: string }>();
  const { data, isLoading, error } = useAppDetail(appId ?? '');

  if (isLoading) {
    return <div className="loading">読み込み中...</div>;
  }

  if (error || !data) {
    return <div className="error">アプリの読み込みに失敗しました</div>;
  }

  const { app, versions } = data;
  const latestVersion = versions[0];

  return (
    <div className="app-detail-page">
      <div className="app-detail-page__header">
        <div className="app-detail-page__icon">
          {app.icon_url ? (
            <img src={app.icon_url} alt={`${app.name} アイコン`} />
          ) : (
            <div className="app-detail-page__icon-placeholder">{app.name.charAt(0)}</div>
          )}
        </div>
        <div className="app-detail-page__info">
          <h1>{app.name}</h1>
          <span className="app-detail-page__category">{app.category}</span>
          {app.description && <p>{app.description}</p>}
          {latestVersion && (
            <DownloadButton
              appId={app.id}
              versionId={latestVersion.id}
              label={`v${latestVersion.version} をダウンロード`}
            />
          )}
        </div>
      </div>
      {latestVersion?.release_notes && (
        <div className="app-detail-page__release-notes">
          <h2>リリースノート</h2>
          <p>{latestVersion.release_notes}</p>
        </div>
      )}
      <VersionHistory versions={versions} appId={app.id} />
    </div>
  );
}
