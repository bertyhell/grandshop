/// <reference path="../../node_modules/electron/electron.d.ts" />
import { setModifiedImageAction, setOriginalImageAction } from '../store/actions';
import store from '../store/store';
import { ElectronCallFunction, ElectronCallFunctionRequest, ImageModificationInfo, RendererCallFunctionRequest } from '../shared/types';
import toastService from '../services/toast-service';
import { AlertType } from '../components/Alert/Alert';
import { CustomError } from '../shared/error';

const { ipcRenderer } = (window as any).require('electron');

const callableFunctions = {
  setOriginalImage: (url: string) => {
    store.dispatch(setOriginalImageAction(url));
    store.dispatch(setModifiedImageAction(null));
  },
  setModifiedImage: (url: string) => {
    store.dispatch(setModifiedImageAction(url));
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

export function openImage(options: Electron.OpenDialogSyncOptions = {}) {
  callMainFunction('openImage', [options]);
}

export function saveImage(options: Electron.SaveDialogOptions = {}) {
  callMainFunction('saveImage', [options]);
}

export function applyModifications(modifications: ImageModificationInfo[]) {
  callMainFunction('applyModifications', [modifications]);
}
