/**
 * MUI Checkbox ラッパー
 */

import React from 'react';
import FormControl from '@mui/material/FormControl';
import FormControlLabel from '@mui/material/FormControlLabel';
import Checkbox from '@mui/material/Checkbox';
import FormHelperText from '@mui/material/FormHelperText';
import { Controller, FieldValues } from 'react-hook-form';
import type { FormFieldComponentProps } from '../types.js';

export function MuiCheckbox<T extends FieldValues>(
  props: FormFieldComponentProps<T>
): React.ReactElement {
  const {
    name,
    label,
    helperText,
    parsedInfo,
    form,
    size = 'medium',
    disabled = false,
    readOnly = false,
  } = props;

  return (
    <Controller
      name={name}
      control={form.control}
      render={({ field, fieldState: { error } }) => (
        <FormControl
          error={!!error}
          required={parsedInfo.required}
          disabled={disabled}
        >
          <FormControlLabel
            control={
              <Checkbox
                {...field}
                checked={!!field.value}
                onChange={(e) => field.onChange(e.target.checked)}
                size={size}
                disabled={disabled || readOnly}
              />
            }
            label={label ?? ''}
          />
          {(error?.message || helperText) && (
            <FormHelperText>{error?.message ?? helperText}</FormHelperText>
          )}
        </FormControl>
      )}
    />
  );
}
