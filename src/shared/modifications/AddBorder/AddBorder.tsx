import React, { FunctionComponent, useEffect, useState } from 'react';
import classnames from 'classnames';

import ImageModification from '../../../components/ImageModification/ImageModification';
import FormField from '../../../components/FormField/FormField';
import ColorInput from '../../../components/ColorInput/ColorInput';
import { ImageModificationInfo } from '../../types';
import { MultiRange } from '../../../components/MultiRange/MultiRange';

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
      <ImageModification
          label={title}
          className={classnames('c-add-border', className)}
          isOpen={isOpen}
          onToggle={onToggle}
          onDelete={onDelete}
      >
        <FormField label="Dikte" labelFor="thickness">
          <MultiRange min={0} max={2000} step={1} values={[thickness]} onChange={(values: number[]) => setThickness(values[0])}/>
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
