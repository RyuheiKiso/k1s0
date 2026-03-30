// H-14 監査対応: N+1 問題の解消
// 旧実装では AppStatsRow が各アプリ行をレンダリングするたびに個別に fetchDownloadStats を呼び出す
// N+1 パターン（N アプリに対して 1+N リクエスト）が発生していた。
// useQueries を使用して全アプリの統計を並列取得することで N+1 → 並列 N リクエストに改善する。
import { useQueries } from '@tanstack/react-query';
import { useApps } from '../hooks/useApps';
import { fetchDownloadStats } from '../api/client';

export function AdminPage() {
  const { data: apps, isLoading, error } = useApps();

  // H-14 監査対応: useQueries で全アプリの統計を並列取得する
  // apps が undefined の場合は空配列を渡し、クエリが発行されないようにする
  const statsQueries = useQueries({
    queries: (apps ?? []).map((app) => ({
      queryKey: ['stats', app.id],
      queryFn: () => fetchDownloadStats(app.id),
    })),
  });

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
            <th>最新バージョン</th>
            <th>最終更新日</th>
          </tr>
        </thead>
        <tbody>
          {/* statsQueries はアプリリストと同じ順序で並ぶため index で対応する統計を取得できる */}
          {apps?.map((app, index) => {
            const stats = statsQueries[index]?.data;
            return (
              <tr key={app.id}>
                <td>{app.name}</td>
                <td>{app.category}</td>
                <td>{stats?.total_downloads ?? '-'}</td>
                <td>{stats?.version_downloads ?? '-'}</td>
                <td>{stats?.latest_version ?? '-'}</td>
                <td>{new Date(app.updated_at).toLocaleDateString('ja-JP')}</td>
              </tr>
            );
          })}
        </tbody>
      </table>
    </div>
  );
}
