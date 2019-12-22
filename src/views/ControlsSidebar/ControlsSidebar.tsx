import React, { FunctionComponent, useState } from 'react';

import { transformImage } from '../../electron-helpers/interop-web';

import './ControlsSidebar.scss';
import Button from '../../components/Button/Button';
import FormField from '../../components/FormField/FormField';
import Input from '../../components/Input/Input';
import ImageModification from '../../components/ImageModification/ImageModification';
import ColorInput from '../../components/ColorInput/ColorInput';

interface ControlsSidebarProps {
}

const ControlsSidebar: FunctionComponent<ControlsSidebarProps> = () => {
  const [thickness, setThickness] = useState<string>('30');
  const [color, setColor] = useState<string>('#225eff');

  const applyBorder = () => {
    try {
      transformImage(parseInt(thickness, 10), color);
    } catch (err) {
      // show toast
    }
  };

  return (
      <div className="m-controls-sidebar">
        <Button onClick={() => {
        }} label="Open afbeelding"/>
        <ImageModification label="Rand 1">
          <FormField label="Dikte" labelFor="thickness">
            <Input id="thickness" type="text" value={thickness} onChange={setThickness}/>
          </FormField>
          <FormField label="Kleur" labelFor="color">
            <ColorInput value={color} onChange={setColor} />
          </FormField>
          <Button onClick={() => applyBorder()} label="Rand toepassen"/>
        </ImageModification>
      </div>
  );
};

export default ControlsSidebar;
