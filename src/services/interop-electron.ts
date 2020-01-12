import { OpenDialogSyncOptions, SaveDialogOptions } from 'electron';
import { Dimensions } from 'gm';

import { setModificationInProgressAction, setModifiedImageAction, setOriginalImageAction } from '../store/actions';
import store from '../store/store';
import { ElectronCallFunction, ElectronCallFunctionRequest, RendererCallFunctionRequest } from '../shared/types';
import toastService from './toast-service';
import { AlertType } from '../components/Alert/Alert';
import { CustomError } from '../shared/error';
import { ImageModificationInfo } from '../shared/modifications/types';

const { ipcRenderer } = (window as any).require('electron');

const callableFunctions = {
  setOriginalImage: (base64: string, dimensions: Dimensions) => {
    store.dispatch(setOriginalImageAction({
      base64,
      dimensions,
    }));
    store.dispatch(setModifiedImageAction(null));
  },
  setModifiedImage: (base64: string, dimensions: Dimensions) => {
    store.dispatch(setModifiedImageAction({
      base64,
      dimensions,
    }));
  },
  setModificationInProgress: (inProgress: boolean) => {
    store.dispatch(setModificationInProgressAction(inProgress));
  },
  showNotification: (message: string, status: AlertType, innerException: CustomError) => {
    (toastService[status] || toastService['info'])(message, { innerException });
  }
};

ipcRenderer.on('mainprocess-response', (event: any, arg: RendererCallFunctionRequest) => {
  const func: Function | undefined = (callableFunctions as any)[arg.func];
  if (func) {
    try {
      func(...arg.funcArgs);
    } catch (err) {
      console.error('Failed to execute renderer function that was requested from the main thread', { event, arg, func })
    }
  } else {
    console.error('Received func request on mainprocess-response, but function doesn\'t exist', { event, arg });
  }
});

function callMainFunction(name: ElectronCallFunction, args: any[]) {
  ipcRenderer.send('request-mainprocess-action', {
    func: name,
    funcArgs: args
  } as ElectronCallFunctionRequest);
}

export function openImage(options: OpenDialogSyncOptions = {}) {
  callMainFunction('openImage', [options]);
}

export function saveImage(options: SaveDialogOptions = {}) {
  callMainFunction('saveImage', [options]);
}

export function applyModifications(modifications: ImageModificationInfo[]) {
  callMainFunction('applyModifications', [modifications]);
}
