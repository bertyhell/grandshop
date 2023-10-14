import { FunctionComponent, useEffect } from 'react';

import { ModificationProps } from '../types';

import './Sepia.scss';

const Sepia: FunctionComponent<ModificationProps> = ({ onApply }) => {
	useEffect(() => {
		onApply({
			name: 'sepia',
			arguments: [],
		});
	}, []);
	return null;
};

export default Sepia;
