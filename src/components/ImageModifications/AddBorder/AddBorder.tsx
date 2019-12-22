import React, { FunctionComponent, ReactNode, useEffect, useState } from 'react';
import classnames from 'classnames';

import './AddBorder.scss';
import ImageModification from '../../ImageModification/ImageModification';
import FormField from '../../FormField/FormField';
import Input from '../../Input/Input';
import ColorInput from '../../ColorInput/ColorInput';
import Button from '../../Button/Button';
import { applyModifications, ImageModificationInfo } from '../../../helpers/interop-electron';

interface AddBorderProps {
  className?: string;
  title: string;
  onApply: (modification: ImageModificationInfo) => void,
  isOpen: boolean,
  onToggle: () => void,
}

const AddBorder: FunctionComponent<AddBorderProps> = ({
                                                        className,
                                                        title,
                                                        onApply,
                                                        isOpen,
                                                        onToggle
                                                      }) => {

  const [thickness, setThickness] = useState<string>('100');
  const [color, setColor] = useState<string>('#225eff');

  useEffect(() => {
    applyBorder();
  });

  const applyBorder = () => {
    try {
      onApply({
            name: 'addBorder',
            arguments: [parseInt(thickness, 10), color]
          }
      );
    } catch (err) {
      // show toast
    }
  };

  return (
      <ImageModification label={title} className={classnames('c-add-border', className)} isOpen={isOpen} onToggle={onToggle}>
        <FormField label="Dikte" labelFor="thickness">
          <Input id="thickness" type="text" value={thickness} onChange={(value: string) => {
            setThickness(value);
          }}/>
        </FormField>
        <FormField label="Kleur" labelFor="color">
          <ColorInput value={color} onChange={(value: string) => {
            setColor(value);
          }}/>
        </FormField>
        <Button onClick={() => applyBorder()} label="Rand toepassen"/>
      </ImageModification>
  );
};

export default AddBorder;
