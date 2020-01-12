import React, { FunctionComponent, useEffect, useState } from 'react';

import { ModificationProps } from '../types';

import './Annotate.scss';
import { TextArea } from '../../../components/TextArea/TextArea';
import FormField from '../../../components/FormField/FormField';
import { MultiRange } from '../../../components/MultiRange/MultiRange';

const Annotate: FunctionComponent<ModificationProps> = ({
                                                          onApply
                                                        }) => {
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
        <FormField label="Tekst">
          <TextArea placeholder="Typ hier uw tekst in" value={text} onChange={setText}/>
        </FormField>
        <FormField label="Marge">
          <MultiRange min={0} max={400} values={[marge]} onChange={(values) => setMarge(values[0])}/>
        </FormField>
        <FormField label="Tekst grootte">
          <MultiRange min={0} max={200} values={[fontSize]} onChange={(values) => setFontSize(values[0])}/>
        </FormField>
      </>
  );
};

export default Annotate;
