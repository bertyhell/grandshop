/// <reference path="../../node_modules/electron/electron.d.ts" />
import { setModifiedImageUrlAction, setOriginalImageUrlAction } from '../store/actions';
import store from '../store/store';

const { ipcRenderer } = (window as any).require('electron');

interface CallFunctionRequest {
  func: 'applyModifications' | 'selectOriginalImage',
  funcArgs: any[];
}

export interface ImageModificationInfo {
  name: 'addBorder' | 'sepia';
  arguments: any;
}

const callableFunctions = {
  setOriginalImageUrl: (url: string) => {
    store.dispatch(setOriginalImageUrlAction(url));
    store.dispatch(setModifiedImageUrlAction(null));
  },
  setModifiedImageUrl: (url: string) => {
    store.dispatch(setModifiedImageUrlAction(url));
  },
  showNotification: (message: string, status: 'success' | 'warn' | 'error') => {
    // TODO show notification toasts
  },
};

ipcRenderer.on('mainprocess-response', (event: any, arg: CallFunctionRequest) => {
  const func: Function | undefined = (callableFunctions as any)[arg.func];
  if (func) {
    try {
      func(...arg.funcArgs);
    } catch (err) {
      console.error('Failed to execute main function that was requested from the renderer thread', { event, arg, func })
    }
  } else {
    console.error('Received func request on request-mainprocess-action, but function doesn\'t exist', { event, arg });
  }
});

function callMainFunction(name: string, args: any[]) {
  ipcRenderer.send('request-mainprocess-action', {
    func: name,
    funcArgs: args
  } as CallFunctionRequest);
}

export function selectOriginalImage(options: Electron.OpenDialogSyncOptions = {}) {
  callMainFunction('selectOriginalImage', [options]);
}

export function saveImage(options: Electron.SaveDialogOptions = {}) {
  callMainFunction('saveImage', [options]);
}

export function applyModifications(modifications: ImageModificationInfo[]) {
  callMainFunction('applyModifications', [modifications]);
}
