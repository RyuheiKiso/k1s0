/**
 * ArrayField - 配列フィールドコンポーネント
 */

import React from 'react';
import Box from '@mui/material/Box';
import Button from '@mui/material/Button';
import IconButton from '@mui/material/IconButton';
import Typography from '@mui/material/Typography';
import Paper from '@mui/material/Paper';
import FormHelperText from '@mui/material/FormHelperText';
import AddIcon from '@mui/icons-material/Add';
import DeleteIcon from '@mui/icons-material/Delete';
import { useFieldArray, FieldValues, Path, ArrayPath } from 'react-hook-form';
import type { FormFieldComponentProps } from '../types.js';

export function ArrayField<T extends FieldValues>(
  props: FormFieldComponentProps<T> & {
    renderItem: (index: number, remove: () => void) => React.ReactNode;
    itemDefault?: Record<string, unknown>;
  }
): React.ReactElement {
  const {
    name,
    label,
    helperText,
    config,
    parsedInfo,
    form,
    disabled = false,
    renderItem,
    itemDefault = {},
  } = props;

  const { fields, append, remove } = useFieldArray({
    control: form.control,
    name: name as ArrayPath<T>,
  });

  const minItems = config?.minItems ?? 0;
  const maxItems = config?.maxItems ?? Infinity;
  const addButtonLabel = config?.addButtonLabel ?? '追加';

  const error = form.formState.errors[name];
  const errorMessage = error?.message ?? (error?.root as { message?: string })?.message;

  const canAdd = fields.length < maxItems && !disabled;
  const canRemove = fields.length > minItems && !disabled;

  return (
    <Box>
      {label && (
        <Typography
          variant="subtitle1"
          color={errorMessage ? 'error' : 'textPrimary'}
          sx={{ mb: 1 }}
        >
          {label}
          {parsedInfo.required && ' *'}
        </Typography>
      )}

      <Box sx={{ display: 'flex', flexDirection: 'column', gap: 2 }}>
        {fields.map((field, index) => (
          <Paper
            key={field.id}
            variant="outlined"
            sx={{ p: 2, position: 'relative' }}
          >
            {canRemove && (
              <IconButton
                size="small"
                onClick={() => remove(index)}
                sx={{ position: 'absolute', top: 8, right: 8 }}
                aria-label={`${index + 1}番目を削除`}
              >
                <DeleteIcon fontSize="small" />
              </IconButton>
            )}
            {renderItem(index, () => remove(index))}
          </Paper>
        ))}
      </Box>

      {canAdd && (
        <Button
          startIcon={<AddIcon />}
          onClick={() => append(itemDefault as Parameters<typeof append>[0])}
          sx={{ mt: 2 }}
          variant="outlined"
          size="small"
          disabled={disabled}
        >
          {addButtonLabel}
        </Button>
      )}

      {(errorMessage || helperText) && (
        <FormHelperText error={!!errorMessage} sx={{ mt: 1 }}>
          {errorMessage ?? helperText}
        </FormHelperText>
      )}
    </Box>
  );
}
