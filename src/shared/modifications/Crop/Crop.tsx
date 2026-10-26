import { useTranslation } from 'react-i18next';
import React, { FunctionComponent, PointerEvent as ReactPointerEvent, useEffect, useRef, useState } from 'react';

import { MultiRange } from '../../../components/MultiRange/MultiRange';
import { ModificationProps } from '../types';

import './Crop.scss';
import FormField from '../../../components/FormField/FormField';
import { getImageSource } from '../../helpers/base64';

interface DragStart {
	x: number;
	y: number;
	top: number;
	right: number;
	bottom: number;
	left: number;
}

const Crop: FunctionComponent<ModificationProps> = ({
	imageInfo,
	onApply
}) => {
	const [t] = useTranslation();

	const [top, setTop] = useState<number>(0);
	const [right, setRight] = useState<number>(0);
	const [bottom, setBottom] = useState<number>(0);
	const [left, setLeft] = useState<number>(0);

	const previewRef = useRef<HTMLDivElement>(null);
	const dragStart = useRef<DragStart | null>(null);

	const { width, height } = imageInfo.dimensions;

	useEffect(() => {
		onApply({
				name: 'crop',
				arguments: [top, right, bottom, left]
			}
		);
	}, [top, right, bottom, left]); // eslint-disable-line react-hooks/exhaustive-deps

	const handleDragStart = (evt: ReactPointerEvent<HTMLDivElement>) => {
		evt.preventDefault();
		evt.currentTarget.setPointerCapture(evt.pointerId);
		dragStart.current = { x: evt.clientX, y: evt.clientY, top, right, bottom, left };
	};

	const handleDragMove = (evt: ReactPointerEvent<HTMLDivElement>) => {
		const start = dragStart.current;
		if (!start || !previewRef.current) {
			return;
		}
		// Convert the mouse movement from screen pixels to image pixels
		const scale = width / previewRef.current.clientWidth;
		const dx = Math.round((evt.clientX - start.x) * scale);
		const dy = Math.round((evt.clientY - start.y) * scale);

		// Move the crop area without resizing it, keeping it inside the image
		const clampedDx = Math.min(Math.max(dx, -start.left), start.right);
		const clampedDy = Math.min(Math.max(dy, -start.top), start.bottom);

		setLeft(start.left + clampedDx);
		setRight(start.right - clampedDx);
		setTop(start.top + clampedDy);
		setBottom(start.bottom - clampedDy);
	};

	const handleDragEnd = () => {
		dragStart.current = null;
	};

	return (
		<>
			<div className="c-crop-preview" ref={previewRef}>
				<img src={getImageSource(imageInfo.base64)} alt="" draggable={false} />
				<div
					className="c-crop-preview__area"
					style={{
						top: `${top / height * 100}%`,
						right: `${right / width * 100}%`,
						bottom: `${bottom / height * 100}%`,
						left: `${left / width * 100}%`
					}}
				>
					<div
						className="c-crop-preview__handle"
						onPointerDown={handleDragStart}
						onPointerMove={handleDragMove}
						onPointerUp={handleDragEnd}
						onPointerCancel={handleDragEnd}
					/>
				</div>
			</div>
			<FormField label={t('shared/modifications/crop/crop___top')} labelFor="top">
				<MultiRange min={0} max={height} step={1} id="top" values={[top]}
										onChange={(values: number[]) => setTop(values[0])} />
			</FormField>
			<FormField label={t('shared/modifications/crop/crop___left')} labelFor="left">
				<MultiRange min={0} max={width} step={1} id="left" values={[left]}
										onChange={(values: number[]) => setLeft(values[0])} />
			</FormField>
			<FormField label={t('shared/modifications/crop/crop___right')} labelFor="right">
				<MultiRange min={0} max={width} step={1} id="right" values={[right]}
										onChange={(values: number[]) => setRight(values[0])} />
			</FormField>
			<FormField label={t('shared/modifications/crop/crop___bottom')} labelFor="bottom">
				<MultiRange min={0} max={height} step={1} id="bottom" values={[bottom]}
										onChange={(values: number[]) => setBottom(values[0])} />
			</FormField>
		</>
	);
};

export default Crop;
