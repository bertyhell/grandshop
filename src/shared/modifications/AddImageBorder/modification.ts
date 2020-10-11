import sharp, { Sharp } from 'sharp';

import ImageManager from '../../../electron/image-manager';
import { ImageInfo } from '../modifications';
import { analyseBorderPath } from './border-image-analyser';
import path from 'path';
import { getImageDimensions } from '../../helpers/get-image-dimensions';

export async function addImageBorder(
	imageInfo: ImageInfo,
	borderPath: string,
	borderWidth: number
): Promise<Buffer> {
	const sharpImage: Sharp = ImageManager.bufferToSharp(imageInfo.buffer);

	const borderInfo = analyseBorderPath(borderPath);
	if (!borderInfo) {
		return imageInfo.buffer;
	}

	const borderImage = sharp(path.join(borderPath));
	let borderImageInfo = await getImageDimensions(borderImage);
	borderImage.resize(Math.round((borderImageInfo.width * borderWidth) / 100));
	borderImageInfo = await getImageDimensions(borderImage);
	const scale = (size: number): number => {
		const scaleFactor =
			Math.round((borderImageInfo.width * borderWidth) / 100) / borderImageInfo.width;
		return Math.floor(size * scaleFactor);
	};
	const topLeftCorner = borderImage.clone().extract({
		left: 0,
		top: 0,
		width: scale(borderInfo.x),
		height: scale(borderInfo.y),
	});
	const topRightCorner = borderImage.clone().extract({
		left: scale(borderImageInfo.width - borderInfo.x),
		top: 0,
		width: scale(borderInfo.x),
		height: scale(borderInfo.y),
	});
	const BottomLeftCorner = borderImage.clone().extract({
		left: 0,
		top: scale(borderImageInfo.height - borderInfo.y),
		width: scale(borderInfo.x),
		height: scale(borderInfo.y),
	});
	const bottomRightCorner = borderImage.clone().extract({
		left: scale(borderImageInfo.width - borderInfo.x),
		top: scale(borderImageInfo.height - borderInfo.y),
		width: scale(borderInfo.x),
		height: scale(borderInfo.y),
	});

	sharpImage.composite([
		{ input: await topLeftCorner.toBuffer(), gravity: 'northwest' },
		{ input: await topRightCorner.toBuffer(), gravity: 'northeast' },
		{ input: await BottomLeftCorner.toBuffer(), gravity: 'southwest' },
		{ input: await bottomRightCorner.toBuffer(), gravity: 'southeast' },
	]);

	await sharpImage.clone().png().toFile('C:/Users/Bert/Desktop/test.png');

	return ImageManager.sharpToBuffer(sharpImage);
}
