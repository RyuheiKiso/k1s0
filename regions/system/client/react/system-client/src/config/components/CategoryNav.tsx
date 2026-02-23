import type { ConfigCategorySchema } from '../types';

interface CategoryNavProps {
  categories: ConfigCategorySchema[];
  activeId: string;
  onSelect: (id: string) => void;
}

export function CategoryNav({ categories, activeId, onSelect }: CategoryNavProps) {
  return (
    <nav className="category-nav" aria-label="設定カテゴリ">
      <ul>
        {categories.map((cat) => (
          <li key={cat.id}>
            <button
              type="button"
              className={cat.id === activeId ? 'category-nav__item--active' : 'category-nav__item'}
              onClick={() => onSelect(cat.id)}
              aria-current={cat.id === activeId ? 'true' : undefined}
            >
              {cat.icon && <span className="category-nav__icon">{cat.icon}</span>}
              {cat.label}
            </button>
          </li>
        ))}
      </ul>
    </nav>
  );
}
