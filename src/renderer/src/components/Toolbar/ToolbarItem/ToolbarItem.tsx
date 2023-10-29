import React, { FunctionComponent, ReactNode } from 'react';

import clsx from 'clsx';

import './ToolbarItem.scss';

export interface ToolbarItemProps {
	children: ReactNode;
	grow?: boolean;
	className?: string;
}

export const ToolbarItem: FunctionComponent<ToolbarItemProps> = ({
	children,
	grow = false,
	className,
}) => {
	return (
		<div className={clsx(className, 'c-toolbar__item', { 'c-toolbar__item--grow': grow })}>
			{children}
		</div>
	);
};
