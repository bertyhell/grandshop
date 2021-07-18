import { ImageInfo } from '../modifications/modifications';
import fs from 'fs-extra';
import { getImageDimensions } from './get-image-dimensions';
import ImageManager from '../../electron/image-manager';

export async function loadImage(path: string): Promise<ImageInfo> {
	const sharpImg = await ImageManager.bufferToSharp(await fs.readFile(path));
	const buffer = await sharpImg.png().toBuffer();

	return {
		buffer,
		dimensions: await getImageDimensions(sharpImg),
	}
}
