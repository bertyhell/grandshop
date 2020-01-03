import React, { FunctionComponent } from 'react';
import { getTrackBackground, Range } from 'react-range';

import classnames from 'classnames';

import './MultiRange.scss';
import toastService from '../../services/toast-service';
import Input from '../Input/Input';
import { clamp } from 'lodash-es';

export interface MultiRangeProps {
  className?: string;
  id?: string;
  disabled?: boolean;
  values?: number[];
  step?: number;
  min?: number;
  max?: number;
  allowOverlap?: boolean;
  onChange?: (values: number[]) => void;
}

export const MultiRange: FunctionComponent<MultiRangeProps> = ({
                                                                 className,
                                                                 id,
                                                                 disabled = false,
                                                                 values = [100],
                                                                 step = 0.1,
                                                                 min = 0,
                                                                 max = 100,
                                                                 allowOverlap = false,
                                                                 onChange = () => {},
                                                               }) => {
  const handleThicknessInputChanged = (value: string) => {
    try {
      const val = parseInt(value, 10);
      onChange([clamp(val, min, max)]);
    } catch (err) {
      toastService.warn('De waarde moet een geldig getal zijn');
    }
  };

  const classes = classnames('c-input-range', className, { 'c-input-range__disabled': disabled });

  const sortedValues = [...values];
  sortedValues.sort((a: number, b: number) => a - b);
  return (
      <div id={id} className={classes}>
        <Range
            step={step}
            min={min}
            max={max}
            allowOverlap={allowOverlap}
            values={values}
            onChange={onChange}
            renderTrack={({ props, children }: any) => (
                <div
                    onMouseDown={props.onMouseDown}
                    onTouchStart={props.onTouchStart}
                    style={{
                      ...props.style,
                      height: '36px',
                      display: 'flex',
                      width: '100%',
                    }}
                >
                  <div
                      ref={props.ref}
                      style={{
                        height: '8px',
                        width: '100%',
                        borderRadius: '4px',
                        background: getTrackBackground({
                          min,
                          max,
                          values: sortedValues,
                          colors:
                              values.length === 1
                                  ? ['hsl(190, 80%, 40%)', 'rgb(196, 196, 196)']
                                  : ['rgb(196, 196, 196)', 'hsl(190, 80%, 40%)', 'rgb(196, 196, 196)'],
                        }),
                        alignSelf: 'center',
                      }}
                  >
                    {children}
                  </div>
                </div>
            )}
            renderThumb={({ props }: any) => (
                <div
                    {...props}
                    style={{
                      ...props.style,
                      height: '40px',
                      width: '40px',
                    }}
                />
            )}
        />
        <Input type="number" value={values[0].toString()} onChange={handleThicknessInputChanged} />
      </div>
  );
};
