import { BrowserWindow } from "electron";
import { CustomError } from '../shared/error';
import Color from 'color';

export class ColorPicker {
  public static async getColorAtMouse(win: BrowserWindow, x: number, y: number): Promise<string> {
    try {
      const img: Electron.NativeImage = await win.webContents.capturePage({ x, y, width: 1, height: 1 });
      const rawData = img.toBitmap();
      return '#' + Color({
        r: rawData[2],
        g: rawData[1],
        b: rawData[0],
      }).alpha(rawData[3] / 255.0).hex().substring(1);
    } catch (err) {
      throw new CustomError('Failed to get pixel color', err, {x, y});
    }
  }
}
