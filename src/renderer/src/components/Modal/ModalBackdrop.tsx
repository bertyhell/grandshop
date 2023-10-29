import React, { FunctionComponent } from 'react';

import clsx from 'clsx';

export interface ModalBackdropProps {
	visible?: boolean;
}

export const ModalBackdrop: FunctionComponent<ModalBackdropProps> = ({ visible = false }) => (
	<div className={clsx('c-modal-backdrop', { 'c-modal-backdrop--visible': visible })} />
);
