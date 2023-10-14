import React, { Fragment, FunctionComponent, MouseEvent, ReactNode } from 'react';

import clsx from 'clsx';
import ReactDOM from 'react-dom';

import { useKeyPress } from '../../hooks/useKeyPress';

import { Button } from '../Button/Button';
import { Toolbar } from '../Toolbar/Toolbar';
import { ToolbarLeft, ToolbarRight } from '../Toolbar/Toolbar.slots';
import { ToolbarItem } from '../Toolbar/ToolbarItem/ToolbarItem';

import { ModalBackdrop } from './ModalBackdrop';

import './Modal.scss';
import { ButtonToolbar } from '../ButtonToolbar/ButtonToolbar';

export interface ModalProps {
	children: ReactNode;
	disableContextClick?: boolean;
	isOpen: boolean;
	title?: string;
	size?: 'small' | 'medium' | 'large' | 'extra-large' | 'fullscreen' | 'fullwidth' | 'auto';
	scrollable?: boolean;
	onClose?: () => void;
	className?: string;
	headerRight?: ReactNode;
	footerRight?: ReactNode;
	footerLeft?: ReactNode;
}

export const Modal: FunctionComponent<ModalProps> = ({
	children,
	className,
	disableContextClick = false,
	isOpen,
	title,
	size,
	scrollable,
	onClose,
	headerRight,
	footerRight,
	footerLeft,
}) => {
	useKeyPress('Escape', close);

	function close() {
		onClose && onClose();
	}

	function onContextClick(event: MouseEvent<HTMLElement>) {
		// close the modal when clicking outside the modal
		if (!disableContextClick && event.target === event.currentTarget) {
			close();
		}
	}

	return ReactDOM.createPortal(
		<Fragment>
			<div
				className={clsx(className, 'c-modal-context', {
					'c-modal-context--visible': isOpen,
				})}
				onClick={onContextClick}
			>
				<div
					className={clsx('c-modal', {
						'c-modal--small': size === 'small',
						'c-modal--medium': size === 'medium',
						'c-modal--large': size === 'large',
						'c-modal--extra-large': size === 'extra-large',
						'c-modal--fullscreen': size === 'fullscreen',
						'c-modal--fullwidth': size === 'fullwidth',
						'c-modal--height-auto': size === 'auto',
						'c-modal--scrollable': scrollable,
					})}
				>
					{(!!title || !!headerRight || !!onClose) && (
						<div className="c-modal__header c-modal__header--bordered">
							<Toolbar autoHeight spaced>
								{title && (
									<ToolbarLeft>
										<ToolbarItem>
											<h2
												className="c-modal__title"
												dangerouslySetInnerHTML={{ __html: title }}
											/>
										</ToolbarItem>
									</ToolbarLeft>
								)}
								<ToolbarRight>
									{headerRight && <ToolbarItem>{headerRight}</ToolbarItem>}
									{!!onClose && (
										<ToolbarItem>
											<Button
												onClick={close}
												icon="fa-times"
												type="primary"
												ariaLabel="close modal"
											/>
										</ToolbarItem>
									)}
								</ToolbarRight>
							</Toolbar>
						</div>
					)}
					<div className="c-modal__body">{children}</div>
					{(footerLeft || footerRight) && (
						<div className="c-modal__footer c-modal__footer--bordered">
							<Toolbar spaced>
								{footerLeft && (
									<ToolbarLeft>
										<ToolbarItem>
											<ButtonToolbar>{footerLeft}</ButtonToolbar>
										</ToolbarItem>
									</ToolbarLeft>
								)}
								{footerRight && (
									<ToolbarRight>
										<ToolbarItem>
											<ButtonToolbar>{footerRight}</ButtonToolbar>
										</ToolbarItem>
									</ToolbarRight>
								)}
							</Toolbar>
						</div>
					)}
				</div>
			</div>
			<ModalBackdrop visible={isOpen} />
		</Fragment>,
		document.body
	);
};
