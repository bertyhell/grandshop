import React, { createRef, FunctionComponent, MutableRefObject, RefObject, useEffect, useRef } from 'react';
import iro from '@jaames/iro';

import classnames from 'classnames';

import './ColorInput.scss';

interface IroColorPicker {
  on: (action: string, callback: Function) => void;
  color: {
    hexString: string;
    set: (value: any) => void;
    setState: (state: any) => void;
  }
}

export interface ColorInputProps {
  className?: string;
  value?: string;
  onChange?: (color: string) => void;
}

const ColorInput: FunctionComponent<ColorInputProps> = ({
                                                          className,
                                                          value = '#850000',
                                                          onChange = () => {
                                                          }
                                                        }) => {
  const colorPicker: MutableRefObject<IroColorPicker | null> = useRef<IroColorPicker | null>(null);
  const el: RefObject<HTMLDivElement> = createRef<HTMLDivElement>();

  useEffect(() => {
    if (!el.current) {
      return;
    }
    if (!colorPicker.current) {
      // create a new iro color picker and pass component props to it
      colorPicker.current = new iro.ColorPicker(el.current, { color: value });
      // call onColorChange prop whenever the color changes
      if (!colorPicker.current) {
        return;
      }
      colorPicker.current.on('color:change', (color: { hexString: string }) => {
        onChange(color.hexString);
      });
    } else if(value !== colorPicker.current.color.hexString) {
      colorPicker.current.color.set(value);
    }
  });

  return <div className={classnames('o-color-input', className)}>
    <div
        ref={el}
    />
    <div className="a-color-preview" style={{backgroundColor: value}}/>
  </div>;
};

export default ColorInput;
