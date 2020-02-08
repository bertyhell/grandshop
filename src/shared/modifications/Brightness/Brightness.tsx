import React, { FunctionComponent, useEffect, useState } from 'react';

import { MultiRange } from '../../../components/MultiRange/MultiRange';
import { ModificationProps } from '../types';

import './Brightness.scss';

const Brightness: FunctionComponent<ModificationProps> = ({
                                                          onApply
                                                        }) => {
  const [brightness, setBrightness] = useState<number>(1.1);

  useEffect(() => {
    onApply({
          name: 'brightness',
          arguments: [brightness]
        }
    );
  }, [brightness]); // eslint-disable-line react-hooks/exhaustive-deps

  return (
      <MultiRange min={0} max={5} step={0.01} values={[brightness]} onChange={(values: number[]) => setBrightness(values[0])}/>
  );
};

export default Brightness;
