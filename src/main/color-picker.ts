import type { BrowserWindow } from 'electron';
import { CustomError } from '../shared/error';

function toHex(value: number): string {
	return value.toString(16).padStart(2, '0');
}

export class ColorPicker {
	public static async getColorAtMouse(win: BrowserWindow, x: number, y: number): Promise<string> {
		try {
			const img: Electron.NativeImage = await win.webContents.capturePage({
				x,
				y,
				width: 1,
				height: 1,
			});
			// Bitmap data is in BGRA order
			const rawData = img.toBitmap();
			return '#' + toHex(rawData[2]) + toHex(rawData[1]) + toHex(rawData[0]);
		} catch (err) {
			throw new CustomError('Failed to get pixel color', err, { x, y });
		}
	}
}
