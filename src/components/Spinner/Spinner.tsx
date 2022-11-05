import React, { FunctionComponent } from 'react';

import classnames from 'classnames';

import './Spinner.scss';

export interface SpinnerProps {
	className?: string;
}

const Spinner: FunctionComponent<SpinnerProps> = ({ className }) => {
	return (
		<div className={classnames('o-spinner', 'sk-cube-grid', className)}>
			<div className="sk-cube sk-cube1" />
			<div className="sk-cube sk-cube2" />
			<div className="sk-cube sk-cube3" />
			<div className="sk-cube sk-cube4" />
			<div className="sk-cube sk-cube5" />
			<div className="sk-cube sk-cube6" />
			<div className="sk-cube sk-cube7" />
			<div className="sk-cube sk-cube8" />
			<div className="sk-cube sk-cube9" />
		</div>
	);
};

export default Spinner;
