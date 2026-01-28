/**
 * MUI Rating ラッパー
 */

import React from 'react';
import Box from '@mui/material/Box';
import Rating from '@mui/material/Rating';
import Typography from '@mui/material/Typography';
import FormHelperText from '@mui/material/FormHelperText';
import { Controller, FieldValues } from 'react-hook-form';
import type { FormFieldComponentProps } from '../types.js';

export function MuiRating<T extends FieldValues>(
  props: FormFieldComponentProps<T>
): React.ReactElement {
  const {
    name,
    label,
    helperText,
    config,
    parsedInfo,
    form,
    size = 'medium',
    disabled = false,
    readOnly = false,
  } = props;

  const max = config?.max ?? parsedInfo.max ?? 5;

  return (
    <Controller
      name={name}
      control={form.control}
      render={({ field, fieldState: { error } }) => (
        <Box>
          {label && (
            <Typography
              component="legend"
              color={error ? 'error' : 'textPrimary'}
              sx={{ mb: 0.5 }}
            >
              {label}
              {parsedInfo.required && ' *'}
            </Typography>
          )}
          <Rating
            {...field}
            value={field.value ?? 0}
            onChange={(_, value) => field.onChange(value)}
            max={max}
            size={size}
            disabled={disabled}
            readOnly={readOnly}
          />
          {(error?.message || helperText) && (
            <FormHelperText error={!!error}>
              {error?.message ?? helperText}
            </FormHelperText>
          )}
        </Box>
      )}
    />
  );
}
