import { useTranslation } from 'react-i18next';
import React, { FunctionComponent, useEffect, useState } from 'react';

import FormField from '../../components/FormField/FormField';
import ColorInput from '../../components/ColorInput/ColorInput';
import { MultiRange } from '../../components/MultiRange/MultiRange';
import { ModificationProps } from '../types';

import './AddBorder.scss';

const AddBorder: FunctionComponent<ModificationProps> = ({
                                                           onApply
                                                         }) => {
  const [t] = useTranslation();

  const [thickness, setThickness] = useState<number>(100);
  const [color, setColor] = useState<string>('#225eff');

  useEffect(() => {
    onApply({
          name: 'addBorder',
          arguments: [thickness, color]
        }
    );
  }, [thickness, color]); // eslint-disable-line react-hooks/exhaustive-deps

  return (
      <>
        <FormField label={t('shared/modifications/add-border/add-border___thickness')} labelFor="thickness">
          <MultiRange min={0} max={2000} step={1} values={[thickness]} onChange={(values: number[]) => setThickness(values[0])}/>
        </FormField>
        <FormField label={t('shared/modifications/add-border/add-border___color')} labelFor="color">
          <ColorInput value={color} onChange={(value: string) => {
            setColor(value);
          }}/>
        </FormField>
      </>
  );
};

export default AddBorder;
