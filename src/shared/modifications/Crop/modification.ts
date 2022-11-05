import { Sharp } from 'sharp';

import ImageManager from '../../../electron/image-manager';
import { ImageInfo } from '../modifications';

export function crop(
	imageInfo: ImageInfo,
	top: number,
	right: number,
	bottom: number,
	left: number
): Promise<Buffer> {
	const sharp: Sharp = ImageManager.bufferToSharp(imageInfo.buffer);

	const state = sharp.extract({
		width: imageInfo.dimensions.width - right - left,
		height: imageInfo.dimensions.height - top - bottom,
		left,
		top,
	});

	return ImageManager.sharpToBuffer(state);
}
