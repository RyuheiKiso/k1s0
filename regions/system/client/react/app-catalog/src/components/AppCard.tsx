import { Link } from 'react-router-dom';
import type { App } from '../api/types';

interface AppCardProps {
  app: App;
  latestVersion?: string;
}

export function AppCard({ app, latestVersion }: AppCardProps) {
  return (
    <Link to={`/apps/${app.id}`} className="app-card" data-testid="app-card">
      <div className="app-card__icon">
        {app.icon_url ? (
          <img src={app.icon_url} alt={`${app.name} アイコン`} />
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
