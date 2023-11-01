import React, { FunctionComponent, useEffect, useState } from 'react';

import { MultiRange } from '../../../renderer/src/components/MultiRange/MultiRange';
import { ModificationProps } from '../types';

import './Rotate.scss';
import ColorInput from '../../../renderer/src/components/ColorInput/ColorInput';

const Rotate: FunctionComponent<ModificationProps> = ({ onApply }) => {
	const [angle, setAngle] = useState<number>(0);
	const [backgroundColor, setBackgroundColor] = useState<string>('#FFFFFF');

	useEffect(() => {
		onApply({
			name: 'rotate',
			arguments: [angle, backgroundColor],
		});
	}, [angle, backgroundColor]);

	return (
		<>
			<MultiRange
				min={-360}
				max={360}
				step={0.01}
				values={[angle]}
				onChange={(values: number[]) => setAngle(values[0])}
			/>
			<ColorInput value={backgroundColor} onChange={setBackgroundColor}></ColorInput>
		</>
	);
};

export default Rotate;
