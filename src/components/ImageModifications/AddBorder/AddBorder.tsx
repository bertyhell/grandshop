import React, { FunctionComponent, useEffect, useState } from 'react';
import classnames from 'classnames';

import ImageModification from '../../ImageModification/ImageModification';
import FormField from '../../FormField/FormField';
import Input from '../../Input/Input';
import ColorInput from '../../ColorInput/ColorInput';
import { ImageModificationInfo } from '../../../shared/types';

import './AddBorder.scss';

interface AddBorderProps {
  className?: string;
  title: string;
  onApply: (modification: ImageModificationInfo) => void,
  isOpen: boolean,
  onToggle: () => void,
  onDelete: () => void,
}

const AddBorder: FunctionComponent<AddBorderProps> = ({
                                                        className,
                                                        title,
                                                        onApply,
                                                        isOpen,
                                                        onToggle,
                                                        onDelete
                                                      }) => {

  const [thickness, setThickness] = useState<string>('100');
  const [color, setColor] = useState<string>('#225eff');

  useEffect(() => {
    onApply({
          name: 'addBorder',
          arguments: [parseInt(thickness, 10), color]
        }
    );
  }, [thickness, color]); // eslint-disable-line react-hooks/exhaustive-deps

  return (
      <ImageModification
          label={title}
          className={classnames('c-add-border', className)}
          isOpen={isOpen}
          onToggle={onToggle}
          onDelete={onDelete}
      >
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
      </ImageModification>
  );
};

export default AddBorder;
