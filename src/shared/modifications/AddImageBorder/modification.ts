import sharp, { Sharp } from 'sharp';

import ImageManager from '../../../electron/image-manager';
import { ImageInfo } from '../modifications';
import { analyseBorderPath } from './border-image-analyser';

export async function addImageBorder(imageInfo: ImageInfo, borderPath: string): Promise<Buffer> {
  const sharpImage: Sharp = ImageManager.bufferToSharp(imageInfo.buffer);

  const borderInfo = analyseBorderPath(borderPath);
  if (!borderInfo) {
    return imageInfo.buffer;
  }

  const borderImage = sharp(borderPath);
  const topLeftCorner = borderImage.clone().extract({ left: 0, top: 0, width: borderInfo.x, height: borderInfo.y });

  sharpImage.composite([{ input: await topLeftCorner.toBuffer(), gravity: 'northwest' }]);

  return ImageManager.sharpToBuffer(sharpImage);
}
