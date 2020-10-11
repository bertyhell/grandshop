/// <reference path="../../node_modules/electron/electron.d.ts" />
/// <reference path="../../node_modules/@types/fs-extra/index.d.ts" />

// Modules to control application life and create native browser window
import {
	ElectronCallFunctionRequest,
	RendererCallFunction,
	RendererCallFunctionRequest,
} from '../shared/types';
import path, { ParsedPath } from 'path';
import { app, BrowserWindow, dialog, ipcMain } from 'electron';
import fs from 'fs-extra';
import isDev from 'electron-is-dev';
import throttle from '@jcoreio/async-throttle';

import { ImageModificationInfo } from '../shared/modifications/types';
import i18n from '../shared/translations/i18n';

import ImageManager from './image-manager';
import { CustomError } from '../shared/error';
import { ColorPicker } from './color-picker';

export interface Dimensions {
	width: number;
	height: number;
}

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
			nodeIntegration: true,
			webSecurity: false,
		},
		backgroundColor: '#888888',
		show: false,
	});

	mainWindow.once('ready-to-show', () => {
		mainWindow.maximize();
		mainWindow.show();
	});

	if (isDev) {
		mainWindow.loadURL(
			isDev ? 'http://localhost:3000' : `file://${app.getAppPath()}/build/index.html`
		);
	} else {
		mainWindow.setMenu(null);
		mainWindow.loadFile(path.join(app.getAppPath(), '/build/index.html'));
	}

	// Open the DevTools.
	// mainWindow.webContents.openDevTools();

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

async function openImage(
	event: Electron.IpcMainInvokeEvent,
	options: Electron.OpenDialogSyncOptions
) {
	let response: Electron.OpenDialogReturnValue | undefined = undefined;
	try {
		response = await dialog.showOpenDialog(mainWindow, options);
		if (response.canceled || !response.filePaths || !response.filePaths.length) {
			return;
		}
		// Store the original image path in the image manager
		const originalImageInfo: {
			buffer: Buffer;
			dimensions: Dimensions;
		} = await ImageManager.setOriginalImagePath(response.filePaths[0]); // TODO add multi image support

		// Pass the original image as a base64 string to the client for preview
		callRendererFunction('setOriginalImage', [
			originalImageInfo.buffer.toString('base64'),
			originalImageInfo.dimensions,
		]);
	} catch (err) {
		const error = new CustomError('Failed to open file', err, {
			dialogOptions: options,
			dialogResponse: response,
		});
		callRendererFunction('showNotification', [
			i18n.t('electron/main___het-openen-van-de-afbeelding-is-mislukt'),
			'error',
			error,
		]);
		console.error(error);
	}
}

async function saveImage(event: Electron.IpcMainInvokeEvent, options: Electron.SaveDialogOptions) {
	let response: Electron.SaveDialogReturnValue | undefined = undefined;
	try {
		const parts: ParsedPath = ImageManager.getOriginalImagePathParts();
		const defaultFileName =
			parts.name + '-' + new Date().toLocaleDateString().replace(/[^0-9]+/g, '-') + '.png';
		response = await dialog.showSaveDialog(mainWindow, {
			...options,
			defaultPath: defaultFileName,
		});
		if (response.canceled || !response.filePath) {
			return;
		} // TODO allow the user to enter a different extension, then use gm to save the file in that extension
		// Write the modified buffer image to the save location
		await fs.writeFile(response.filePath, ImageManager.getModifiedImageBuffer());

		callRendererFunction('showNotification', [
			i18n.t('electron/main___image-has-been-saved'),
			'success',
		]);
	} catch (err) {
		const error = new CustomError('Failed to save file', err, {
			dialogOptions: options,
			dialogResponse: response,
		});
		callRendererFunction('showNotification', [
			i18n.t('electron/main___saving-the-image-failed'),
			'error',
			error,
		]);
		console.error(error);
	}
}

async function setLanguage(event: Electron.IpcMainInvokeEvent, language: string) {
	try {
		await i18n.changeLanguage(language);
	} catch (err) {
		throw new CustomError('Failed to change the language', err, { language });
	}
}

async function getColorAtPixel(
	event: Electron.IpcMainInvokeEvent,
	x: number,
	y: number
): Promise<string> {
	return ColorPicker.getColorAtMouse(mainWindow, x, y);
}

function callRendererFunction(func: RendererCallFunction, funcArgs: any[]) {
	mainWindow.webContents.send('mainprocess-response', {
		func,
		funcArgs,
	} as RendererCallFunctionRequest);
}

async function getImageBorders(): Promise<string[]> {
	const bordersFolder = path.join(
		path.dirname(__dirname),
		'../resources_external/modifiers/AddImageBorder/borders'
	);
	console.log('path: ', bordersFolder);

	const borderFileNames = await fs.readdir(bordersFolder);

	console.log('path: ', bordersFolder, borderFileNames);

	return borderFileNames;
}

const applyModifications = throttle(
	async (event: Electron.IpcMainInvokeEvent, modifications: ImageModificationInfo[]) => {
		try {
			callRendererFunction('setModificationInProgress', [true]);
			const modifiedImageInfo: {
				buffer: Buffer;
				dimensions: Dimensions;
			} = await ImageManager.updateModifications(modifications);
			callRendererFunction('setModifiedImage', [
				modifiedImageInfo.buffer.toString('base64'),
				modifiedImageInfo.dimensions,
			]);
			callRendererFunction('setModificationInProgress', [false]);
		} catch (err) {
			const error = new CustomError('Failed to apply modifications to image', err, {
				modifications,
			});
			callRendererFunction('showNotification', [
				i18n.t('electron/main___applying-the-last-operation-failed'),
				'error',
				error,
			]);
			console.error(error);
		}
	},
	0
);

const callableFunctions = {
	openImage,
	saveImage,
	applyModifications,
	setLanguage,
	getColorAtPixel,
	getImageBorders,
};

ipcMain.handle(
	'request-mainprocess-action',
	async (event: Electron.IpcMainInvokeEvent, arg: ElectronCallFunctionRequest) => {
		try {
			const func: Function | undefined = callableFunctions[arg.func];
			if (!func) {
				throw new CustomError(
					"Received func request on request-mainprocess-action, but function doesn't exist",
					null,
					{
						event,
						arg,
					}
				);
			}
			return await func(event, ...arg.funcArgs);
		} catch (err) {
			callRendererFunction('showNotification', [i18n.t(err.message), 'error', err]);
			console.error(err);
		}
	}
);
