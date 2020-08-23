// Modules to control application life and create native browser window
import {
	ElectronCallFunctionRequest,
	RendererCallFunction,
	RendererCallFunctionRequest,
} from '../utils/types';
import { ParsedPath } from 'path';
import { dialog, ipcMain } from 'electron';
import fs from 'fs-extra';
import throttle from '@jcoreio/async-throttle';

import { ImageModificationInfo } from '../modifications/types';
import i18n from '../translations/i18n';

import ImageManager from './image-manager';
import { CustomError } from '../utils/error';
import { ColorPicker } from './color-picker';
import { getMainWindow } from '../main.dev';

export interface Dimensions {
  width: number;
  height: number;
}

async function openImage(
	options: Electron.OpenDialogSyncOptions
) {
	let response: Electron.OpenDialogReturnValue | undefined = undefined;
	try {
		response = await dialog.showOpenDialog(await getMainWindow, options);
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

async function saveImage(options: Electron.SaveDialogOptions) {
	let response: Electron.SaveDialogReturnValue | undefined = undefined;
	try {
		const parts: ParsedPath = ImageManager.getOriginalImagePathParts();
		const defaultFileName =
			parts.name + '-' + new Date().toLocaleDateString().replace(/[^0-9]+/g, '-') + '.png';
		response = await dialog.showSaveDialog(await getMainWindow, {
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

async function setLanguage(language: string) {
	try {
		await i18n.changeLanguage(language);
	} catch (err) {
		throw new CustomError('Failed to change the language', err, { language });
	}
}

async function getColorAtPixel(
	x: number,
	y: number
): Promise<string> {
	return ColorPicker.getColorAtMouse(await getMainWindow, x, y);
}

const applyModifications = throttle(
	async (modifications: ImageModificationInfo[]) => {
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
};

async function callRendererFunction(func: RendererCallFunction, funcArgs: any[]) {
  (await getMainWindow).webContents.send('mainprocess-response', {
    func,
    funcArgs,
  } as RendererCallFunctionRequest);
}

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
			return await func(...arg.funcArgs);
		} catch (err) {
			callRendererFunction('showNotification', [i18n.t(err.message), 'error', err]);
			console.error(err);
		}
	}
);
