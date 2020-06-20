import React, { createRef, FunctionComponent, MutableRefObject, RefObject, useEffect, useRef } from 'react';
import iro from '@jaames/iro';
import { useMouseMove } from 'react-use-mouse-move';
import { connect } from 'react-redux';
import classnames from 'classnames';

import { getColorAtPixel } from '../../services/interop-electron';
import { AppState } from '../../store/types';
import { selectColorAtPixel } from '../../store/selectors';

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
  colorAtMouse: string | null;
}

const ColorInput: FunctionComponent<ColorInputProps> = ({
                                                          className,
                                                          value = '#850000',
                                                          onChange = () => {},
                                                          colorAtMouse,
                                                        }) => {
  const colorPicker: MutableRefObject<IroColorPicker | null> = useRef<IroColorPicker | null>(null);
  const el: RefObject<HTMLDivElement> = createRef<HTMLDivElement>();

  const copyMouseColorToState = (evt: MouseEvent) => {
    console.log('clicked', evt.clientX, evt.clientY);
    getColorAtPixel(evt.clientX, evt.clientY);
  };

  useEffect(() => {
    if (colorAtMouse) {
      console.log('color under mouse changed react', colorAtMouse);
      onChange(colorAtMouse);
    }
  }, [colorAtMouse, onChange]);

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
    } else if (value !== colorPicker.current.color.hexString) {
      colorPicker.current.color.set(value);
    }
    document.addEventListener('click', copyMouseColorToState);

    // Destructor
    return () => {
      document.removeEventListener('click', copyMouseColorToState);
    }
  }, []);

  return <div className={classnames('o-color-input', className)}>
    <div
        ref={el}
    />
    <div className="a-color-preview" style={{backgroundColor: colorAtMouse || value}}/>
  </div>;
};

const mapStateToProps = (state: { app: AppState }) => ({
  colorAtMouse: selectColorAtPixel(state),
});

export default connect(mapStateToProps)(ColorInput);
