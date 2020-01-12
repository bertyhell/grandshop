import React, { FunctionComponent, useEffect, useState } from 'react';

import { ModificationProps } from '../types';

import './Sepia.scss';
import FormField from '../../../components/FormField/FormField';
import { MultiRange } from '../../../components/MultiRange/MultiRange';
import ColorInput from '../../../components/ColorInput/ColorInput';

const Sepia: FunctionComponent<ModificationProps> = ({
                                                       onApply
                                                     }) => {
  const [brightness, setBrightness] = useState<number>(87);
  const [color, setColor] = useState<string>('#0f0a05');
  const [redPercent, setRedPercent] = useState<number>(7);
  const [greenPercent, setGreenPercent] = useState<number>(21);
  const [bluePercent, setBluePercent] = useState<number>(50);

  useEffect(() => {
    console.log('color: ', color);
    onApply({
          name: 'sepia',
          arguments: [brightness, color, redPercent, greenPercent, bluePercent]
        }
    );
  }, [brightness, color, redPercent, greenPercent, bluePercent]); // eslint-disable-line react-hooks/exhaustive-deps
  return <>
    <FormField label="Helderheid" labelFor="brightness">
      <MultiRange min={0} max={255} step={1} values={[brightness]} onChange={(values: number[]) => setBrightness(values[0])}/>
    </FormField>
    <FormField label="Kleur" labelFor="color">
      <ColorInput value={color} onChange={(value: string) => {
        setColor(value);
      }}/>
    </FormField>
    <FormField label="Rood accent" labelFor="redPercent">
      <MultiRange min={0} max={100} step={1} values={[redPercent]} onChange={(values: number[]) => setRedPercent(values[0])}/>
    </FormField>
    <FormField label="Groen accent" labelFor="greenPercent">
      <MultiRange min={0} max={100} step={1} values={[greenPercent]} onChange={(values: number[]) => setGreenPercent(values[0])}/>
    </FormField>
    <FormField label="Blauw accent" labelFor="bluePercent">
      <MultiRange min={0} max={100} step={1} values={[bluePercent]} onChange={(values: number[]) => setBluePercent(values[0])}/>
    </FormField>
  </>
};

export default Sepia;
