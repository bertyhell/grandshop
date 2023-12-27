import { useTranslation } from 'react-i18next';
import React, { FunctionComponent, useEffect, useState } from 'react';
import prettyBytes from 'pretty-bytes';

import { MultiRange } from '../../../renderer/src/components/MultiRange/MultiRange';
import { ModificationProps } from '../types';
import FormField from '../../../renderer/src/components/FormField/FormField';

import './Resize.scss';

const Resize: FunctionComponent<ModificationProps> = ({
	imageInfo,
	imageInfoAfterModification,
	onApply,
}) => {
	const [t] = useTranslation();

	const getMaxSize = () => {
		return Math.min(Math.max(imageInfo.dimensions.width, imageInfo.dimensions.height), 10000);
	};
	const [size, setSize] = useState<number>(getMaxSize());

	useEffect(() => {
		onApply({
			name: 'resize',
			arguments: [size],
		});
	}, [size]);

	return (
		<>
			<FormField
				label={t('shared/modifications/resize/resize___maximal-dimension-in-pixels')}
			>
				<MultiRange
					min={10}
					max={getMaxSize()}
					step={1}
					values={[size]}
					onChange={(values: number[]) => setSize(values[0])}
				/>
			</FormField>
			{/* https://softwareengineering.stackexchange.com/a/288673 */}
			<span>
				{t('shared/modifications/resize/resize___resulting-file-size')}:{' '}
				{prettyBytes((imageInfoAfterModification.base64.length - 814) / 1.35)}
			</span>
		</>
	);
};

export default Resize;
