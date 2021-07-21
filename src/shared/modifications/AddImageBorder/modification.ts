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

		const scaleFactor = borderWidth / borderInfo.cornerX;
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
			width: floor(scale(borderInfo.cornerX)),
			height: floor(scale(borderInfo.cornerY)),
		});
		const topRightCorner = borderImage.clone().extract({
			left: floor(scale(borderImageInfo.width - borderInfo.cornerX)),
			top: 0,
			width: floor(scale(borderInfo.cornerX)),
			height: floor(scale(borderInfo.cornerY)),
		});
		const bottomLeftCorner = borderImage.clone().extract({
			left: 0,
			top: floor(scale(borderImageInfo.height - borderInfo.cornerY)),
			width: floor(scale(borderInfo.cornerX)),
			height: floor(scale(borderInfo.cornerY)),
		});
		const bottomRightCorner = borderImage.clone().extract({
			left: floor(scale(borderImageInfo.width - borderInfo.cornerX)),
			top: floor(scale(borderImageInfo.height - borderInfo.cornerY)),
			width: floor(scale(borderInfo.cornerX)),
			height: floor(scale(borderInfo.cornerY)),
		});

		// Horizontal edges
		const imageTopEdgeSpace = imageInfo.dimensions.width - scale(borderInfo.cornerX) * 2;
		const numberOfHorizontalSegments = imageTopEdgeSpace / scale(borderInfo.edgeX);
		const roundedNumberOfHorizontalSegments = Math.round(numberOfHorizontalSegments);
		let horizontalScaleFactor: number;
		if (numberOfHorizontalSegments > roundedNumberOfHorizontalSegments) {
			horizontalScaleFactor = 1 - Math.abs(numberOfHorizontalSegments - roundedNumberOfHorizontalSegments) / numberOfHorizontalSegments;
		} else {
			horizontalScaleFactor = 1 + Math.abs(numberOfHorizontalSegments - roundedNumberOfHorizontalSegments) / numberOfHorizontalSegments;
		}
		const topEdgeSegments = [];
		const bottomEdgeSegments = [];
		const topEdgeSegmentBuffer = await borderImage
			.clone()
			.extract({
				left: floor(scale(borderInfo.cornerX)),
				top: 0,
				width: floor(scale(borderInfo.edgeX)),
				height: floor(scale(borderInfo.cornerY)),
			})
			.resize(floor(scale(borderInfo.edgeX * horizontalScaleFactor)), floor(scale(borderInfo.cornerY)), {fit: 'fill'	})
			.toBuffer();
		const bottomEdgeSegmentBuffer = await borderImage
			.clone()
			.extract({
				left: floor(scale(borderInfo.cornerX)),
				top: floor(scale(borderImageInfo.height - borderInfo.cornerY)),
				width: floor(scale(borderInfo.edgeX)),
				height: floor(scale(borderInfo.cornerY)),
			})
			.resize(floor(scale(borderInfo.edgeX * horizontalScaleFactor)), floor(scale(borderInfo.cornerY)), {fit: 'fill'	})
			.toBuffer();
		for (let i = 0; i < roundedNumberOfHorizontalSegments; i += 1) {
			topEdgeSegments.push({
				input: topEdgeSegmentBuffer,
				gravity: 'northwest',
				left: floor(scale(borderInfo.cornerX + borderInfo.edgeX / horizontalScaleFactor * i)),
				top: 0,
			});
			bottomEdgeSegments.push({
				input: bottomEdgeSegmentBuffer,
				gravity: 'northwest',
				left: floor(scale(borderInfo.cornerX + borderInfo.edgeX / horizontalScaleFactor * i)),
				top: borderImageInfo.height - floor(scale(borderInfo.cornerY)),
			});
		}

		// Vertical edges
		const imageLeftEdgeSpace = imageInfo.dimensions.height - borderWidth * 2;
		const numberOfVerticalSegments = imageLeftEdgeSpace / scale(borderInfo.edgeY);
		const roundedNumberOfVerticalSegments = Math.round(numberOfVerticalSegments);
		let verticalScaleFactor: number;
		if (numberOfVerticalSegments > roundedNumberOfVerticalSegments) {
			verticalScaleFactor = 1 - Math.abs(numberOfVerticalSegments - roundedNumberOfVerticalSegments) / numberOfVerticalSegments;
		} else {
			verticalScaleFactor = 1 + Math.abs(numberOfVerticalSegments - roundedNumberOfVerticalSegments) / numberOfVerticalSegments;
		}
		const leftEdgeSegments = [];
		const rightEdgeSegments = [];
		const leftEdgeSegmentBuffer = await borderImage
			.clone()
			.extract({
				left: 0,
				top: floor(scale(borderInfo.cornerY)),
				width: floor(scale(borderInfo.cornerX)),
				height: floor(scale(borderInfo.edgeY)),
			})
			.resize(floor(scale(borderInfo.cornerX)), floor(scale(borderInfo.edgeY / verticalScaleFactor)), {fit: 'fill'	})
			.toBuffer();
		const rightEdgeSegmentBuffer = await borderImage
			.clone()
			.extract({
				left: floor(scale(borderImageInfo.width - borderInfo.cornerX)),
				top: floor(scale(borderInfo.cornerY)),
				width: floor(scale(borderInfo.cornerX)),
				height: floor(scale(borderInfo.edgeY)),
			})
			.resize(floor(scale(borderInfo.cornerY)), floor(scale(borderInfo.edgeY / verticalScaleFactor)), {fit: 'fill'	})
			.toBuffer();
		for (let i = 0; i < roundedNumberOfVerticalSegments; i += 1) {
			leftEdgeSegments.push({
				input: leftEdgeSegmentBuffer,
				gravity: 'northwest',
				left: 0,
				top: floor(scale(borderInfo.cornerY + borderInfo.edgeY / verticalScaleFactor * i)),
			});
			rightEdgeSegments.push({
				input: rightEdgeSegmentBuffer,
				gravity: 'northwest',
				left: imageInfo.dimensions.width - floor(scale(borderInfo.cornerX)),
				top: floor(scale(borderInfo.cornerY + borderInfo.edgeY / verticalScaleFactor * i)),
			});
		}

		sharpImage.composite([
			// Corners
			{ input: await topLeftCorner.toBuffer(), gravity: 'northwest' },
			{ input: await topRightCorner.toBuffer(), gravity: 'northeast' },
			{ input: await bottomLeftCorner.toBuffer(), gravity: 'southwest' },
			{ input: await bottomRightCorner.toBuffer(), gravity: 'southeast' },

			// Edges
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
