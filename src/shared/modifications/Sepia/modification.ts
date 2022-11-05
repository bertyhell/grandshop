import ImageManager from '../../../electron/image-manager';
import { ImageInfo } from '../modifications';
import { Sharp } from 'sharp';

export function sepia(imageInfo: ImageInfo): Promise<Buffer> {
	const sharp: Sharp = ImageManager.bufferToSharp(imageInfo.buffer);

	const state = sharp.recomb([
		[0.393, 0.769, 0.189],
		[0.349, 0.686, 0.168],
		[0.272, 0.534, 0.131],
	]);

	return ImageManager.sharpToBuffer(state);
}
