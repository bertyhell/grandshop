import { useTranslation } from 'react-i18next';
import React, { FunctionComponent, useEffect, useState } from 'react';

import { ModificationProps } from '../types';

import { TextArea } from '../../../renderer/src/components/TextArea/TextArea';
import FormField from '../../../renderer/src/components/FormField/FormField';
import { MultiRange } from '../../../renderer/src/components/MultiRange/MultiRange';
import ColorInput from '../../../renderer/src/components/ColorInput/ColorInput';

import './Annotate.scss';

const Annotate: FunctionComponent<ModificationProps> = ({ onApply }) => {
	const [t] = useTranslation();

	const [text, setText] = useState<string>('');
	const [fontSize, setFontSize] = useState<number>(30);
	const [xPosition, setXPosition] = useState<number>(50);
	const [yPosition, setYPosition] = useState<number>(95);
	const [textColor, setTextColor] = useState<string>('#000000');
	const [backgroundColor, setBackgroundColor] = useState<string>('#FFFFFF');

	useEffect(() => {
		if (!isNaN(fontSize) && !isNaN(xPosition) && !isNaN(yPosition))
			onApply({
				name: 'annotate',
				arguments: [
					text,
					Math.round(fontSize),
					Math.round(xPosition),
					Math.round(yPosition),
					textColor,
					backgroundColor,
				],
			});
	}, [text, fontSize, xPosition, yPosition, textColor, backgroundColor]);

	return (
		<>
			<FormField label={t('shared/modifications/annotate/annotate___text')}>
				<TextArea placeholder="Typ hier uw tekst in" value={text} onChange={setText} />
			</FormField>
			<FormField label={t('shared/modifications/annotate/annotate___text-size')}>
				<MultiRange
					min={0}
					max={200}
					step={1}
					values={[fontSize]}
					onChange={(values) => setFontSize(values[0])}
				/>
			</FormField>
			<FormField label={t('Horizontal position')}>
				<MultiRange
					min={-500}
					max={500}
					step={1}
					values={[xPosition]}
					onChange={(values) => setXPosition(values[0])}
				/>
			</FormField>
			<FormField label={t('Vertical position')}>
				<MultiRange
					min={-500}
					max={500}
					step={1}
					values={[yPosition]}
					onChange={(values) => setYPosition(values[0])}
				/>
			</FormField>
			<FormField label={t('Tekst kleur')}>
				<ColorInput value={textColor} onChange={setTextColor}></ColorInput>
			</FormField>
			<FormField label={t('Achtergrond kleur')}>
				<ColorInput value={backgroundColor} onChange={setBackgroundColor}></ColorInput>
			</FormField>
		</>
	);
};

export default Annotate;
