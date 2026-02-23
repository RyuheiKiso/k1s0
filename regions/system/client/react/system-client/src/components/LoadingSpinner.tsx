import React from 'react';

interface LoadingSpinnerProps {
  size?: number;
  message?: string;
}

export function LoadingSpinner({ size = 40, message }: LoadingSpinnerProps) {
  return (
    <div role="status" aria-label={message ?? 'Loading'}>
      <div
        style={{
          width: size,
          height: size,
          borderRadius: '50%',
          border: '3px solid #e0e0e0',
          borderTopColor: '#1976d2',
          animation: 'spin 0.8s linear infinite',
        }}
      />
      {message && <p>{message}</p>}
    </div>
  );
}
