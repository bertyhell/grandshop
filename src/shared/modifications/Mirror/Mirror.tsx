import React, { FunctionComponent, useEffect, useState } from 'react';

import { ModificationProps } from '../types';

import './Mirror.scss';
import { Toggle } from '../../../renderer/src/components/Toggle/Toggle';
import { useTranslation } from 'react-i18next';

const Mirror: FunctionComponent<ModificationProps> = ({ onApply }) => {
	const [t] = useTranslation();
	const [isHorizontal, setIsHorizontal] = useState<boolean>(false);
	const [isVertical, setIsVertical] = useState<boolean>(false);

	useEffect(() => {
		onApply({
			name: 'mirror',
			arguments: [isHorizontal, isVertical],
		});
	}, [isHorizontal, isVertical]);

	return (
		<>
			<Toggle
				isChecked={isHorizontal}
				onChange={setIsHorizontal}
				label={t('Mirror horizontally')}
				offLabel={t('No')}
				onLabel={t('Yes')}
			/>
			<Toggle
				isChecked={isVertical}
				onChange={setIsVertical}
				label={t('Mirror Vertically')}
				offLabel={t('No')}
				onLabel={t('Yes')}
			/>
		</>
	);
};

export default Mirror;
