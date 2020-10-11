import React, { FunctionComponent, useCallback, useEffect, useState } from 'react';

import { ModificationProps } from '../types';

import './AddImageBorder.scss';
import { Select } from '../../../components/Select/Select';
import toastService from '../../../services/toast-service';
import { CustomError } from '../../error';
import { getImageBorders } from '../../../services/renderer-interop';
import { compact } from 'lodash-es';
import Spinner from '../../../components/Spinner/Spinner';
import { analyseBorderPath, ImageBorderInfo } from './border-image-analyser';

const AddImageBorder: FunctionComponent<ModificationProps> = ({
  onApply,
}) => {
  const [imageBorders, setImageBorders] = useState<ImageBorderInfo[] | null>(
    null
  );
  const [selectedBorderPath, setSelectedBorderPath] = useState<string | null>(
    null
  );

  const updateImageBorderList = useCallback(async () => {
    try {
      const borders: string[] = await getImageBorders();
      if (!borders) {
        return;
      }
      console.log('image border got borders', JSON.stringify(borders));
      const analysedBorders = borders.map(analyseBorderPath);
      console.log('image border analysed borders', JSON.stringify(analysedBorders));
      setImageBorders(compact(analysedBorders));
    } catch( err) {
      console.error(new CustomError('Failed to find image borders', err));
      toastService.error('Failed to find image borders');
    }
  }, [setImageBorders])

  useEffect(() => {
    console.log('image border init');
    updateImageBorderList();
  }, [updateImageBorderList]);

  useEffect(() => {
    if (selectedBorderPath) {
      onApply({
        name: 'addImageBorder',
        arguments: [],
      });
    }
  }, [onApply, selectedBorderPath])

  if (!imageBorders || !imageBorders.length) {
    return <Spinner />;
  }
  return (
    <>
      <Select
        options={imageBorders.map((imageBorder) => ({
          label: imageBorder.name,
          value: imageBorder.path,
        }))}
        value={selectedBorderPath}
        onChange={setSelectedBorderPath}
      />
    </>
  );
};

export default AddImageBorder;
