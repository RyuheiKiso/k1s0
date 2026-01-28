/**
 * FormContainer - フォームのコンテナコンポーネント
 */

import React from 'react';
import Box from '@mui/material/Box';

interface FormContainerProps {
  children: React.ReactNode;
  onSubmit: (e: React.FormEvent) => void;
  className?: string;
  sx?: Record<string, unknown>;
}

export function FormContainer(props: FormContainerProps): React.ReactElement {
  const { children, onSubmit, className, sx } = props;

  return (
    <Box
      component="form"
      onSubmit={onSubmit}
      noValidate
      autoComplete="off"
      className={className}
      sx={{
        width: '100%',
        ...sx,
      }}
    >
      {children}
    </Box>
  );
}
