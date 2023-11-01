import ImageManager from '../image-manager';
import { ImageInfo } from '../modifications';
import { Sharp } from 'sharp';

export function greyscale(imageInfo: ImageInfo): Promise<Buffer> {
	const sharp: Sharp = ImageManager.bufferToSharp(imageInfo.buffer);

	const state = sharp.greyscale();

	return ImageManager.sharpToBuffer(state);
}
