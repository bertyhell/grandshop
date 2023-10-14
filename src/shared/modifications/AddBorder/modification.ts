import ImageManager from '../image-manager';
import { ImageInfo } from '../modifications';
import { Sharp } from 'sharp';

export function addBorder(imageInfo: ImageInfo, thickness: number, color: string): Promise<Buffer> {
	const sharp: Sharp = ImageManager.bufferToSharp(imageInfo.buffer);

	const state = sharp.extend({
		top: thickness,
		bottom: thickness,
		left: thickness,
		right: thickness,
		background: color,
	});

	return ImageManager.sharpToBuffer(state);
}
