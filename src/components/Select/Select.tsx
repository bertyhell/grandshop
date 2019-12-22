import React, { FunctionComponent } from 'react';
import ReactSelect, { ActionMeta } from 'react-select';

import classnames from 'classnames';

import './Select.scss';

export interface SelectOption<T = string> {
  value: T;
  label: string;
  disabled?: boolean;
}

export interface SelectProps {
  className?: string
  options: SelectOption[];
  id?: string;
  disabled?: boolean;
  loading?: boolean;
  clearable?: boolean;
  value?: string;
  placeholder?: string;
  onChange?: (value: string) => void;
}

export const Select: FunctionComponent<SelectProps> = ({
                                                         className,
                                                         options,
                                                         id,
                                                         disabled = false,
                                                         loading = false,
                                                         clearable = false,
                                                         placeholder,
                                                         value,
                                                         onChange = () => {},
                                                       }) => {
  function onValueChange(changedValue: SelectOption<string>, actionMeta: ActionMeta) {
    if (actionMeta.action !== 'create-option' && changedValue) {
      onChange(changedValue.value);
    }
  }

  return (
      <ReactSelect
          className={classnames('c-select', className)}
          classNamePrefix="c-select"
          id={id}
          options={options}
          defaultValue={options.find(option => option.value === value)}
          isMulti={false}
          isDisabled={disabled}
          isLoading={loading}
          isClearable={clearable}
          isOptionDisabled={option => !!option.disabled}
          placeholder={placeholder}
          onChange={onValueChange as any}
      />
  );
};
