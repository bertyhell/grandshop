import React, { FunctionComponent } from 'react';

import './Toggle.scss';

interface ToggleProps {
	className?: string;
	label: string;
	offLabel: string;
	onLabel: string;
	isChecked: boolean;
	onChange: (isChecked: boolean) => void;
}

export const Toggle: FunctionComponent<ToggleProps> = ({
	className,
	label,
	offLabel,
	onLabel,
	isChecked,
	onChange = () => {},
}) => {
	return (
		<div className={'c-switch' + (className ? ' ' + className : '')}>
			<div>{label}</div>
			<div>{offLabel}</div>
			<label className="c-switch__toggle">
				<input type="checkbox" checked={isChecked} onChange={() => onChange(!isChecked)} />
				<div className="c-switch__toggle__slider"></div>
			</label>

			<div>{onLabel}</div>
		</div>
	);
};
