import { useQuery } from '@tanstack/react-query';
import { useApps } from '../hooks/useApps';
import { fetchDownloadStats } from '../api/client';
import type { App, DownloadStats } from '../api/types';

function AppStatsRow({ app }: { app: App }) {
  const { data: stats } = useQuery({
    queryKey: ['stats', app.id],
    queryFn: () => fetchDownloadStats(app.id),
  });

  return (
    <tr>
      <td>{app.name}</td>
      <td>{app.category}</td>
      <td>{stats?.total_downloads ?? '-'}</td>
      <td>{stats?.version_downloads ?? '-'}</td>
      <td>{new Date(app.updated_at).toLocaleDateString('ja-JP')}</td>
    </tr>
  );
}

export function AdminPage() {
  const { data: apps, isLoading, error } = useApps();

  if (isLoading) {
    return <div className="loading">読み込み中...</div>;
  }

  if (error) {
    return <div className="error">データの読み込みに失敗しました</div>;
  }

  return (
    <div className="admin-page">
      <h1 className="admin-page__title">ダウンロード統計</h1>
      <table className="admin-page__table">
        <thead>
          <tr>
            <th>アプリ名</th>
            <th>カテゴリ</th>
            <th>総ダウンロード数</th>
            <th>最新バージョンDL数</th>
            <th>最終更新日</th>
          </tr>
        </thead>
        <tbody>
          {apps?.map((app) => <AppStatsRow key={app.id} app={app} />)}
        </tbody>
      </table>
    </div>
  );
}
