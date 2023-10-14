import React, { FunctionComponent, ReactNode } from 'react';

import clsx from 'clsx';

import './ToolbarTitle.scss';

export interface ToolbarTitleProps {
	children: ReactNode;
	className?: string;
}

export const ToolbarTitle: FunctionComponent<ToolbarTitleProps> = ({ children, className }) => {
	return <h2 className={clsx(className, 'c-toolbar__title')}>{children}</h2>;
};
