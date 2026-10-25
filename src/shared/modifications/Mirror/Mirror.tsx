import { FunctionComponent, useEffect, useState } from 'react';

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
				label={t('shared/modifications/mirror/mirror___mirror-horizontally')}
				offLabel={t('shared/modifications/mirror/mirror___no')}
				onLabel={t('shared/modifications/mirror/mirror___yes')}
			/>
			<Toggle
				isChecked={isVertical}
				onChange={setIsVertical}
				label={t('shared/modifications/mirror/mirror___mirror-vertically')}
				offLabel={t('shared/modifications/mirror/mirror___no')}
				onLabel={t('shared/modifications/mirror/mirror___yes')}
			/>
		</>
	);
};

export default Mirror;
