// M-8 対応: safeImageUrl をインポートし、icon_url の XSS リスクを排除する
import { Link } from 'react-router-dom';
import type { App } from '../api/types';
import { safeImageUrl } from '../lib/safeUrl';

interface AppCardProps {
  app: App;
  latestVersion?: string;
}

export function AppCard({ app, latestVersion }: AppCardProps) {
  // icon_url を検証済みの安全なURLに変換する（javascript:/data: スキームは undefined になる）
  const iconUrl = safeImageUrl(app.icon_url);
  return (
    <Link to={`/apps/${app.id}`} className="app-card" data-testid="app-card">
      <div className="app-card__icon">
        {iconUrl ? (
          <img src={iconUrl} alt={`${app.name} アイコン`} />
        ) : (
          <div className="app-card__icon-placeholder">{app.name.charAt(0)}</div>
        )}
      </div>
      <div className="app-card__body">
        <h3 className="app-card__name">{app.name}</h3>
        {app.description && (
          <p className="app-card__description">{app.description}</p>
        )}
        <div className="app-card__meta">
          <span className="app-card__category">{app.category}</span>
          {latestVersion && (
            <span className="app-card__version">v{latestVersion}</span>
          )}
        </div>
      </div>
    </Link>
  );
}
