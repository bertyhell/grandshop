import { FunctionComponent, useEffect } from 'react';

import { ModificationProps } from '../types';

import './Sepia.scss';

const Sepia: FunctionComponent<ModificationProps> = ({ onApply }) => {
	useEffect(() => {
		onApply({
			name: 'sepia',
			arguments: [],
		});
	}, []); // eslint-disable-line react-hooks/exhaustive-deps
	return null;
};

export default Sepia;
