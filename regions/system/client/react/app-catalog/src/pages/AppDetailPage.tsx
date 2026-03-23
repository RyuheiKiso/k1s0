// M-8 対応: safeImageUrl をインポートし、icon_url の XSS リスクを排除する
import { useParams } from 'react-router-dom';
import type { App, AppVersion } from '../api/types';
import { useAppDetail } from '../hooks/useAppDetail';
import { PlatformBadge } from '../components/PlatformBadge';
import { VersionHistory } from '../components/VersionHistory';
import { DownloadButton } from '../components/DownloadButton';
import { detectClientPlatform, formatArch, formatBytes, platformLabels } from '../lib/platform';
import { safeImageUrl } from '../lib/safeUrl';

function buildPreviewCards(app: App) {
  return [
    {
      title: `${app.name} ホーム`,
      description: '主要機能へ素早くアクセスするための開始画面。',
    },
    {
      title: '作業フロー',
      description: '更新後の操作イメージを確認できる代表画面。',
    },
    {
      title: 'レポート',
      description: app.description ?? 'リリース内容を反映したプレビュー。',
    },
  ];
}

function getRecommendedVersion(versions: AppVersion[]) {
  const detectedPlatform = detectClientPlatform();

  return {
    detectedPlatform,
    recommendedVersion:
      (detectedPlatform
        ? versions.find((version) => version.platform === detectedPlatform)
        : undefined) ?? versions[0],
  };
}

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
  const { detectedPlatform, recommendedVersion } = getRecommendedVersion(versions);
  const previewCards = buildPreviewCards(app);
  const supportedPlatforms = [...new Set(versions.map((version) => version.platform))];
  // icon_url を検証済みの安全なURLに変換する（javascript:/data: スキームは undefined になる）
  const iconUrl = safeImageUrl(app.icon_url);

  return (
    <div className="app-detail-page">
      <div className="app-detail-page__header">
        <div className="app-detail-page__icon">
          {iconUrl ? (
            <img src={iconUrl} alt={`${app.name} アイコン`} />
          ) : (
            <div className="app-detail-page__icon-placeholder">{app.name.charAt(0)}</div>
          )}
        </div>
        <div className="app-detail-page__info">
          <h1>{app.name}</h1>
          <span className="app-detail-page__category">{app.category}</span>
          {app.description && <p>{app.description}</p>}
          {recommendedVersion && (
            <div className="app-detail-page__download-panel">
              <div className="app-detail-page__recommendation">
                <p className="app-detail-page__eyebrow">推奨ダウンロード</p>
                <strong>
                  v{recommendedVersion.version} / {platformLabels[recommendedVersion.platform]} /{' '}
                  {formatArch(recommendedVersion.arch)}
                </strong>
                <p>
                  {detectedPlatform
                    ? `この端末では ${platformLabels[detectedPlatform]} 向けバイナリを推奨します。`
                    : '利用端末を判定できなかったため、最新バージョンを表示しています。'}
                </p>
              </div>
              <DownloadButton
                appId={app.id}
                version={recommendedVersion.version}
                platform={recommendedVersion.platform}
                arch={recommendedVersion.arch}
                label={`v${recommendedVersion.version} をダウンロード`}
              />
            </div>
          )}
        </div>
      </div>

      {recommendedVersion && (
        <section className="app-detail-page__summary">
          <div className="app-detail-page__summary-card">
            <p className="app-detail-page__summary-label">対応 OS</p>
            <div className="app-detail-page__platforms">
              {supportedPlatforms.map((platform) => (
                <PlatformBadge key={platform} platform={platform} />
              ))}
            </div>
          </div>
          <div className="app-detail-page__summary-card">
            <p className="app-detail-page__summary-label">ファイルサイズ</p>
            <strong>{formatBytes(recommendedVersion.size_bytes)}</strong>
          </div>
          <div className="app-detail-page__summary-card">
            <p className="app-detail-page__summary-label">SHA-256</p>
            <code>{recommendedVersion.checksum_sha256}</code>
          </div>
        </section>
      )}

      <section className="app-detail-page__screenshots">
        <div className="app-detail-page__section-heading">
          <h2>スクリーンショット</h2>
          <p>代表的な利用シーンをプレビューで確認できます。</p>
        </div>
        <div className="app-detail-page__preview-grid">
          {previewCards.map((preview) => (
            <article key={preview.title} className="app-detail-page__preview-card">
              <div className="app-detail-page__preview-visual">{app.name}</div>
              <h3>{preview.title}</h3>
              <p>{preview.description}</p>
            </article>
          ))}
        </div>
      </section>

      {recommendedVersion?.release_notes && (
        <div className="app-detail-page__release-notes">
          <h2>最新リリースノート</h2>
          <p>{recommendedVersion.release_notes}</p>
        </div>
      )}
      <VersionHistory versions={versions} appId={app.id} />
    </div>
  );
}
