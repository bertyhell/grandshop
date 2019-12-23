import { isNil } from 'lodash-es';
import React, { FunctionComponent, ReactNode } from 'react';
import { toast, ToastOptions } from 'react-toastify';
import Alert, { AlertProps } from '../components/Alert/Alert';
import { CustomError } from '../shared/error';

export enum ToastType {
  ERROR = 'error',
  WARN = 'warn',
  INFO = 'info',
  SUCCESS = 'success',
}

type AlertMessage = string | string[] | ReactNode;
type ToastServiceFn = (message: AlertMessage, options?: ToastOptions & { innerException?: CustomError }) => void;
type ToastService = { [type in ToastType]: ToastServiceFn };

interface ToastProps extends AlertProps {
}

// const Toast: FunctionComponent<ToastProps> = ({ ...rest }) => (
//     <Alert {...rest} className="u-spacer-top"/>
// );

function showToast(
    alertMessage: AlertMessage,
    options: ToastOptions & { innerException?: CustomError } = {},
    alertType: ToastType = ToastType.INFO
) {
  if (isNil(alertMessage)) {
    return;
  }

  // let message = alertMessage;
  //
  // if (Array.isArray(alertMessage)) {
  //   const messages = alertMessage as string[];
  //   message = (
  //       <div>
  //         {messages.map((message: string) => ({ message }))}
  //       </div>
  //   );
  // }

  const { innerException, ...toastOptions } = options;
  if (innerException) {
    console.error('Error in the main tread: ', innerException);
  }
  // toast(<Toast message={message} type={alertType}/>, { autoClose: 7000, ...(toastOptions || {}) });
  toast[alertType](alertMessage);
}

const toastService: ToastService = {
  error: (message, options) => showToast(message, options, ToastType.ERROR),
  warn: (message, options) => showToast(message, options, ToastType.WARN),
  info: (message, options) => showToast(message, options, ToastType.INFO),
  success: (message, options) => showToast(message, options, ToastType.SUCCESS)
};

export default toastService;
