import React from 'react';

interface AppButtonProps {
  label: string;
  onClick: () => void;
  isLoading?: boolean;
  variant?: 'primary' | 'secondary' | 'text';
  disabled?: boolean;
}

export function AppButton({
  label,
  onClick,
  isLoading = false,
  variant = 'primary',
  disabled = false,
}: AppButtonProps) {
  return (
    <button
      type="button"
      onClick={onClick}
      disabled={disabled || isLoading}
      data-variant={variant}
    >
      {isLoading ? '...' : label}
    </button>
  );
}
