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
  const [marge, setMarge] = useState<number>(40);
  const [fontSize, setFontSize] = useState<number>(30);

  useEffect(() => {
    const lines = text.split('\n');
    onApply({
          name: 'annotate',
          arguments: [fontSize, marge, lines]
        }
    );
  }, [text, marge, fontSize]); // eslint-disable-line react-hooks/exhaustive-deps

  return (
      <>
        <FormField label={t('shared/modifications/annotate/annotate___text')}>
          <TextArea placeholder="Typ hier uw tekst in" value={text} onChange={setText}/>
        </FormField>
        <FormField label={t('shared/modifications/annotate/annotate___margin')}>
          <MultiRange min={0} max={400} values={[marge]} onChange={(values) => setMarge(values[0])}/>
        </FormField>
        <FormField label={t('shared/modifications/annotate/annotate___text-size')}>
          <MultiRange min={0} max={200} values={[fontSize]} onChange={(values) => setFontSize(values[0])}/>
        </FormField>
      </>
  );
};

export default Annotate;
