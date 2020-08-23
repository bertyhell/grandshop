import React, { FunctionComponent, ReactNode } from 'react';

import classnames from 'classnames';

import './ButtonToolbar.scss';

export interface ButtonToolbarProps {
	children: ReactNode;
	className?: string;
}

const ButtonToolbar: FunctionComponent<ButtonToolbarProps> = ({ className = '', children }) => (
	<div className={classnames(className, 'c-button-toolbar')}>{children}</div>
);

export { ButtonToolbar };
