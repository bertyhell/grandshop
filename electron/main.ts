/// <reference path="../node_modules/electron/electron.d.ts" />
/// <reference path="../node_modules/@types/fs-extra/index.d.ts" />

// Modules to control application life and create native browser window
const ImageManager = require('./image-manager').default;
const CustomError = require('./helpers/error').default;

const { app, BrowserWindow, dialog, ipcMain } = require('electron');
const path = require('path');
const fs = require('fs-extra');

const isDev = require('electron-is-dev');

const appFolder: string = path.resolve(__dirname, '..');
const windowStateFileName: string = path.resolve(appFolder, 'last-window-state.json');
require('electron-reload')(appFolder, {
  ignored: [/node_modules|[/\\]\./, /tmp/]
});

interface ImageModificationInfo {
  name: 'addBorder'; // TODO add more modifications
  arguments: any;
}

// Keep a global reference of the window object, if you don't, the window will
// be closed automatically when the JavaScript object is garbage collected.
let mainWindow: any;

function createWindow() {
  // Create the browser window.
  let data: any;
  try {
    data = JSON.parse(fs.readFileSync(windowStateFileName, 'utf8'));
  } catch (e) {
  }
  mainWindow = new BrowserWindow({
    ...((data && data.bounds) ? data.bounds : { width: 800, height: 600 }),
    webPreferences: {
      preload: path.resolve(__dirname, 'preload.js'),
      nodeIntegration: true,
      webSecurity: false
    }
  });

  // mainWindow.webContents.openDevTools();

  mainWindow.loadURL(
      isDev ? 'http://localhost:3000' : `file://${path.resolve(__dirname, '../build/index.html')}`
  );
  // // and load the index.html of the app.
  // mainWindow.loadFile('public/index.html');

  // Open the DevTools.
  // mainWindow.webContents.openDevTools()

  // Emitted when the window is closed.
  mainWindow.on('closed', () => {
    var data = {
      bounds: mainWindow.getBounds()
    };
    fs.writeFileSync(windowStateFileName, JSON.stringify(data));
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

async function selectOriginalImage(event: Electron.IpcMainEvent, options: Electron.OpenDialogSyncOptions) {
  const response: Electron.OpenDialogReturnValue = await dialog.showOpenDialog(mainWindow, options);
  if (response.canceled || !response.filePaths || !response.filePaths.length) {
    return;
  }
  // Store the original image path in the image manager
  ImageManager.setOriginalImagePath(response.filePaths[0]); // TODO add multi image support

  // Pass the original image path to the client for preview
  const relativeUrl = convertPathToUrl(response.filePaths[0]);
  event.sender.send('mainprocess-response', {
    func: 'setOriginalImageUrl',
    funcArgs: [relativeUrl]
  } as RendererCallFunctionRequest)
}

async function saveImage(event: Electron.IpcMainEvent, options: Electron.SaveDialogOptions) {
  let response: Electron.SaveDialogReturnValue | undefined = undefined;
  try {
    response = await dialog.showSaveDialog(mainWindow, options);
    if (response.canceled || !response.filePath) {
      return;
    }// TODO allow the user to enter a different extension, then use gm to save the file in that extension
    // Copy the last modified image to the desired location
    await fs.copyFile(ImageManager.getModifiedImagePath(), response.filePath);

    event.sender.send('mainprocess-response', {
      func: 'showNotification',
      funcArgs: ['Afbeelding is opgeslagen', 'success']
    } as RendererCallFunctionRequest)
  } catch (err) {
    console.error(new CustomError('Failed to save file', err, { dialogOptions: options, dialogResponse: response }))
  }
}

function convertPathToUrl(filePath: string) {
  if (!filePath) {
    return filePath;
  }
  return filePath.replace(/\\+/g, '/');
}

async function applyModifications(event: Electron.IpcMainEvent, modifications: ImageModificationInfo[]) {
  const modifiedImageUrl = await ImageManager.updateModifications(modifications);
  event.sender.send('mainprocess-response', {
    func: 'setModifiedImageUrl',
    funcArgs: [convertPathToUrl(modifiedImageUrl)]
  } as RendererCallFunctionRequest)
}

const callableFunctions = {
  selectOriginalImage,
  saveImage,
  applyModifications
};

interface RendererCallFunctionRequest {
  func: 'setOriginalImageUrl' | 'setModifiedImageUrl' | 'showNotification',
  funcArgs: any[];
}

interface ElectronCallFunctionRequest {
  func: 'selectOriginalImage' | 'applyModifications',
  funcArgs: any[];
}

ipcMain.on('request-mainprocess-action', (event: Electron.IpcMainEvent, arg: ElectronCallFunctionRequest) => {
  const func: Function | undefined = (callableFunctions as any)[arg.func];
  if (func) {
    try {
      func(event, ...arg.funcArgs);
    } catch (err) {
      console.error('Failed to execute main function that was requested from the renderer thread', { event, arg, func })
    }
  } else {
    console.error('Received func request on request-mainprocess-action, but function doesn\'t exist', { event, arg });
  }
});

