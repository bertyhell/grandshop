import { State } from 'gm';
import ImageManager from '../../../electron/image-manager';
import { ImageInfo } from '../modifications';

export function sepia(imageInfo: ImageInfo): Promise<Buffer> {
  const gmState: State = ImageManager.bufferToGm(imageInfo.buffer);

  const state = gmState.sepia();

  return ImageManager.gmToBuffer(state);
}
