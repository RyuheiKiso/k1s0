// ステータスバッジのバリアント定義（色の種類）
type BadgeVariant = 'success' | 'warning' | 'danger' | 'info' | 'neutral';

// 各バリアントに対応する背景色と文字色のマッピング
const variantStyles: Record<BadgeVariant, { backgroundColor: string; color: string }> = {
  success: { backgroundColor: '#d4edda', color: '#155724' },
  warning: { backgroundColor: '#fff3cd', color: '#856404' },
  danger: { backgroundColor: '#f8d7da', color: '#721c24' },
  info: { backgroundColor: '#cce5ff', color: '#004085' },
  neutral: { backgroundColor: '#e2e3e5', color: '#383d41' },
};

// StatusBadgeのProps定義
interface StatusBadgeProps {
  // バッジに表示するラベルテキスト
  label: string;
  // バッジの色バリアント
  variant: BadgeVariant;
}

// ステータスバッジコンポーネント: ステータスを色分けして視覚的に表示
export function StatusBadge({ label, variant }: StatusBadgeProps) {
  const style = variantStyles[variant];

  return (
    <span
      style={{
        padding: '2px 8px',
        borderRadius: '4px',
        fontSize: '0.85em',
        ...style,
      }}
      role="status"
      aria-label={label}
    >
      {label}
    </span>
  );
}
