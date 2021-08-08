import sharp, { Sharp } from 'sharp';

import ImageManager from '../../../electron/image-manager';
import { ImageInfo } from '../modifications';
import { analyseBorderPath } from './border-image-analyser';
import path from 'path';
import { getImageDimensions } from '../../helpers/get-image-dimensions';
import { CustomError } from '../../error';

export async function addImageBorder(
	imageInfo: ImageInfo,
	borderPath: string,
	borderWidth: number,
): Promise<Buffer> {
	try {
		const sharpImage: Sharp = ImageManager.bufferToSharp(imageInfo.buffer);

		const borderInfo = analyseBorderPath(borderPath);
		if (!borderInfo) {
			throw new CustomError('border path does not have the expected format: name_xx_xx_xx_xx.png');
		}

		const borderImage = await sharp(path.resolve(borderPath));
		const borderImageInfo = await getImageDimensions(borderImage);
		const scaleFactor = borderWidth / borderInfo.cornerX;
		const scale = (size: number): number => {
			return size * scaleFactor;
		};
		const floor = Math.floor;


		// Corners
		const topLeftCorner = borderImage.clone().extract({
			left: 0,
			top: 0,
			width: borderInfo.cornerX,
			height: borderInfo.cornerY,
		}).resize(floor(scale(borderInfo.cornerX)), floor(scale(borderInfo.cornerY)), {fit: 'fill'});
		const topRightCorner = borderImage.clone().extract({
			left: borderImageInfo.width - borderInfo.cornerX,
			top: 0,
			width: borderInfo.cornerX,
			height: borderInfo.cornerY,
		}).resize(floor(scale(borderInfo.cornerX)), floor(scale(borderInfo.cornerY)), {fit: 'fill'});
		const bottomLeftCorner = borderImage.clone().extract({
			left: 0,
			top: borderImageInfo.height - borderInfo.cornerY,
			width: borderInfo.cornerX,
			height: borderInfo.cornerY,
		}).resize(floor(scale(borderInfo.cornerX)), floor(scale(borderInfo.cornerY)), {fit: 'fill'});
		const bottomRightCorner = borderImage.clone().extract({
			left: borderImageInfo.width - borderInfo.cornerX,
			top: borderImageInfo.height - borderInfo.cornerY,
			width: borderInfo.cornerX,
			height: borderInfo.cornerY,
		}).resize(floor(scale(borderInfo.cornerX)), floor(scale(borderInfo.cornerY)), {fit: 'fill'});

		// Horizontal edges
		// Extract the top edge repeatable part and the bottom edge repeatable part from the border image
		const topEdgeSegmentBuffer = await borderImage
			.clone()
			.extract({
				left: borderInfo.cornerX,
				top: 0,
				width: borderInfo.edgeX,
				height: borderInfo.cornerY,
			})
			.toBuffer();
		const bottomEdgeSegmentBuffer = await borderImage
			.clone()
			.extract({
				left: borderInfo.cornerX,
				top: borderImageInfo.height - borderInfo.cornerY,
				width: borderInfo.edgeX,
				height: borderInfo.cornerY,
			})
			.toBuffer();
		// Add the repeatable parts to fill up the required image space between the corner parts
		const imageTopEdgeSpace = imageInfo.dimensions.width - scale(borderInfo.cornerX) * 2;
		const numberOfHorizontalSegments = imageTopEdgeSpace / scale(borderInfo.edgeX);
		const roundedNumberOfHorizontalSegments = Math.round(numberOfHorizontalSegments);
		const topEdgeSegments = [];
		const bottomEdgeSegments = [];
		for (let i = 0; i < roundedNumberOfHorizontalSegments; i += 1) {
			topEdgeSegments.push({
				input: topEdgeSegmentBuffer,
				gravity: 'northwest',
				left: borderInfo.edgeX * i,
				top: 0,
			});
			bottomEdgeSegments.push({
				input: bottomEdgeSegmentBuffer,
				gravity: 'northwest',
				left: borderInfo.edgeX * i,
				top: 0,
			});
		}
		const topEdgeImage = await sharp({
			create: {
				channels: 4,
				width: borderInfo.edgeX * roundedNumberOfHorizontalSegments,
				height: borderInfo.cornerY,
				background: { r: 0, g: 0, b: 0, alpha: 0 },
			},
		}).png();
		const bottomEdgeImage = await sharp({
			create: {
				channels: 4,
				width: borderInfo.edgeX * roundedNumberOfHorizontalSegments,
				height: borderInfo.cornerY,
				background: { r: 0, g: 0, b: 0, alpha: 0 },
			},
		}).png();

		// Vertical edges
		const leftEdgeSegmentBuffer = await borderImage
			.clone()
			.extract({
				left: 0,
				top: borderInfo.cornerY,
				width: borderInfo.cornerX,
				height: borderInfo.edgeY,
			})
			.toBuffer();
		const rightEdgeSegmentBuffer = await borderImage
			.clone()
			.extract({
				left: borderImageInfo.width - borderInfo.cornerX,
				top: borderInfo.cornerY,
				width: borderInfo.cornerX,
				height: borderInfo.edgeY,
			})
			.toBuffer();
		// Add the repeatable parts to fill up the required image space between the corner parts
		const imageLeftEdgeSpace = imageInfo.dimensions.height - scale(borderInfo.cornerY) * 2;
		const numberOfVerticalSegments = imageLeftEdgeSpace / scale(borderInfo.edgeY);
		const roundedNumberOfVerticalSegments = Math.round(numberOfVerticalSegments);
		const leftEdgeSegments = [];
		const rightEdgeSegments = [];
		for (let i = 0; i < roundedNumberOfVerticalSegments; i += 1) {
			leftEdgeSegments.push({
				input: leftEdgeSegmentBuffer,
				gravity: 'northwest',
				left: 0,
				top: borderInfo.edgeY * i,
			});
			rightEdgeSegments.push({
				input: rightEdgeSegmentBuffer,
				gravity: 'northwest',
				left: 0,
				top: borderInfo.edgeY * i,
			});
		}
		const leftEdgeImage = await sharp({
			create: {
				channels: 4,
				width: borderInfo.cornerX,
				height: borderInfo.edgeY * roundedNumberOfVerticalSegments,
				background: { r: 0, g: 0, b: 0, alpha: 0 },
			},
		}).png();
		const rightEdgeImage = await sharp({
			create: {
				channels: 4,
				width: borderInfo.cornerX,
				height: borderInfo.edgeY * roundedNumberOfVerticalSegments,
				background: { r: 0, g: 0, b: 0, alpha: 0 },
			},
		}).png();


		// Scale image to desired width
		const topEdgeImageScaled = await sharp(await topEdgeImage.composite(topEdgeSegments).toBuffer()).resize(imageTopEdgeSpace, floor(scale(borderInfo.cornerY)), { fit: 'fill' });
		const bottomEdgeImageScaled = await sharp(await bottomEdgeImage.composite(bottomEdgeSegments).toBuffer()).resize(imageTopEdgeSpace, floor(scale(borderInfo.cornerY)), { fit: 'fill' });

		const leftEdgeImageScaled = await sharp((await leftEdgeImage.composite(leftEdgeSegments).toBuffer())).resize(floor(scale(borderInfo.cornerX)), imageLeftEdgeSpace, { fit: 'fill' });
		const rightEdgeImageScaled = await sharp((await rightEdgeImage.composite(rightEdgeSegments).toBuffer())).resize(floor(scale(borderInfo.cornerX)), imageLeftEdgeSpace, { fit: 'fill' });

		sharpImage.composite([
			// Corners
			{ input: await topLeftCorner.toBuffer(), gravity: 'northwest' },
			{ input: await topRightCorner.toBuffer(), gravity: 'northeast' },
			{ input: await bottomLeftCorner.toBuffer(), gravity: 'southwest' },
			{ input: await bottomRightCorner.toBuffer(), gravity: 'southeast' },

			// Edges
			{ input: await topEdgeImageScaled.toBuffer(), left: scale(borderInfo.cornerX), top: 0 },
			{
				input: await bottomEdgeImageScaled.toBuffer(),
				left: scale(borderInfo.cornerX),
				top: imageInfo.dimensions.height - scale(borderInfo.cornerY),
			},
			{ input: await leftEdgeImageScaled.toBuffer(), left: 0, top: scale(borderInfo.cornerY) },
			{
				input: await rightEdgeImageScaled.toBuffer(),
				left: imageInfo.dimensions.width - scale(borderInfo.cornerX),
				top: scale(borderInfo.cornerY),
			},
		]);

		return ImageManager.sharpToBuffer(sharpImage);
	} catch (err) {
		console.error(err);
		throw err;
	}
}
