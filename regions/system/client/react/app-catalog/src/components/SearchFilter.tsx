import { useState } from 'react';

interface SearchFilterProps {
  onSearch: (query: string) => void;
  onCategoryChange: (category: string) => void;
  categories: string[];
  selectedCategory: string;
}

export function SearchFilter({
  onSearch,
  onCategoryChange,
  categories,
  selectedCategory,
}: SearchFilterProps) {
  const [query, setQuery] = useState('');

  const handleSearchChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const value = e.target.value;
    setQuery(value);
    onSearch(value);
  };

  return (
    <div className="search-filter">
      <input
        type="text"
        className="search-filter__input"
        placeholder="アプリを検索..."
        value={query}
        onChange={handleSearchChange}
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
    </div>
  );
}
