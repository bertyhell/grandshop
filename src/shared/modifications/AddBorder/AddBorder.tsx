import React, { FunctionComponent, useEffect, useState } from 'react';

import FormField from '../../../components/FormField/FormField';
import ColorInput from '../../../components/ColorInput/ColorInput';
import { MultiRange } from '../../../components/MultiRange/MultiRange';
import { ModificationProps } from '../types';

import './AddBorder.scss';

const AddBorder: FunctionComponent<ModificationProps> = ({
                                                           onApply
                                                         }) => {

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
        <FormField label={t('Dikte')} labelFor="thickness">
          <MultiRange min={0} max={2000} step={1} values={[thickness]} onChange={(values: number[]) => setThickness(values[0])}/>
        </FormField>
        <FormField label={t('Kleur')} labelFor="color">
          <ColorInput value={color} onChange={(value: string) => {
            setColor(value);
          }}/>
        </FormField>
      </>
  );
};

export default AddBorder;
