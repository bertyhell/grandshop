import { Sharp } from 'sharp';

import ImageManager from '../image-manager';
import { ImageInfo } from '../modifications';

export function mirror(
	imageInfo: ImageInfo,
	horizontal: boolean,
	vertical: boolean
): Promise<Buffer> {
	let sharp: Sharp = ImageManager.bufferToSharp(imageInfo.buffer);

	if (horizontal) {
		sharp = sharp.flop();
	}
	if (vertical) {
		sharp = sharp.flip();
	}

	return ImageManager.sharpToBuffer(sharp);
}
