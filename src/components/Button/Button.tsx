import React, { FunctionComponent, MouseEvent, ReactNode } from 'react';

import classnames from 'classnames';

import './Button.scss';

export interface ButtonProps {
  className?: string;
  active?: boolean;
  ariaLabel?: string;
  children?: ReactNode;
  disabled?: boolean;
  label?: string;
  block?: boolean;

  onClick?(event: MouseEvent<HTMLElement>): void;

  title?: string;
  type?:
      | 'primary'
      | 'secondary';
}

const Button: FunctionComponent<ButtonProps> = ({
                                                  ariaLabel,
                                                  children,
                                                  className,
                                                  disabled,
                                                  label,
                                                  block = true,
                                                  onClick,
                                                  title,
                                                  type = 'primary'
                                                }) => (
    <button
        className={classnames(className, 'c-button', {
          [`c-button-${type}`]: type,
          [`c-button-block`]: block,
        })}
        onClick={disabled ? () => {
        } : onClick}
        disabled={disabled}
        aria-label={ariaLabel}
        title={title}
    >
      {children ? (
          children
      ) : (
          <div className="c-button__content">
            {label && <div className="c-button__label">{label}</div>}
          </div>
      )}
    </button>
);

export default Button;
