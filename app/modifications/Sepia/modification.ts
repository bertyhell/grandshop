import ImageManager from '../../electron/image-manager';
import { ImageInfo } from '../modifications';
import { Sharp } from 'sharp';

export function sepia(imageInfo: ImageInfo): Promise<Buffer> {
  const sharp: Sharp = ImageManager.bufferToSharp(imageInfo.buffer);

  const state = sharp
    .recomb([
      [0.3588, 0.7044, 0.1368],
      [0.2990, 0.5870, 0.1140],
      [0.2392, 0.4696, 0.0912],
    ]);

  return ImageManager.sharpToBuffer(state);
}
