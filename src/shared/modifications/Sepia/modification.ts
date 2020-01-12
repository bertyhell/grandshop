import gm, { State } from 'gm';
import ImageManager from '../../../electron/image-manager';
import { ImageInfo } from '../modifications';

export function sepia(imageInfo: ImageInfo, brightness: number, color: string, redPercent: number, greenPercent: number, bluePercent: number): Promise<Buffer> {
  const gmState: State = ImageManager.bufferToGm(imageInfo.buffer);

  gm(imageInfo.dimensions.width, imageInfo.dimensions.height, color);

  const state = gmState
      .modulate(brightness, 0, 100)
      .fill(color)
      .colorize(redPercent, greenPercent, bluePercent);

  return ImageManager.gmToBuffer(state);
}
