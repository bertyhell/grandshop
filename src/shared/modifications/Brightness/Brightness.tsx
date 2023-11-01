import React, { FunctionComponent, useEffect, useState } from 'react';

import { MultiRange } from '../../../renderer/src/components/MultiRange/MultiRange';
import { ModificationProps } from '../types';

import './Brightness.scss';
import { useTranslation } from 'react-i18next';

const Brightness: FunctionComponent<ModificationProps> = ({ onApply }) => {
	const [t] = useTranslation();
	const [brightness, setBrightness] = useState<number>(1.1);
	const [contrast, setContrast] = useState<number>(1.1);

	useEffect(() => {
		onApply({
			name: 'brightness',
			arguments: [brightness, contrast],
		});
	}, [brightness, contrast]);

	return (
		<>
			<label>{t('shared/modifications/brightness/brightness___brightness')}</label>
			<MultiRange
				min={0}
				max={5}
				step={0.01}
				values={[brightness]}
				onChange={(values: number[]) => setBrightness(values[0])}
			/>
			<label>{t('shared/modifications/brightness/brightness___contrast')}</label>
			<MultiRange
				min={0}
				max={5}
				step={0.01}
				values={[contrast]}
				onChange={(values: number[]) => setContrast(values[0])}
			/>
		</>
	);
};

export default Brightness;
