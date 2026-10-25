import { FunctionComponent, ReactNode } from 'react';

import clsx from 'clsx';

import './ButtonToolbar.scss';

export interface ButtonToolbarProps {
	children: ReactNode;
	className?: string;
}

const ButtonToolbar: FunctionComponent<ButtonToolbarProps> = ({ className = '', children }) => (
	<div className={clsx(className, 'c-button-toolbar')}>{children}</div>
);

export { ButtonToolbar };
