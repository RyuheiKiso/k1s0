/**
 * FormActions - フォームのアクションボタン群
 */

import React from 'react';
import Box from '@mui/material/Box';
import Button from '@mui/material/Button';
import LoadingButton from '@mui/lab/LoadingButton';

interface FormActionsProps {
  submitLabel?: string;
  cancelLabel?: string;
  resetLabel?: string;
  showCancel?: boolean;
  showReset?: boolean;
  onCancel?: () => void;
  onReset?: () => void;
  loading?: boolean;
  disabled?: boolean;
}

export function FormActions(props: FormActionsProps): React.ReactElement {
  const {
    submitLabel = '送信',
    cancelLabel = 'キャンセル',
    resetLabel = 'リセット',
    showCancel = false,
    showReset = false,
    onCancel,
    onReset,
    loading = false,
    disabled = false,
  } = props;

  return (
    <Box
      sx={{
        display: 'flex',
        gap: 2,
        justifyContent: 'flex-end',
        mt: 3,
      }}
    >
      {showReset && (
        <Button
          type="button"
          variant="text"
          onClick={onReset}
          disabled={disabled || loading}
        >
          {resetLabel}
        </Button>
      )}
      {showCancel && (
        <Button
          type="button"
          variant="outlined"
          onClick={onCancel}
          disabled={loading}
        >
          {cancelLabel}
        </Button>
      )}
      <LoadingButton
        type="submit"
        variant="contained"
        loading={loading}
        disabled={disabled}
      >
        {submitLabel}
      </LoadingButton>
    </Box>
  );
}
