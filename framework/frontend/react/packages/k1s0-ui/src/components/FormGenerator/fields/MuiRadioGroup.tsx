/**
 * MUI RadioGroup ラッパー
 */

import React from 'react';
import FormControl from '@mui/material/FormControl';
import FormLabel from '@mui/material/FormLabel';
import RadioGroup from '@mui/material/RadioGroup';
import FormControlLabel from '@mui/material/FormControlLabel';
import Radio from '@mui/material/Radio';
import FormHelperText from '@mui/material/FormHelperText';
import { Controller, FieldValues } from 'react-hook-form';
import type { FormFieldComponentProps, FieldOption } from '../types.js';

export function MuiRadioGroup<T extends FieldValues>(
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

  // 選択肢を取得
  const options: FieldOption[] =
    config?.options ??
    (parsedInfo.enumValues?.map((v) => ({ label: v, value: v })) ?? []);

  return (
    <Controller
      name={name}
      control={form.control}
      render={({ field, fieldState: { error } }) => (
        <FormControl
          component="fieldset"
          disabled={disabled}
          error={!!error}
          required={parsedInfo.required}
        >
          <FormLabel component="legend">{label}</FormLabel>
          <RadioGroup
            {...field}
            value={field.value ?? ''}
            row={options.length <= 3}
          >
            {options.map((option) => (
              <FormControlLabel
                key={String(option.value)}
                value={option.value}
                control={<Radio size={size} />}
                label={option.label}
                disabled={option.disabled || readOnly}
              />
            ))}
          </RadioGroup>
          {(error?.message || helperText) && (
            <FormHelperText>{error?.message ?? helperText}</FormHelperText>
          )}
        </FormControl>
      )}
    />
  );
}
