import { Sharp } from 'sharp';

import ImageManager from '../image-manager';
import { ImageInfo } from '../modifications';

export function rotate(
	imageInfo: ImageInfo,
	angle: number,
	backgroundColor: string
): Promise<Buffer> {
	const sharp: Sharp = ImageManager.bufferToSharp(imageInfo.buffer);

	const state = sharp.rotate(angle, { background: backgroundColor });

	return ImageManager.sharpToBuffer(state);
}
