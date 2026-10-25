// Modules to control application life and create native browser window
import {
	Dimensions,
	ElectronCallFunctionRequest,
	FrontendImageInfo,
	RendererCallFunction,
	RendererCallFunctionRequest,
} from '../shared/types';
import { extname, join, ParsedPath } from 'path';
import { app, BrowserWindow, dialog, ipcMain, shell } from 'electron';
import fs from 'fs-extra';
import throttle from '@jcoreio/async-throttle';
import appIcon from '../../resources/icon.ico?asset&asarUnpack';

import type { ImageModificationInfo } from '../shared/modifications/types';
import i18n from '../shared/translations/i18n';

import ImageManager from '../shared/modifications/image-manager';
import { CustomError } from '../shared/error';
import { ColorPicker } from './color-picker';
import { electronApp, is, optimizer } from '@electron-toolkit/utils';
import { ImageInfo } from '../shared/modifications/modifications';

// Keep a global reference of the window object, if you don't, the window will
// be closed automatically when the JavaScript object is garbage collected.
let mainWindow: BrowserWindow | null;

function createWindow(): void {
	// Create the browser window.
	mainWindow = new BrowserWindow({
		// ...((data && data.bounds) ? data.bounds : { width: 800, height: 600 }),
		webPreferences: {
			// The renderer only talks to the main process through the api exposed in the preload script
			sandbox: true,
			preload: join(__dirname, '../preload/index.js'),
		},
		backgroundColor: '#888888',
		show: false,
		icon: appIcon,
	});

	mainWindow.once('ready-to-show', () => {
		mainWindow?.maximize();
		mainWindow?.show();
	});

	mainWindow.webContents.setWindowOpenHandler((details) => {
		shell.openExternal(details.url);
		return { action: 'deny' };
	});

	// Open the DevTools.
	// mainWindow.webContents.openDevTools();

	// HMR for renderer base on electron-vite cli.
	// Load the remote URL for development or the local html file for production.
	if (is.dev && process.env['ELECTRON_RENDERER_URL']) {
		mainWindow.loadURL(process.env['ELECTRON_RENDERER_URL']);
	} else {
		mainWindow.loadFile(join(__dirname, '../renderer/index.html'));
	}

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
app.whenReady().then(() => {
	// Set app user model id for windows
	electronApp.setAppUserModelId('com.gitlab.com.bertyhell.grandshop');

	// Default open or close DevTools by F12 in development
	// and ignore CommandOrControl + R in production.
	// see https://github.com/alex8088/electron-toolkit/tree/master/packages/utils
	app.on('browser-window-created', (_, window) => {
		optimizer.watchWindowShortcuts(window);
	});

	createWindow();

	app.on('activate', function () {
		// On macOS it's common to re-create a window in the app when the
		// dock icon is clicked and there are no other windows open.
		if (BrowserWindow.getAllWindows().length === 0) createWindow();
	});
});

// Quit when all windows are closed.
app.on('window-all-closed', () => {
	// On macOS it is common for applications and their menu bar
	// to stay active until the user quits explicitly with Cmd + Q
	if (process.platform !== 'darwin') {
		app.quit();
	}
});

async function openImage(
	_event: Electron.IpcMainInvokeEvent,
	options: Electron.OpenDialogSyncOptions
): Promise<void> {
	let response: Electron.OpenDialogReturnValue | undefined = undefined;
	try {
		response = await dialog.showOpenDialog(mainWindow as BrowserWindow, options);
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

async function saveImage(
	_event: Electron.IpcMainInvokeEvent,
	options: Electron.SaveDialogOptions
): Promise<void> {
	let response: Electron.SaveDialogReturnValue | undefined = undefined;
	try {
		const parts: ParsedPath = ImageManager.getOriginalImagePathParts();
		const defaultFileName =
			parts.name + '-' + new Date().toLocaleDateString().replace(/[^0-9]+/g, '-') + '.png';
		response = await dialog.showSaveDialog(mainWindow as BrowserWindow, {
			...options,
			// Save next to the original image by default
			defaultPath: join(parts.dir, defaultFileName),
		});
		if (response.canceled || !response.filePath) {
			return;
		}
		let filePath = response.filePath;
		if (!extname(filePath)) {
			filePath += '.png';
		}
		// Write the modified image to the save location in the format of the chosen extension
		await fs.writeFile(
			filePath,
			await ImageManager.convertToFormat(
				ImageManager.getLastModifiedImageBuffer(),
				extname(filePath).substring(1)
			)
		);

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

async function setLanguage(_event: Electron.IpcMainInvokeEvent, language: string): Promise<void> {
	try {
		await i18n.changeLanguage(language);
	} catch (err) {
		throw new CustomError('Failed to change the language', err, { language });
	}
}

async function getColorAtPixel(
	_event: Electron.IpcMainInvokeEvent,
	x: number,
	y: number
): Promise<string> {
	return ColorPicker.getColorAtMouse(mainWindow as BrowserWindow, x, y);
}

// eslint-disable-next-line @typescript-eslint/no-explicit-any
function callRendererFunction(func: RendererCallFunction, funcArgs: any[]): void {
	(mainWindow as BrowserWindow).webContents.send('mainprocess-response', {
		func,
		funcArgs,
	} as RendererCallFunctionRequest);
}

const applyModifications = throttle(
	async (_event: Electron.IpcMainInvokeEvent, modifications: ImageModificationInfo[]) => {
		try {
			callRendererFunction('setModificationInProgress', [true]);
			const modifiedImageInfos: ImageInfo[] =
				await ImageManager.updateModifications(modifications);

			const modifiedImages = modifiedImageInfos.map((imageInfo): FrontendImageInfo => ({
				base64: imageInfo.buffer.toString('base64'),
				dimensions: imageInfo.dimensions,
			}));
			callRendererFunction('setModifiedImages', [modifiedImages]);
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
};

ipcMain.handle(
	'request-mainprocess-action',
	async (event: Electron.IpcMainInvokeEvent, arg: ElectronCallFunctionRequest) => {
		try {
			const func: ((evt, ...args) => void) | undefined = callableFunctions[arg.func];
			if (!func) {
				throw new CustomError(
					"Received func request on request-mainprocess-action, but function doesn't exist1",
					null,
					{
						event,
						arg,
					}
				);
			}
			return await func(event, ...arg.funcArgs);
		} catch (err: unknown) {
			callRendererFunction('showNotification', [
				i18n.t((err as { message: string })?.message),
				'error',
				err,
			]);
			console.error(err);
		}
	}
);
