import React, { FunctionComponent, ReactNode } from 'react';

import classnames from 'classnames';

import './Alert.scss';

export type AlertType = 'success' | 'warn' | 'error' | 'info';

export interface AlertProps {
  className?: string;
  message?: ReactNode | string;
  type?: AlertType;
}

const Alert: FunctionComponent<AlertProps> = ({
                                                       children,
                                                       className,
                                                       message = '',
                                                       type = 'info',
                                                     }) => {
  return (
      <div
          className={classnames(className, 'c-alert', {
            [`c-alert-${type}`]: type,
          })}
      >
        <div className="c-alert-body">
          <div className="c-alert-message">{message || children}</div>
        </div>
      </div>
  );
};

export default Alert;
