import React, { FunctionComponent, useCallback, useEffect, useState } from 'react';

import { ModificationProps } from '../types';

import './AddImageBorder.scss';
import { Select } from '../../../components/Select/Select';
import toastService from '../../../services/toast-service';
import { CustomError } from '../../error';
import { getImageBorders } from '../../../services/renderer-interop';
import { compact } from 'lodash-es';
import Spinner from '../../../components/Spinner/Spinner';
import { analyseBorderPath, ImageBorderInfo } from './border-image-analyser';
import FormField from '../../../components/FormField/FormField';
import { MultiRange } from '../../../components/MultiRange/MultiRange';
import { useTranslation } from 'react-i18next';

const AddImageBorder: FunctionComponent<ModificationProps> = ({ onApply }) => {
	const [t] = useTranslation();
	const [imageBorders, setImageBorders] = useState<ImageBorderInfo[] | null>(null);
	const [selectedBorderPath, setSelectedBorderPath] = useState<string | null>(null);
	const [borderWidth, setBorderWidth] = useState<number>(100);

	const updateImageBorderList = useCallback(async () => {
		try {
			const borders: string[] = await getImageBorders();
			if (!borders) {
				return;
			}
			const analysedBorders = borders.map(analyseBorderPath);
			setImageBorders(compact(analysedBorders));
		} catch (err) {
			console.error(new CustomError('Failed to find image borders', err));
			toastService.error('Failed to find image borders');
		}
	}, [setImageBorders]);

	useEffect(() => {
		updateImageBorderList();
	}, [updateImageBorderList]);

	useEffect(() => {
		if (selectedBorderPath) {
			onApply({
				name: 'addImageBorder',
				arguments: [selectedBorderPath, borderWidth],
			});
		}
	}, [onApply, selectedBorderPath, borderWidth]);

	if (!imageBorders || !imageBorders.length) {
		return <Spinner />;
	}
	return (
		<>
			<FormField label={t('Border style')}>
				<Select
					options={imageBorders.map((imageBorder) => ({
						label: imageBorder.name,
						value: imageBorder.path,
					}))}
					value={selectedBorderPath}
					onChange={setSelectedBorderPath}
				/>
			</FormField>
			<FormField label={t('shared/modifications/annotate/annotate___text-size')}>
				<MultiRange
					min={0}
					max={500}
					values={[borderWidth]}
					onChange={(values) => setBorderWidth(values[0])}
				/>
			</FormField>
		</>
	);
};

export default AddImageBorder;
