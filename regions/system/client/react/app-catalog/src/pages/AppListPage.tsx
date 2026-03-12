import { useMemo, useState } from 'react';
import { useQueries } from '@tanstack/react-query';
import { fetchAppVersions } from '../api/client';
import type { AppVersion } from '../api/types';
import { useApps } from '../hooks/useApps';
import { AppCard } from '../components/AppCard';
import { SearchFilter } from '../components/SearchFilter';

export function AppListPage() {
  const [search, setSearch] = useState('');
  const [category, setCategory] = useState('');
  const [platform, setPlatform] = useState<AppVersion['platform'] | ''>('');

  const { data: apps, isLoading, error } = useApps({ search, category: category || undefined });
  const versionQueries = useQueries({
    queries: (apps ?? []).map((app) => ({
      queryKey: ['app-versions', app.id],
      queryFn: () => fetchAppVersions(app.id),
      enabled: Boolean(apps),
      staleTime: 5 * 60 * 1000,
    })),
  });

  const categories = useMemo(() => {
    if (!apps) return [];
    return [...new Set(apps.map((app) => app.category))].sort();
  }, [apps]);

  const versionsByAppId = useMemo(
    () =>
      new Map(
        (apps ?? []).map((app, index) => [
          app.id,
          (versionQueries[index]?.data ?? []) as AppVersion[],
        ]),
      ),
    [apps, versionQueries],
  );

  const filteredApps = useMemo(() => {
    if (!apps) {
      return [];
    }

    return apps.filter((app) => {
      if (!platform) {
        return true;
      }
      const versions = versionsByAppId.get(app.id) ?? [];
      return versions.some((version) => version.platform === platform);
    });
  }, [apps, platform, versionsByAppId]);

  if (isLoading) {
    return <div className="loading">読み込み中...</div>;
  }

  if (error) {
    return <div className="error">アプリの読み込みに失敗しました</div>;
  }

  return (
    <div className="app-list-page">
      <h1 className="app-list-page__title">アプリカタログ</h1>
      <SearchFilter
        query={search}
        onSearch={setSearch}
        onCategoryChange={setCategory}
        onPlatformChange={setPlatform}
        categories={categories}
        selectedCategory={category}
        selectedPlatform={platform}
      />
      {platform && (
        <p className="app-list-page__hint">
          選択した OS に対応するアプリのみ表示しています。
        </p>
      )}
      <div className="app-list-page__grid">
        {filteredApps.length > 0 ? (
          filteredApps.map((app) => (
            <AppCard
              key={app.id}
              app={app}
              latestVersion={versionsByAppId.get(app.id)?.[0]?.version}
            />
          ))
        ) : (
          <p className="app-list-page__empty">アプリが見つかりません</p>
        )}
      </div>
    </div>
  );
}
