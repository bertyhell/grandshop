import { State } from 'gm';
import ImageManager from '../../../electron/image-manager';
import { ImageInfo } from '../modifications';

export function addBorder(imageInfo: ImageInfo, thickness: number, color: string): Promise<Buffer> {
  const gmState: State = ImageManager.bufferToGm(imageInfo.buffer);

  const state = gmState
      .borderColor(color)
      .border(thickness, thickness);

  return ImageManager.gmToBuffer(state);
}
