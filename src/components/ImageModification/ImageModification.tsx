import React, { FunctionComponent, ReactNode } from 'react';
import classnames from 'classnames';
import { Accordion } from '../Accordion/Accordion';

import './ImageModification.scss';

interface ImageModificationProps {
	className?: string;
	label?: string;
	children?: ReactNode;
	isOpen: boolean;
	onToggle: () => void;
	onDelete: () => void;
}

const ImageModification: FunctionComponent<ImageModificationProps> = ({
	className,
	label,
	children = null,
	isOpen,
	onToggle,
	onDelete,
}) => {
	return (
		<Accordion
			title={label}
			className={classnames(className, 'o-image-modification')}
			isOpen={isOpen}
			onToggle={onToggle}
			onDelete={onDelete}
		>
			{children && <div className="o-image-modification-controls">{children}</div>}
		</Accordion>
	);
};

export default ImageModification;
