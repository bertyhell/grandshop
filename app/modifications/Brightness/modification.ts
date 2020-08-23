import { Sharp } from 'sharp';

import ImageManager from '../../electron/image-manager';
import { ImageInfo } from '../modifications';

export function brightness(imageInfo: ImageInfo, brightness: number): Promise<Buffer> {
  const sharp: Sharp = ImageManager.bufferToSharp(imageInfo.buffer);

  const state = sharp.modulate({
    brightness
  });

  return ImageManager.sharpToBuffer(state);
}
