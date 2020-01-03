/// <reference path="../node_modules/electron/electron.d.ts" />
/// <reference path="../node_modules/@types/fs-extra/index.d.ts" />

// Modules to control application life and create native browser window
import { ElectronCallFunctionRequest, ImageModificationInfo, RendererCallFunction, RendererCallFunctionRequest } from '../src/shared/types';
import { ParsedPath } from 'path';

import ImageManager from './image-manager';
import CustomError from './helpers/error';

import { app, BrowserWindow, dialog, ipcMain } from 'electron';
import path from 'path';
import fs from 'fs-extra';

import isDev from 'electron-is-dev';

const appFolder: string = path.resolve(__dirname, '..');
// const windowStateFileName: string = path.resolve(appFolder, 'last-window-state.json');
require('electron-reload')(appFolder);

// Keep a global reference of the window object, if you don't, the window will
// be closed automatically when the JavaScript object is garbage collected.
let mainWindow: any;

function createWindow() {
  // Create the browser window.
  // let data: any;
  // try {
  //   data = JSON.parse(fs.readFileSync(windowStateFileName, 'utf8'));
  // } catch (e) {
  // }
  mainWindow = new BrowserWindow({
    // ...((data && data.bounds) ? data.bounds : { width: 800, height: 600 }),
    webPreferences: {
      preload: path.resolve(__dirname, 'preload.js'),
      nodeIntegration: true,
      webSecurity: false
    }
  });
  mainWindow.maximize();
  if (!isDev) {
    mainWindow.setMenu(null);
  }

  mainWindow.loadURL(
      isDev ?
          'http://localhost:3000' :
          `file://${path.resolve(__dirname, '../build/index.html')}`
  );


  // Open the DevTools.
  // mainWindow.webContents.openDevTools()

  // Emitted when the window is closed.
  mainWindow.on('closed', async () => {
    // Store the last window position and dimensions
    // fs.writeFileSync(windowStateFileName, JSON.stringify({
    //   bounds: mainWindow.getBounds()
    // }));

    // Dereference the window object, usually you would store windows
    // in an array if your app supports multi windows, this is the time
    // when you should delete the corresponding element.
    mainWindow = null;
  });
}

// This method will be called when Electron has finished
// initialization and is ready to create browser windows.
// Some APIs can only be used after this event occurs.
app.on('ready', createWindow);

// Quit when all windows are closed.
app.on('window-all-closed', () => {
  // On macOS it is common for applications and their menu bar
  // to stay active until the user quits explicitly with Cmd + Q
  if (process.platform !== 'darwin') {
    app.quit();
  }
});

app.on('activate', () => {
  // On macOS it's common to re-create a window in the app when the
  // dock icon is clicked and there are no other windows open.
  if (mainWindow === null) {
    createWindow();
  }
});

async function openImage(event: Electron.IpcMainEvent, options: Electron.OpenDialogSyncOptions) {
  let response: Electron.OpenDialogReturnValue | undefined = undefined;
  try {
    response = await dialog.showOpenDialog(mainWindow, options);
    if (response.canceled || !response.filePaths || !response.filePaths.length) {
      return;
    }
    // Store the original image path in the image manager
    const originalImageBuffer: Buffer = await ImageManager.setOriginalImagePath(response.filePaths[0]); // TODO add multi image support

    // Pass the original image as a base64 string to the client for preview
    callRendererFunction(event, 'setOriginalImage', [originalImageBuffer.toString('base64')]);
  } catch (err) {
    const error = new CustomError('Failed to open file', err, { dialogOptions: options, dialogResponse: response });
    callRendererFunction(event, 'showNotification', ['Het openen van de afbeelding is mislukt', 'error', error]);
    console.error(error);
  }
}

async function saveImage(event: Electron.IpcMainEvent, options: Electron.SaveDialogOptions) {
  let response: Electron.SaveDialogReturnValue | undefined = undefined;
  try {
    const parts: ParsedPath = ImageManager.getOriginalImagePathParts();
    const defaultFileName = parts.name + '-' + new Date().toLocaleDateString().replace(/[^0-9]+/g, '-') + parts.ext;
    response = await dialog.showSaveDialog(mainWindow, { ...options, defaultPath: defaultFileName });
    if (response.canceled || !response.filePath) {
      return;
    }// TODO allow the user to enter a different extension, then use gm to save the file in that extension
    // Write the modified buffer image to the save location
    await fs.writeFile(response.filePath, ImageManager.getModifiedImageBuffer());

    callRendererFunction(event, 'showNotification', ['Afbeelding is opgeslagen', 'success']);
  } catch (err) {
    const error = new CustomError('Failed to save file', err, { dialogOptions: options, dialogResponse: response });
    callRendererFunction(event, 'showNotification', ['Het opslaan van de afbeelding is mislukt', 'error', error]);
    console.error(error);
  }
}

function callRendererFunction(event: Electron.IpcMainEvent, func: RendererCallFunction, funcArgs: any[]) {
  event.sender.send('mainprocess-response', { func, funcArgs } as RendererCallFunctionRequest)
}

async function applyModifications(event: Electron.IpcMainEvent, modifications: ImageModificationInfo[]) {
  try {
    const modifiedImageBuffer: Buffer = await ImageManager.updateModifications(modifications);
    callRendererFunction(event, 'setModifiedImage', [modifiedImageBuffer.toString('base64')]);
  } catch (err) {
    const error = new CustomError('Failed to apply modifications to image', err, { modifications });
    callRendererFunction(event, 'showNotification', ['Het toepassen van de laatste bewerking is mislukt', 'error', error]);
    console.error(error);
  }
}

const callableFunctions = {
  openImage,
  saveImage,
  applyModifications
};

ipcMain.on('request-mainprocess-action', (event: Electron.IpcMainEvent, arg: ElectronCallFunctionRequest) => {
  const func: Function | undefined = callableFunctions[arg.func];
  if (func) {
    try {
      func(event, ...arg.funcArgs);
    } catch (err) {
      console.error('Failed to execute main function that was requested from the renderer thread', { event, arg, func })
    }
  } else {
    console.error('Received func request on request-mainprocess-action, but function doesn\'t exist1', { event, arg });
  }
});

