import React, { ChangeEvent, FunctionComponent } from 'react';
import TextareaAutosize from 'react-textarea-autosize';
import classnames from 'classnames';

export interface TextAreaProps {
  className?: string;
  id?: string;
  name?: string;
  minRows?: number;
  disabled?: boolean;
  placeholder?: string;
  value?: string;
  onChange?: (value: string) => void;
}

export const TextArea: FunctionComponent<TextAreaProps> = ({
                                                             className,
                                                             id,
                                                             name,
                                                             minRows = 4,
                                                             disabled = false,
                                                             placeholder,
                                                             value = '',
                                                             onChange = () => {},
                                                           }) => {
  function onValueChange(event: ChangeEvent<HTMLTextAreaElement>) {
    onChange(event.target.value);
  }

  return (
      <TextareaAutosize
          className={classnames(className, 'o-input')}
          id={id}
          name={name}
          minRows={minRows}
          disabled={disabled}
          placeholder={placeholder}
          value={value}
          onChange={onValueChange}
      />
  );
};
