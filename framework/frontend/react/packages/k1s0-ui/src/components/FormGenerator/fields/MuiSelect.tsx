/**
 * MUI Select ラッパー
 */

import React from 'react';
import FormControl from '@mui/material/FormControl';
import InputLabel from '@mui/material/InputLabel';
import Select from '@mui/material/Select';
import MenuItem from '@mui/material/MenuItem';
import FormHelperText from '@mui/material/FormHelperText';
import { Controller, FieldValues } from 'react-hook-form';
import type { FormFieldComponentProps, FieldOption } from '../types.js';

export function MuiSelect<T extends FieldValues>(
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

  // 選択肢を取得（config から、または enum 値から）
  const options: FieldOption[] =
    config?.options ??
    (parsedInfo.enumValues?.map((v) => ({ label: v, value: v })) ?? []);

  const labelId = `${name}-label`;

  return (
    <Controller
      name={name}
      control={form.control}
      render={({ field, fieldState: { error } }) => (
        <FormControl
          variant={variant}
          size={size}
          fullWidth={fullWidth}
          disabled={disabled}
          error={!!error}
          required={parsedInfo.required}
        >
          <InputLabel id={labelId}>{label}</InputLabel>
          <Select
            {...field}
            value={field.value ?? ''}
            labelId={labelId}
            label={label}
            readOnly={readOnly}
            displayEmpty={!!placeholder}
          >
            {placeholder && (
              <MenuItem value="" disabled>
                <em>{placeholder}</em>
              </MenuItem>
            )}
            {options.map((option) => (
              <MenuItem
                key={String(option.value)}
                value={option.value as string | number}
                disabled={option.disabled}
              >
                {option.label}
              </MenuItem>
            ))}
          </Select>
          {(error?.message || helperText) && (
            <FormHelperText>{error?.message ?? helperText}</FormHelperText>
          )}
        </FormControl>
      )}
    />
  );
}
