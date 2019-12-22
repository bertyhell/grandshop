import React, { FunctionComponent, MutableRefObject, useEffect, useRef } from 'react';
import iro from '@jaames/iro';

import classnames from 'classnames';

import './ColorInput.scss';

interface IroColorPicker {
  on: (action: string, callback: Function) => void;
  color: {
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
  let colorPicker: MutableRefObject<IroColorPicker | null> = useRef<IroColorPicker | null>(null);
  let el: HTMLDivElement | null = null;

  useEffect(() => {
    // create a new iro color picker and pass component props to it
    colorPicker.current = new iro.ColorPicker(el, { color: value });
    // call onColorChange prop whenever the color changes
    if (!colorPicker.current) {
      return;
    }
    colorPicker.current.on('color:change', (color: { hexString: string }) => {
      onChange(color.hexString);
    });
  }, [el, onChange]);

  useEffect(() => {
    if (!colorPicker.current) {
      return;
    }
    colorPicker.current.color.set(value);
    // this.colorPicker.setState(colorPickerState);
  });

  return <div className={classnames('o-color-input', className)}>
    <div
        ref={refEl => el = refEl}
    />
    <div className="a-color-preview" style={{backgroundColor: value}}/>
  </div>;
};

export default ColorInput;
