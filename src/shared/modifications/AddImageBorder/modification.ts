import sharp, { Sharp } from 'sharp';

import ImageManager from '../../../electron/image-manager';
import { ImageInfo } from '../modifications';
import { analyseBorderPath } from './border-image-analyser';
import path from 'path';
import { getImageDimensions } from '../../helpers/get-image-dimensions';

export async function addImageBorder(
	imageInfo: ImageInfo,
	borderPath: string,
	borderWidth: number,
): Promise<Buffer> {
	try {
		const sharpImage: Sharp = ImageManager.bufferToSharp(imageInfo.buffer);

		const borderInfo = analyseBorderPath(borderPath);
		if (!borderInfo) {
			return imageInfo.buffer;
		}

		const scaleFactor = borderWidth / borderInfo.x;
		let borderImage = await sharp(path.resolve(borderPath));
		const borderImageInfo = await getImageDimensions(borderImage);
		borderImage = await sharp(await borderImage.resize(Math.round(scaleFactor * borderImageInfo.width)).toBuffer());
		const scale = (size: number): number => {
			return size * scaleFactor;
		};
		const floor = Math.floor;

		// Corners
		const topLeftCorner = borderImage.clone().extract({
			left: 0,
			top: 0,
			width: floor(scale(borderInfo.x)),
			height: floor(scale(borderInfo.y)),
		});
		const topRightCorner = borderImage.clone().extract({
			left: floor(scale(borderImageInfo.width - borderInfo.x)),
			top: 0,
			width: floor(scale(borderInfo.x)),
			height: floor(scale(borderInfo.y)),
		});
		const bottomLeftCorner = borderImage.clone().extract({
			left: 0,
			top: floor(scale(borderImageInfo.height - borderInfo.y)),
			width: floor(scale(borderInfo.x)),
			height: floor(scale(borderInfo.y)),
		});
		const bottomRightCorner = borderImage.clone().extract({
			left: floor(scale(borderImageInfo.width - borderInfo.x)),
			top: floor(scale(borderImageInfo.height - borderInfo.y)),
			width: floor(scale(borderInfo.x)),
			height: floor(scale(borderInfo.y)),
		});

		// Horizontal edges
		const borderTopEdgeWidth = scale(borderImageInfo.width - borderInfo.x * 2);
		const imageTopEdgeSpace = imageInfo.dimensions.width - scale(borderInfo.x) * 2;
		const numberOfHorizontalSegments = imageTopEdgeSpace / borderTopEdgeWidth;
		const roundedNumberOfHorizontalSegments = Math.round(numberOfHorizontalSegments);
		let horizontalScaleFactor: number;
		if (numberOfHorizontalSegments > roundedNumberOfHorizontalSegments) {
			horizontalScaleFactor = 1 - Math.abs(numberOfHorizontalSegments - roundedNumberOfHorizontalSegments) / numberOfHorizontalSegments;
		} else {
			horizontalScaleFactor = 1 + Math.abs(numberOfHorizontalSegments - roundedNumberOfHorizontalSegments) / numberOfHorizontalSegments;
		}
		const topEdgeSegments = [];
		const bottomEdgeSegments = [];
		const topSegmentWidth = borderImageInfo.width - borderInfo.x * 2;
		const topEdgeSegmentBuffer = await borderImage
			.clone()
			.extract({
				left: floor(scale(borderInfo.x)),
				top: 0,
				width: floor(scale(topSegmentWidth)),
				height: floor(scale(borderInfo.y)),
			})
			.resize(floor(scale(topSegmentWidth * horizontalScaleFactor)), floor(scale(borderInfo.y)), {fit: 'fill'	})
			.toBuffer();
		const bottomEdgeSegmentBuffer = await borderImage
			.clone()
			.extract({
				left: floor(scale(borderInfo.x)),
				top: floor(scale(borderImageInfo.height - borderInfo.y)),
				width: floor(scale(topSegmentWidth)),
				height: floor(scale(borderInfo.y)),
			})
			.resize(floor(scale(topSegmentWidth * horizontalScaleFactor)), floor(scale(borderInfo.y)), {fit: 'fill'	})
			.toBuffer();
		for (let i = 0; i < roundedNumberOfHorizontalSegments; i += 1) {
			topEdgeSegments.push({
				input: topEdgeSegmentBuffer,
				gravity: 'northwest',
				left: floor(scale(borderInfo.x + topSegmentWidth / horizontalScaleFactor * i)),
				top: 0,
			});
			bottomEdgeSegments.push({
				input: bottomEdgeSegmentBuffer,
				gravity: 'northwest',
				left: floor(scale(borderInfo.x + topSegmentWidth / horizontalScaleFactor * i)),
				top: borderImageInfo.height - floor(scale(borderInfo.y)),
			});
		}

		// Vertical edges
		const borderLeftEdgeHeight = scale(borderImageInfo.height - borderInfo.y * 2);
		const imageLeftEdgeSpace = imageInfo.dimensions.height - borderWidth * 2;
		const numberOfVerticalSegments = imageLeftEdgeSpace / borderLeftEdgeHeight;
		const roundedNumberOfVerticalSegments = Math.round(numberOfVerticalSegments);
		let verticalScaleFactor: number;
		if (numberOfVerticalSegments > roundedNumberOfVerticalSegments) {
			verticalScaleFactor = 1 - Math.abs(numberOfVerticalSegments - roundedNumberOfVerticalSegments) / numberOfVerticalSegments;
		} else {
			verticalScaleFactor = 1 + Math.abs(numberOfVerticalSegments - roundedNumberOfVerticalSegments) / numberOfVerticalSegments;
		}
		const leftEdgeSegments = [];
		const rightEdgeSegments = [];
		const leftSegmentHeight = borderImageInfo.height - borderInfo.y * 2;
		const leftEdgeSegmentBuffer = await borderImage
			.clone()
			.extract({
				left: 0,
				top: floor(scale(borderInfo.y)),
				width: floor(scale(borderInfo.x)),
				height: floor(scale(leftSegmentHeight)),
			})
			.resize(floor(scale(borderInfo.x)), floor(scale(leftSegmentHeight / verticalScaleFactor)), {fit: 'fill'	})
			.toBuffer();
		const rightEdgeSegmentBuffer = await borderImage
			.clone()
			.extract({
				left: floor(scale(borderImageInfo.width - borderInfo.x)),
				top: floor(scale(borderInfo.y)),
				width: floor(scale(borderInfo.x)),
				height: floor(scale(leftSegmentHeight)),
			})
			.resize(floor(scale(borderInfo.y)), floor(scale(leftSegmentHeight / verticalScaleFactor)), {fit: 'fill'	})
			.toBuffer();
		for (let i = 0; i < roundedNumberOfVerticalSegments; i += 1) {
			leftEdgeSegments.push({
				input: leftEdgeSegmentBuffer,
				gravity: 'northwest',
				left: 0,
				top: floor(scale(borderInfo.y + leftSegmentHeight / verticalScaleFactor * i)),
			});
			rightEdgeSegments.push({
				input: rightEdgeSegmentBuffer,
				gravity: 'northwest',
				left: imageInfo.dimensions.width - floor(scale(borderInfo.x)),
				top: floor(scale(borderInfo.y + leftSegmentHeight / verticalScaleFactor * i)),
			});
		}

		sharpImage.composite([
			// Corners
			{ input: await topLeftCorner.toBuffer(), gravity: 'northwest' },
			{ input: await topRightCorner.toBuffer(), gravity: 'northeast' },
			{ input: await bottomLeftCorner.toBuffer(), gravity: 'southwest' },
			{ input: await bottomRightCorner.toBuffer(), gravity: 'southeast' },

			// // Edges
			...topEdgeSegments,
			...bottomEdgeSegments,
			...leftEdgeSegments,
			...rightEdgeSegments,
		]);

		return ImageManager.sharpToBuffer(sharpImage);
	} catch (err) {
		console.error(err);
		throw err;
	}
}
