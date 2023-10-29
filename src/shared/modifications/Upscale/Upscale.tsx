import { FunctionComponent, useEffect } from 'react';

import { ModificationProps } from '../types';

import './Upscale.scss';

const Upscale: FunctionComponent<ModificationProps> = ({ onApply }) => {
	useEffect(() => {
		onApply({
			name: 'upscale',
			arguments: [],
		});
	}, []);
	return null;
};

export default Upscale;
