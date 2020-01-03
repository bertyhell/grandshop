import React, { FunctionComponent, useEffect, useState } from 'react';
import classnames from 'classnames';

import ImageModification from '../../../components/ImageModification/ImageModification';
import { MultiRange } from '../../../components/MultiRange/MultiRange';
import { ImageModificationInfo } from '../../types';

import './Brightness.scss';

interface BrightnessProps {
  className?: string;
  title: string;
  onApply: (modification: ImageModificationInfo) => void,
  isOpen: boolean,
  onToggle: () => void,
  onDelete: () => void,
}

const Brightness: FunctionComponent<BrightnessProps> = ({
                                                        className,
                                                        title,
                                                        onApply,
                                                        isOpen,
                                                        onToggle,
                                                        onDelete
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
      <ImageModification
          label={title}
          className={classnames('c-brightness', className)}
          isOpen={isOpen}
          onToggle={onToggle}
          onDelete={onDelete}
      >
        <MultiRange min={0} max={400} step={1} values={[brightness]} onChange={(values: number[]) => setBrightness(values[0])}/>
      </ImageModification>
  );
};

export default Brightness;
