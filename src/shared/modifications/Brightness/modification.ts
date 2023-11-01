import { Sharp } from 'sharp';

import ImageManager from '../image-manager';
import { ImageInfo } from '../modifications';

export function brightness(
	imageInfo: ImageInfo,
	brightness: number,
	contrast: number
): Promise<Buffer> {
	const sharp: Sharp = ImageManager.bufferToSharp(imageInfo.buffer);

	const state = sharp
		.modulate({
			brightness,
		})
		.linear(contrast, 0);

	return ImageManager.sharpToBuffer(state);
}
