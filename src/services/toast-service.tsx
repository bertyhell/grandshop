import { ReactNode } from 'react';
import { isNil } from 'lodash-es';
import { toast, ToastOptions } from 'react-toastify';
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

function showToast(
    alertMessage: AlertMessage,
    options: ToastOptions & { innerException?: CustomError } = {},
    alertType: ToastType = ToastType.INFO
) {
  if (isNil(alertMessage)) {
    return;
  }

  const { innerException } = options;
  if (innerException) {
    console.error('Error in the main tread: ', innerException);
  }
  toast[alertType](alertMessage);
}

const toastService: ToastService = {
  error: (message, options) => showToast(message, options, ToastType.ERROR),
  warn: (message, options) => showToast(message, options, ToastType.WARN),
  info: (message, options) => showToast(message, options, ToastType.INFO),
  success: (message, options) => showToast(message, options, ToastType.SUCCESS)
};

export default toastService;
