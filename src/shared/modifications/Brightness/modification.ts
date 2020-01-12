import { State } from 'gm';
import ImageManager from '../../../electron/image-manager';
import { ImageInfo } from '../modifications';

export function brightness(imageInfo: ImageInfo, brightness: number): Promise<Buffer> {
  const gmState: State = ImageManager.bufferToGm(imageInfo.buffer);

  const state = (gmState.modulate as any)(brightness);

  return ImageManager.gmToBuffer(state);
}
