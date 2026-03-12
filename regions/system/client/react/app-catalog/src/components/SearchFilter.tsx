import type { AppVersion } from '../api/types';

interface SearchFilterProps {
  query: string;
  onSearch: (query: string) => void;
  onCategoryChange: (category: string) => void;
  onPlatformChange: (platform: AppVersion['platform'] | '') => void;
  categories: string[];
  selectedCategory: string;
  selectedPlatform: AppVersion['platform'] | '';
}

export function SearchFilter({
  query,
  onSearch,
  onCategoryChange,
  onPlatformChange,
  categories,
  selectedCategory,
  selectedPlatform,
}: SearchFilterProps) {
  return (
    <div className="search-filter">
      <input
        type="text"
        className="search-filter__input"
        placeholder="アプリを検索..."
        value={query}
        onChange={(event) => onSearch(event.target.value)}
        aria-label="アプリを検索"
      />
      <select
        className="search-filter__category"
        value={selectedCategory}
        onChange={(e) => onCategoryChange(e.target.value)}
        aria-label="カテゴリで絞り込み"
      >
        <option value="">すべてのカテゴリ</option>
        {categories.map((cat) => (
          <option key={cat} value={cat}>
            {cat}
          </option>
        ))}
      </select>
      <select
        className="search-filter__platform"
        value={selectedPlatform}
        onChange={(event) => onPlatformChange(event.target.value as AppVersion['platform'] | '')}
        aria-label="OSで絞り込み"
      >
        <option value="">すべての OS</option>
        <option value="windows">Windows</option>
        <option value="macos">macOS</option>
        <option value="linux">Linux</option>
      </select>
    </div>
  );
}
