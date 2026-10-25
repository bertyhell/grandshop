import { useTranslation } from 'react-i18next';
import { FunctionComponent, useEffect, useState } from 'react';

import { MultiRange } from '../../../renderer/src/components/MultiRange/MultiRange';
import FormField from '../../../renderer/src/components/FormField/FormField';
import { ModificationProps } from '../types';

import './Crop.scss';

const Crop: FunctionComponent<ModificationProps> = ({ imageInfo, onApply }) => {
	const [t] = useTranslation();

	const [top, setTop] = useState<number>(0);
	const [right, setRight] = useState<number>(0);
	const [bottom, setBottom] = useState<number>(0);
	const [left, setLeft] = useState<number>(0);

	useEffect(() => {
		onApply({
			name: 'crop',
			arguments: [top, right, bottom, left],
		});
	}, [top, right, bottom, left]);

	return (
		<>
			<FormField label={t('shared/modifications/crop/crop___top')} labelFor="top">
				<MultiRange
					min={0}
					max={imageInfo.dimensions.height - 1}
					step={1}
					id="top"
					values={[top]}
					onChange={(values: number[]) => setTop(values[0])}
				/>
			</FormField>
			<FormField label={t('shared/modifications/crop/crop___left')} labelFor="left">
				<MultiRange
					min={0}
					max={imageInfo.dimensions.width - 1}
					step={1}
					id="left"
					values={[left]}
					onChange={(values: number[]) => setLeft(values[0])}
				/>
			</FormField>
			<FormField label={t('shared/modifications/crop/crop___right')} labelFor="right">
				<MultiRange
					min={0}
					max={imageInfo.dimensions.width - 1}
					step={1}
					id="right"
					values={[right]}
					onChange={(values: number[]) => setRight(values[0])}
				/>
			</FormField>
			<FormField label={t('shared/modifications/crop/crop___bottom')} labelFor="bottom">
				<MultiRange
					min={0}
					max={imageInfo.dimensions.height - 1}
					step={1}
					id="bottom"
					values={[bottom]}
					onChange={(values: number[]) => setBottom(values[0])}
				/>
			</FormField>
		</>
	);
};

export default Crop;
