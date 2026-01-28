/**
 * MUI Autocomplete ラッパー
 */

import React from 'react';
import Autocomplete from '@mui/material/Autocomplete';
import TextField from '@mui/material/TextField';
import { Controller, FieldValues } from 'react-hook-form';
import type { FormFieldComponentProps, FieldOption } from '../types.js';

export function MuiAutocomplete<T extends FieldValues>(
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

  // 選択肢を取得
  const options: FieldOption[] =
    config?.options ??
    (parsedInfo.enumValues?.map((v) => ({ label: v, value: v })) ?? []);

  return (
    <Controller
      name={name}
      control={form.control}
      render={({ field, fieldState: { error } }) => (
        <Autocomplete
          value={
            options.find((opt) => opt.value === field.value) ?? null
          }
          onChange={(_, newValue) => {
            field.onChange(newValue?.value ?? null);
          }}
          options={options}
          getOptionLabel={(option) => option.label}
          isOptionEqualToValue={(option, value) => option.value === value.value}
          disabled={disabled}
          readOnly={readOnly}
          fullWidth={fullWidth}
          size={size}
          renderInput={(params) => (
            <TextField
              {...params}
              label={label}
              placeholder={placeholder}
              variant={variant}
              error={!!error}
              helperText={error?.message ?? helperText}
              required={parsedInfo.required}
            />
          )}
        />
      )}
    />
  );
}
