import { FunctionComponent, useEffect } from 'react';

import { ModificationProps } from '../types';

import './Greyscale.scss';

const Greyscale: FunctionComponent<ModificationProps> = ({ onApply }) => {
	useEffect(() => {
		onApply({
			name: 'greyscale',
			arguments: [],
		});
	}, []);
	return null;
};

export default Greyscale;
