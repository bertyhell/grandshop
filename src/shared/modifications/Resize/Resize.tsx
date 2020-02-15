import React, { FunctionComponent, useEffect, useState } from 'react';
import prettyBytes from 'pretty-bytes';

import { MultiRange } from '../../../components/MultiRange/MultiRange';
import { ModificationProps } from '../types';
import FormField from '../../../components/FormField/FormField';

import './Resize.scss';

const Resize: FunctionComponent<ModificationProps> = ({
                                                        imageInfo,
                                                        onApply
                                                      }) => {
  const getMaxSize = () => {
    return Math.min(Math.max(imageInfo.dimensions.width, imageInfo.dimensions.height), 10000);
  };
  const [size, setSize] = useState<number>(getMaxSize());

  useEffect(() => {
    onApply({
          name: 'resize',
          arguments: [size]
        }
    );
  }, [size]); // eslint-disable-line react-hooks/exhaustive-deps

  return (
      <>
      <FormField label={t('Maximale afmeeting in pixels')}>
      <MultiRange
          min={10}
          max={getMaxSize()}
          step={1}
          values={[size]}
          onChange={(values: number[]) => setSize(values[0])}
      />
      </FormField>
        {/* https://softwareengineering.stackexchange.com/a/288673 */}
        <span>Resulterende bestandsgrootte: {prettyBytes((imageInfo.base64.length - 814) / 1.35)}</span>
      </>
  );
};

export default Resize;
