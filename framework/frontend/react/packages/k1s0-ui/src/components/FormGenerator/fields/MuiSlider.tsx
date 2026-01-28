/**
 * MUI Slider ラッパー
 */

import React from 'react';
import Box from '@mui/material/Box';
import Slider from '@mui/material/Slider';
import Typography from '@mui/material/Typography';
import FormHelperText from '@mui/material/FormHelperText';
import { Controller, FieldValues } from 'react-hook-form';
import type { FormFieldComponentProps } from '../types.js';

export function MuiSlider<T extends FieldValues>(
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
  } = props;

  const min = config?.min ?? parsedInfo.min ?? 0;
  const max = config?.max ?? parsedInfo.max ?? 100;
  const step = config?.step ?? 1;
  const marks = config?.marks ?? false;

  return (
    <Controller
      name={name}
      control={form.control}
      render={({ field, fieldState: { error } }) => (
        <Box sx={{ width: '100%' }}>
          {label && (
            <Typography
              id={`${name}-slider-label`}
              gutterBottom
              color={error ? 'error' : 'textPrimary'}
            >
              {label}
              {parsedInfo.required && ' *'}
            </Typography>
          )}
          <Slider
            {...field}
            value={field.value ?? min}
            onChange={(_, value) => field.onChange(value)}
            aria-labelledby={label ? `${name}-slider-label` : undefined}
            valueLabelDisplay="auto"
            min={min}
            max={max}
            step={step}
            marks={marks}
            size={size}
            disabled={disabled}
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
