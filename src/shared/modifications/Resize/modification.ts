import { State } from 'gm';
import ImageManager from '../../../electron/image-manager';
import { ImageInfo } from '../modifications';

export function resize(imageInfo: ImageInfo, size: number): Promise<Buffer> {
  const gmState: State = ImageManager.bufferToGm(imageInfo.buffer);

  const state = gmState.resize(size, size);

  return ImageManager.gmToBuffer(state);
}
