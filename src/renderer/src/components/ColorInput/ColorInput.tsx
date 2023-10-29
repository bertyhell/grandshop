import React, {
	createRef,
	FunctionComponent,
	MutableRefObject,
	ReactNode,
	RefObject,
	useCallback,
	useEffect,
	useRef,
} from 'react';
import iro from '@jaames/iro';
import clsx from 'clsx';

import { getColorAtPixel } from '../../services/interop-electron';

import './ColorInput.scss';

interface IroColorPicker {
	on: (action: string, callback: () => void) => void;
	color: {
		hexString: string;
		set: (value: string) => void;
		setState: (state: unknown) => void;
	};
}

export interface ColorInputProps {
	children?: ReactNode;
	className?: string;
	value?: string;
	onChange?: (color: string) => void;
}

const ColorInput: FunctionComponent<ColorInputProps> = ({
	className,
	value = '#850000',
	onChange = () => {},
}) => {
	const colorPicker: MutableRefObject<IroColorPicker | null> = useRef<IroColorPicker | null>(
		null
	);
	const el: RefObject<HTMLDivElement> = createRef<HTMLDivElement>();

	const changeColor = useCallback(
		(color: string) => {
			if (!color) {
				return;
			}
			onChange(color);
			if (!colorPicker.current) {
				return;
			}
			colorPicker.current.color.set(color);
		},
		[onChange]
	);

	const copyMouseColorToState = useCallback(
		async (evt: MouseEvent) => {
			const color = await getColorAtPixel(evt.clientX, evt.clientY);
			changeColor(color);
		},
		[changeColor]
	);

	useEffect(() => {
		if (!el.current) {
			return;
		}
		if (!colorPicker.current) {
			// create a new iro color picker and pass component props to it
			// eslint-disable-next-line @typescript-eslint/no-explicit-any
			colorPicker.current = new (iro.ColorPicker as any)(el.current, { color: value });
			// call onColorChange prop whenever the color changes
			if (!colorPicker.current) {
				return;
			}
			colorPicker.current.on('color:change', () => {
				changeColor((colorPicker.current as IroColorPicker).color.hexString);
			});
		}
		const img: HTMLImageElement | null = document.querySelector('.viewer-canvas img');
		if (!img) {
			return;
		}
		img.addEventListener('click', copyMouseColorToState);

		// Destructor
		return () => {
			if (img) {
				img.removeEventListener('click', copyMouseColorToState);
			}
		};
	}, [el, value, changeColor, copyMouseColorToState]);

	return (
		<div className={clsx('o-color-input', className)}>
			<div ref={el} />
			<div className="a-color-preview" style={{ backgroundColor: value }} />
		</div>
	);
};

export default ColorInput;
