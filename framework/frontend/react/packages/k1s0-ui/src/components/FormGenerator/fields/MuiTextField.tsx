/**
 * MUI TextField ラッパー
 */

import React from 'react';
import TextField from '@mui/material/TextField';
import { Controller, FieldValues, Path } from 'react-hook-form';
import type { FormFieldComponentProps } from '../types.js';
import { resolveInputType } from '../utils/fieldTypeResolver.js';

export function MuiTextField<T extends FieldValues>(
  props: FormFieldComponentProps<T>
): React.ReactElement {
  const {
    name,
    label,
    placeholder,
    helperText,
    config,
    parsedInfo,
    form,
    variant = 'outlined',
    size = 'medium',
    fullWidth = true,
    disabled = false,
    readOnly = false,
  } = props;

  const inputType = resolveInputType(parsedInfo, config);
  const isMultiline = config?.multiline ?? false;
  const rows = config?.rows ?? 4;
  const maxRows = config?.maxRows;

  return (
    <Controller
      name={name}
      control={form.control}
      render={({ field, fieldState: { error } }) => (
        <TextField
          {...field}
          value={field.value ?? ''}
          onChange={(e) => {
            const value = e.target.value;
            // 数値型の場合は変換
            if (inputType === 'number') {
              const num = value === '' ? undefined : Number(value);
              field.onChange(num);
            } else {
              field.onChange(value);
            }
          }}
          label={label}
          placeholder={placeholder}
          type={isMultiline ? 'text' : inputType}
          variant={variant}
          size={size}
          fullWidth={fullWidth}
          disabled={disabled}
          multiline={isMultiline}
          rows={isMultiline ? rows : undefined}
          maxRows={isMultiline ? maxRows : undefined}
          error={!!error}
          helperText={error?.message ?? helperText}
          required={parsedInfo.required}
          inputProps={{
            readOnly,
            min: parsedInfo.min,
            max: parsedInfo.max,
            step: config?.step,
          }}
        />
      )}
    />
  );
}
