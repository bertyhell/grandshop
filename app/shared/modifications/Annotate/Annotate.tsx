import { useTranslation } from 'react-i18next';
import React, { FunctionComponent, useEffect, useState } from 'react';

import { ModificationProps } from '../types';

import './Annotate.scss';
import { TextArea } from '../../../components/TextArea/TextArea';
import FormField from '../../../components/FormField/FormField';
import { MultiRange } from '../../../components/MultiRange/MultiRange';

const Annotate: FunctionComponent<ModificationProps> = ({
                                                          onApply
                                                        }) => {
  const [t] = useTranslation();

  const [text, setText] = useState<string>('');
  const [fontSize, setFontSize] = useState<number>(30);
  const [xPosition, setXPosition] = useState<number>(50);
  const [yPosition, setYPosition] = useState<number>(95);

  useEffect(() => {
    onApply({
          name: 'annotate',
          arguments: [text, fontSize, xPosition, yPosition]
        }
    );
  }, [text, fontSize, xPosition, yPosition]); // eslint-disable-line react-hooks/exhaustive-deps

  return (
      <>
        <FormField label={t('shared/modifications/annotate/annotate___text')}>
          <TextArea placeholder="Typ hier uw tekst in" value={text} onChange={setText}/>
        </FormField>
        <FormField label={t('shared/modifications/annotate/annotate___text-size')}>
          <MultiRange min={0} max={200} values={[fontSize]} onChange={(values) => setFontSize(values[0])}/>
        </FormField>
        <FormField label={t('Horizontal position')}>
          <MultiRange min={0} max={100} step={0.01} values={[xPosition]} onChange={(values) => setXPosition(values[0])}/>
        </FormField>
        <FormField label={t('Vertical position')}>
          <MultiRange min={0} max={100} step={0.01} values={[yPosition]} onChange={(values) => setYPosition(values[0])}/>
        </FormField>
      </>
  );
};

export default Annotate;
