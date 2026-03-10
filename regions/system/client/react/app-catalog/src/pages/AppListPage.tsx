import { useState, useMemo } from 'react';
import { useApps } from '../hooks/useApps';
import { AppCard } from '../components/AppCard';
import { SearchFilter } from '../components/SearchFilter';

export function AppListPage() {
  const [search, setSearch] = useState('');
  const [category, setCategory] = useState('');

  const { data: apps, isLoading, error } = useApps({ search, category: category || undefined });

  const categories = useMemo(() => {
    if (!apps) return [];
    return [...new Set(apps.map((app) => app.category))].sort();
  }, [apps]);

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
        onSearch={setSearch}
        onCategoryChange={setCategory}
        categories={categories}
        selectedCategory={category}
      />
      <div className="app-list-page__grid">
        {apps && apps.length > 0 ? (
          apps.map((app) => <AppCard key={app.id} app={app} />)
        ) : (
          <p className="app-list-page__empty">アプリが見つかりません</p>
        )}
      </div>
    </div>
  );
}
