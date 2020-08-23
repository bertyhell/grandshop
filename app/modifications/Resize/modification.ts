import { Sharp } from 'sharp';

import ImageManager from '../../electron/image-manager';
import { ImageInfo } from '../modifications';

export function resize(imageInfo: ImageInfo, size: number): Promise<Buffer> {
  const sharp: Sharp = ImageManager.bufferToSharp(imageInfo.buffer);

  const state = sharp.resize(size, size, {fit: 'inside'});

  return ImageManager.sharpToBuffer(state);
}
