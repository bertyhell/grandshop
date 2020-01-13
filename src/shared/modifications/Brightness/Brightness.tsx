import React, { FunctionComponent, useEffect, useState } from 'react';

import { MultiRange } from '../../../components/MultiRange/MultiRange';
import { ModificationProps } from '../types';

import './Brightness.scss';

const Brightness: FunctionComponent<ModificationProps> = ({
                                                          onApply
                                                        }) => {
  const [brightness, setBrightness] = useState<number>(120);

  useEffect(() => {
    onApply({
          name: 'brightness',
          arguments: [brightness]
        }
    );
  }, [brightness]); // eslint-disable-line react-hooks/exhaustive-deps

  return (
      <MultiRange min={0} max={400} step={1} values={[brightness]} onChange={(values: number[]) => setBrightness(values[0])}/>
  );
};

export default Brightness;
