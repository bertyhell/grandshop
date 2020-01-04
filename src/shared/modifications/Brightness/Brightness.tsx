import React, { FunctionComponent, useEffect, useState } from 'react';

import { MultiRange } from '../../../components/MultiRange/MultiRange';
import { ImageModificationInfo } from '../types';

import './Brightness.scss';

interface BrightnessProps {
  onApply: (modification: ImageModificationInfo) => void,
}

const Brightness: FunctionComponent<BrightnessProps> = ({
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
